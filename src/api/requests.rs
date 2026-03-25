use reqwest::{Client};
use std::{collections::HashMap};
use sha2::{Sha512, Digest};

use crate::cache_save_load::CACHE_COUNT_IMAGES;
use crate::models::{api_responses::APIResponses};
use crate::files::config_parse::CONFIGURATIONS;

use tokio::time::{sleep, Duration};

use chrono::prelude::Utc;

#[allow(dead_code)]
#[derive(Clone)]
pub enum RATING {
	S,
	E,
	Q,
	ALL,
}

#[derive(Clone)]
pub struct BooruRequest{
	pub domain: String,
	pub limit: u16,
	pub page: u64,
	pub is_random: bool,
	pub rating: RATING,
	pub tags: Vec<String>,
}
//offline builder
impl BooruRequest {
	pub fn new(domain: String) -> Self {
		let limit: u16 = 1;
		let page: u64 = 1;

		let is_random: bool = false;
		let rating: RATING = RATING::S;
		let tags: Vec<String> = vec![];

        Self {domain, limit, page, is_random, rating, tags}
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
	pub fn get_rating(&self) -> String {
		match self.rating {
			RATING::S => "safe".to_string(),
			RATING::E => "explicit".to_string(),
			RATING::Q => "questionable".to_string(),
			RATING::ALL => "all".to_string()
		}
	}

	pub fn build(&self, client: &Client) -> reqwest::RequestBuilder {
		let mut mapper = CONFIGURATIONS.lock().unwrap();
		let (url, body): (String, String) = mapper.generate_url(&self);

		//println!("{}\n{}", url, body);

		let use_get_request: bool = mapper.get_api_parameter(&self.domain, "request_use_get");
		let mut raw_request: reqwest::RequestBuilder = if use_get_request {
			client.get(url)
		} else {
			client.post(url)
		};
		let headers: HashMap<String, String> = mapper.get_api_parameters_table(&self.domain, "request_headers");
		for (key, value) in headers {
			raw_request = raw_request.header(key, value);
		}

		raw_request.body(body)
	}

	pub fn get_max_lim(&self, page: u64) -> Self {
		let mut requester: BooruRequest = self.clone();
		let mapper = CONFIGURATIONS.lock().unwrap();
		requester.limit = mapper.get_api_parameter(&self.domain, "max_limit");
		requester.page = page;
		requester.is_random = false;

		requester
	}
	pub fn get_min_lim(&self, image: u64) -> Self {
		let mut requester: BooruRequest = self.clone();
		requester.limit = 1;
		requester.page = image;
		requester.is_random = false;

		requester
	}

	pub fn get_hash(&self) -> String {
		let mut tags: Vec<String> = self.tags.clone();
		tags.push(self.get_rating());
		tags.push(self.domain.to_owned());
		tags.sort();

		//let mut hasher = DefaultHasher::new();
		//tags.hash(&mut hasher);
		//hasher.finish()

		let mut hasher512 = Sha512::new();
		for s in &tags {
			hasher512.update(s.as_bytes());
		}
		let result512 = hasher512.finalize();
		format!("{:x}", result512)
	}

}

//Network fetcher
impl BooruRequest {
	async fn fetch_request(&self, client: &Client) -> anyhow::Result<APIResponses> {

		let request: reqwest::RequestBuilder = self.build(client);

		let response: String = request.send().await?.text().await?;

		let mut mapper = CONFIGURATIONS.lock().unwrap();
		let response: APIResponses = mapper.map_api_responses(&self.domain, &response);

		Ok(response)
	}

	pub async fn norandom_get_images(&self, client: &Client) -> anyhow::Result<APIResponses> {
		self.fetch_request(client).await
	}

	pub async fn get_images(&self, client: &Client) -> anyhow::Result<APIResponses> {

		let (use_self_randomizer, max_limit, request_timeout): (bool, u16, f32) = {
			let mapper = CONFIGURATIONS.lock().unwrap();
			(
				mapper.get_api_parameter(&self.domain, "use_self_randomize"),
				mapper.get_api_parameter(&self.domain, "max_limit"),
				mapper.get_api_parameter(&self.domain, "timeout_for_randomize")
			)
		};

		if !use_self_randomizer || !self.is_random {
			self.fetch_request(client).await
		} else {
			let mut total_images: u64 = 0;
			let now_timestamp: u64 = Utc::now().timestamp() as u64;

			let cache = CACHE_COUNT_IMAGES.read().await;
			let fresh = if let Some(&[timestamp, images]) = cache.get(&self.get_hash()) {
				total_images = images;
				timestamp >= now_timestamp - 60 * 60 * 3
			} else {
				false
			};
			drop(cache);

			if !fresh {
				let max_size_page: u16 = max_limit;
				let mut page_size_lim: u64 = 1;


				total_images = loop {


					let result: APIResponses = self.get_max_lim(page_size_lim).norandom_get_images(client).await?;
					sleep(Duration::from_secs_f32(request_timeout)).await;

					if result.len() < max_size_page as usize && result.len() > 0 {
						break (page_size_lim - 1) * max_size_page as u64 + result.len() as u64;
					}
					if result.is_empty() {

						if page_size_lim == 1 {
							break 0;
						}

						let mut low: u64 = page_size_lim / 2;
						let mut high: u64 = page_size_lim;
						while low < high {
							let mid: u64 = (low + high + 1) / 2;

							let result: APIResponses = self.get_max_lim(mid).norandom_get_images(client).await?;
							sleep(Duration::from_secs_f32(request_timeout)).await;

							if result.is_empty() {
								high = mid - 1;
							} else {
								low = mid;
							}
						};
						let last_page: u64 = low;

						let last_page_data: APIResponses = self.get_max_lim(last_page).norandom_get_images(client).await?;
						sleep(Duration::from_secs_f32(request_timeout)).await;

						break (last_page - 1) * max_size_page as u64 + last_page_data.len() as u64;
					}
					page_size_lim *= 2;
				};
				let mut cache = CACHE_COUNT_IMAGES.write().await;
				cache.insert(self.get_hash(), [now_timestamp, total_images]);
				drop(cache);
			}

			let mut result: APIResponses = APIResponses::new();
			if total_images == 0 {
				return Ok(result);
			}

			for _ in 0..self.limit as u64 {
				let mut answer = self.get_min_lim(fastrand::u64(1..=total_images)).norandom_get_images(client).await?;
				sleep(Duration::from_secs_f32(request_timeout)).await;
				result.append(&mut answer);
			}
			Ok(result)
		}
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn get_random_image_test() {
		let client: reqwest::Client = reqwest::Client::new();

		
		let request: BooruRequest = BooruRequest::new("yande.re".to_string())
			//.set_tag("blue_archive".to_string())
			//.randomize()
			.set_limit(3)
			.set_rating(RATING::S)
			.set_tag("blue_archive".to_string());
	

		let a: Vec<crate::models::api_responses::APIResponse> = request.get_images(&client).await.unwrap();
		println!("{:#?}", a);

		assert_eq!(a.len(), 3);
	}
}