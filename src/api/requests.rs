use reqwest::{Client};
use std::{collections::HashMap};
use strfmt::strfmt;

use crate::cache_save_load::CACHE_COUNT_IMAGES;
use crate::models::{api_responses::APIResponses};
use crate::files::config_parse::CONFIGURATIONS;

use chrono::prelude::Utc;

#[allow(dead_code)]
pub enum RATING {
	S,
	E,
	Q,
	ALL,
}

pub enum RequestType {
	Request(String),
	RandomTemplate([String; 2], u16),
}

pub struct Request {
	pub booru_url: String,
	pub rtype: RequestType,
}

impl Request {
	
	async fn fetch_request(&self, client: &Client, request: &str) -> anyhow::Result<APIResponses> {
		let body = client
			.get(request)
			.header("User-Agent", "Mozilla/5.0")
			.send()
			.await?
			.text()
			.await?;

		let response = CONFIGURATIONS.map_api_responses(&self.booru_url, &body).expect("обязательно исправлю");
		Ok(response)
	}

	pub async fn get_images(&self, client: &Client) -> anyhow::Result<APIResponses> {
		match &self.rtype {
			RequestType::Request(request) => {
				Self::fetch_request(&self, client, &request).await
			}
			RequestType::RandomTemplate([template, clarification], limit) => {

				let mut total_images: u64 = 0;

				let now_timestamp: u64 = Utc::now().timestamp() as u64;

				let cache = CACHE_COUNT_IMAGES.read().await;
				let fresh = if let Some(&[timestamp, images]) = cache.get(template) {
					total_images = images;
					timestamp >= now_timestamp - 60 * 60 * 3
				} else {
					false
				};
				drop(cache);

				if !fresh {
					let max_size_page = CONFIGURATIONS.get_image_data(&self.booru_url, "max_limit")?.parse::<u64>()?;
					let mut page_size_lim: u64 = 1;
					total_images = loop {
						
						let result: APIResponses = Self::fetch_request(
							&self,
							client,
							&strfmt(&clarification, &{
								let mut v: HashMap<String, String> = HashMap::new();
								v.insert("image".to_string(), page_size_lim.to_string());
								v
							})?,
						).await.expect("я это обязательно исправлю");

						if result.len() < max_size_page as usize && result.len() > 0 {
							break (page_size_lim - 1) * max_size_page + result.len() as u64;
						}
						if result.is_empty() {
							let mut low: u64 = page_size_lim / 2;
							let mut high: u64 = page_size_lim;
							while low < high {
								let mid: u64 = (low + high + 1) / 2;

								let result: APIResponses = Self::fetch_request(
									&self,
									client,
									&strfmt(&clarification, &{
										let mut v: HashMap<String, String> = HashMap::new();
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
							let last_page_data: APIResponses = Self::fetch_request(
								&self,
								client,
								&strfmt(&clarification, &{
									let mut v: HashMap<String, String> = HashMap::new();
									v.insert("image".to_string(), last_page.to_string());
									v
								})?,
							).await.expect("я это обязательно исправлю");

							break (last_page - 1) * max_size_page + last_page_data.len() as u64;
						}
						page_size_lim *= 2;
					};


					let mut cache = CACHE_COUNT_IMAGES.write().await;
					cache.insert(template.to_owned(), [now_timestamp, total_images]);
					drop(cache);
				}

				let mut result: APIResponses = APIResponses::new();

				for _ in 0..*limit {
					let url: String = strfmt(&template, &{
						let mut v: HashMap<String, String> = HashMap::new();
						v.insert("image".to_string(), fastrand::u64(1..=total_images).to_string());
						v
					})?;

					let mut answer: APIResponses = Self::fetch_request(&self, client, &url).await.expect("я это обязательно исправлю");
					result.append(&mut answer);
				}
				
				Ok(result)
			}
		}
	}
}

pub struct BooruRequest{
	api_url: String,
	limit: u16,
	page: u64,
	is_random: bool,
	rating: RATING,
	tags: Vec<String>,
}
impl BooruRequest {
	pub fn new(api_url: String) -> Self {
		let limit: u16 = 1;
		let page: u64 = 1;

		let is_random: bool = false;
		let rating: RATING = RATING::S;
		let tags: Vec<String> = vec![];

        Self {api_url, limit, page, is_random, rating, tags}
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

	pub fn build(&self) -> Request {
		let add_args: String = CONFIGURATIONS.get_image_data(&self.api_url, "arguments").unwrap_or_else(|_| "".to_string());
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

				format!("tags={}", {truetags.sort(); truetags}.join("+"))
			};
			
			let mut args: Vec<String> = vec![limit, page, tags];
			if !add_args.is_empty() {
				args.push(add_args.to_string());
			}

			Request {
				booru_url: self.api_url.to_owned(),
				rtype: RequestType::Request(format!("{}?{}", self.api_url, args.join("&")))
			}
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

				format!("tags={}", {truetags.sort(); truetags}.join("+"))
			};
			let max_size_page: u64 = CONFIGURATIONS.get_image_data(&self.api_url, "max_limit").unwrap().parse::<u64>().unwrap();

			let common_args: Vec<String> = {
				let mut v = vec!["page={image}".to_string(), tags.to_owned()];
				if !add_args.is_empty() {
					v.push(add_args.to_string());
				}
				v
			};

			let mut args_templ = common_args.clone();
			args_templ.insert(0, "limit=1".to_string());

			
			let mut args_clar = common_args.clone();
			args_clar.insert(0, format!("limit={}", max_size_page));
			Request {
				booru_url: self.api_url.to_owned(),
				rtype: RequestType::RandomTemplate([format!("{}?{}", self.api_url, args_templ.join("&")), format!("{}?{}", self.api_url, args_clar.join("&"))], self.limit)
			}
		}
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn get_random_image_test() {
		let client: reqwest::Client = reqwest::Client::new();

		
		let request: BooruRequest = BooruRequest::new("https://yande.re/post.json".to_string())
			//.set_tag("blue_archive".to_string())
			//.randomize()
			.set_limit(3)
			.set_rating(RATING::S)
			.set_tag("blue_archive".to_string());
		
		match request.build().rtype {
			RequestType::Request(req) => {
				println!("{}", req);
			}
			RequestType::RandomTemplate([req, q], lim) => {
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