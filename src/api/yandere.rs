use reqwest::{Client};
use std::{collections::HashMap};
use strfmt::strfmt;

use crate::models::yandere::{YandereRoot};

#[allow(dead_code)]
pub enum RATING {
	S,
	E,
	Q,
	ALL,
}

pub enum REQUEST {
	Request(String),
	RandomTemplate([String; 2], u16),
}
impl REQUEST {
	
	async fn fetch_request(client: &Client, request: &str) -> Result<YandereRoot, Box<dyn std::error::Error>> {
		let body = client
			.get(request)
			.header("User-Agent", "Mozilla/5.0")
			.send()
			.await?
			.json::<YandereRoot>()
			.await?;

		Ok(body)
	}

	pub async fn get_images(self, client: &Client) -> Result<YandereRoot, Box<dyn std::error::Error>> {
		match self {
			Self::Request(request) => {
				Self::fetch_request(client, &request).await
			}
			Self::RandomTemplate([template, clarification], limit) => {
				let clarification = clarification.clone();

				let mut page_s_1000: u64 = 1;
				let total_images: u64 = loop {
					
					let result: YandereRoot = Self::fetch_request(
						client,
						&strfmt(&clarification, &{
							let mut v = HashMap::new();
							v.insert("image".to_string(), page_s_1000.to_string());
							v
						})?,
					).await.expect("я это обязательно исправлю");

					if result.len() < 1000 && result.len() > 0 {
						break (page_s_1000 - 1) * 1000 + result.len() as u64;
					}
					if result.is_empty() {
						let mut low: u64 = page_s_1000 / 2;
						let mut high: u64 = page_s_1000;
						while low < high {
							let mid: u64 = (low + high + 1) / 2;

							let result: YandereRoot = Self::fetch_request(
								client,
								&strfmt(&clarification, &{
									let mut v = HashMap::new();
									v.insert("image".to_string(), mid.to_string());
									v
								})?,
							).await.expect("я это обязательно исправлю");

							if result.is_empty() {
								high = mid - 1;
							} else {
								low = mid;
							}
						};
						let last_page: u64 = low;
        				let last_page_data: YandereRoot = Self::fetch_request(
							client,
							&strfmt(&clarification, &{
								let mut v = HashMap::new();
								v.insert("image".to_string(), last_page.to_string());
								v
							})?,
						).await.expect("я это обязательно исправлю");

						break (last_page - 1) * 1000 + last_page_data.len() as u64;
					}
					page_s_1000 *= 2;
				};

				let mut result: YandereRoot = YandereRoot::new();

				for _ in 0..limit {
					let url = strfmt(&template, &{
						let mut v = HashMap::new();
						v.insert("image".to_string(), fastrand::u64(1..=total_images).to_string());
						v
					})?;

					let mut answer = Self::fetch_request(client, &url).await?;
					result.append(&mut answer);
				}
				
				Ok(result)
			}
		}
	}
}

pub struct BooruRequest{
	limit: u16,
	page: u64,
	is_random: bool,
	rating: RATING,
	tags: Vec<String>,
}
impl BooruRequest {
	pub fn new() -> Self {
		let limit: u16 = 1;
		let page: u64 = 1;

		let is_random: bool = false;
		let rating: RATING = RATING::S;
		let tags: Vec<String> = vec![];

        Self { limit, page, is_random, rating, tags}
	}

	pub fn set_limit(mut self, limit: u16) -> Self {
		self.limit = limit;
		self
	}
	pub fn set_rating(mut self, rating: RATING) -> Self {
		self.rating = rating;
		self
	}
	pub fn randomize(mut self) -> Self {
		self.is_random = true;
		self
	}
	pub fn set_tag(mut self, tag: String) -> Self {
		self.tags.push(tag.to_owned());
		self
	}

	pub fn build(&self) -> REQUEST {
		if !self.is_random {
			let limit: String = format!("limit={}", self.limit);
			let page: String = format!("page={}", self.page);
			let tags: String = {
				let mut truetags: Vec<String> = self.tags.to_owned();

				match self.rating {
					RATING::S => {
						truetags.push("rating:s".to_string());
					}
					RATING::E => {
						truetags.push("rating:e".to_string());
					}
					RATING::Q => {
						truetags.push("rating:q".to_string());
					}
					RATING::ALL => {}
				}

				format!("tags={}", truetags.join("+"))
			};
			
			let args: Vec<String> = vec![limit, page, tags];

			REQUEST::Request(format!("https://yande.re/post.json?{}", args.join("&")))
		} else {
			let tags: String = {
				let mut truetags: Vec<String> = self.tags.to_owned();

				match self.rating {
					RATING::S => {
						truetags.push("rating:s".to_string());
					}
					RATING::E => {
						truetags.push("rating:e".to_string());
					}
					RATING::Q => {
						truetags.push("rating:q".to_string());
					}
					RATING::ALL => {}
				}

				format!("tags={}", truetags.join("+"))
			};
			
			let args_templ: Vec<String> = vec!["limit=1".to_string(), "page={image}".to_string(), tags.to_owned()];
			let args_clar: Vec<String> = vec!["limit=1000".to_string(), "page={image}".to_string(), tags];
			REQUEST::RandomTemplate([format!("https://yande.re/post.json?{}", args_templ.join("&")), format!("https://yande.re/post.json?{}", args_clar.join("&"))], self.limit)
		}
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn get_random_image_test() {
		let client: reqwest::Client = reqwest::Client::new();

		
		let request: BooruRequest = BooruRequest::new()
			//.set_tag("blue_archive".to_string())
			.randomize()
			.set_limit(3)
			.set_rating(RATING::S)
			.set_tag("blue_archive".to_string());
		
		match request.build() {
			REQUEST::Request(req) => {
				println!("{}", req);
			}
			REQUEST::RandomTemplate([req, q], lim) => {
				println!("{}", req);
				println!("{}", q);
				println!("{}", lim);
			}
		}

		let a = request.build().get_images(&client).await.expect("я это обязательно исправлю");
		println!("{:#?}", a);

		assert_eq!(a.len(), 3);
	}
}