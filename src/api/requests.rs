use reqwest::Client;
use std::{collections::HashMap};
use tinytemplate::{TinyTemplate, format_unescaped};

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
		
		let mut raw_request: reqwest::RequestBuilder = client.get(request);

		let headers: HashMap<String, String> = CONFIGURATIONS.get_api_parameters_table(&self.booru_url, "request_headers")?;
		for (key, value) in headers {
			raw_request = raw_request.header(key, value);
		}

		let response: String = raw_request.send().await?.text().await?;
		let response: APIResponses = CONFIGURATIONS.map_api_responses(&self.booru_url, &response).expect("обязательно исправлю");

		Ok(response)
	}

	pub async fn get_images(&self, client: &Client) -> anyhow::Result<APIResponses> {
		match &self.rtype {
			RequestType::Request(request) => {
				Self::fetch_request(&self, client, &request).await
			}
			RequestType::RandomTemplate([template, clarification], limit) => {

				let mut tt = TinyTemplate::new();
				tt.add_template("url_template", &template).unwrap();
				tt.add_template("url_clarific", &clarification).unwrap();

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
					let max_size_page = CONFIGURATIONS.get_api_parameter_string(&self.booru_url, "max_limit")?.parse::<u64>()?;
					let mut page_size_lim: u64 = 1;

					total_images = loop {

						let check_page = &serde_json::json!({
							"page": page_size_lim
						});

						let result: APIResponses = Self::fetch_request(
							&self,
							client,
							&tt.render("url_clarific", check_page).unwrap(),
						).await.expect("я это обязательно исправлю");

						if result.len() < max_size_page as usize && result.len() > 0 {
							break (page_size_lim - 1) * max_size_page + result.len() as u64;
						}
						if result.is_empty() {

							if page_size_lim == 1 {
								break 0;
							}

							let mut low: u64 = page_size_lim / 2;
							let mut high: u64 = page_size_lim;
							while low < high {
								let mid: u64 = (low + high + 1) / 2;

								let check_page = &serde_json::json!({
									"page": mid
								});

								let result: APIResponses = Self::fetch_request(
									&self,
									client,
									&tt.render("url_clarific", check_page).unwrap(),
								).await.expect("я это обязательно исправлю");

								if result.is_empty() {
									high = mid - 1;
								} else {
									low = mid;
								}
							};
							let last_page: u64 = low;

							let check_page = &serde_json::json!({
								"page": last_page
							});
							let last_page_data: APIResponses = Self::fetch_request(
								&self,
								client,
								&tt.render("url_clarific", check_page).unwrap(),
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

				if total_images == 0 {
					return Ok(result);
				}

				for _ in 0..*limit {
					let check_page = &serde_json::json!({
						"page": fastrand::u64(1..=total_images).to_string()
					});
					let url: String = tt.render("url_template", check_page).unwrap();

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
		let url: String = CONFIGURATIONS.get_api_parameter_string(&self.api_url, "request_url").unwrap();
		let tags_separator: String = CONFIGURATIONS.get_api_parameter_string(&self.api_url, "request_tags_separator").unwrap();

		let use_booru_random: bool = CONFIGURATIONS.get_api_parameter_boolean(&self.api_url, "use_booru_ratings").unwrap();

		let mut tt = TinyTemplate::new();
		tt.add_template("url", &url).unwrap();
		tt.set_default_formatter(&format_unescaped);

		
		let limit: String = format!("limit={}", self.limit);
		let page: String = format!("page={}", self.page);
		let tags: String = {
			let mut truetags: Vec<String> = self.tags.to_owned();

			if use_booru_random {
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
			};
			
			{truetags.sort(); truetags}.join(&tags_separator)
		};

		if !self.is_random {
			let args = &serde_json::json!({
				"limit": limit,
				"page": page,
				"tags": tags
			});
			
			Request {
				booru_url: self.api_url.to_owned(),
				rtype: RequestType::Request(tt.render("url", args).unwrap())
			}
		} else {
			let max_size_page: u64 = CONFIGURATIONS.get_api_parameter_string(&self.api_url, "max_limit").unwrap().parse::<u64>().unwrap();

			let args_templ = &serde_json::json!({
				"limit": 1,
				"page": "{page}",
				"tags": tags
			});

			let args_clar = &serde_json::json!({
				"limit": max_size_page,
				"page": "{page}",
				"tags": tags
			});

			Request {
				booru_url: self.api_url.to_owned(),
				rtype: RequestType::RandomTemplate([tt.render("url", args_templ).unwrap(), tt.render("url", args_clar).unwrap()], self.limit)
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