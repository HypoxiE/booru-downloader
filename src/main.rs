mod api;
mod models;
mod files;

use tokio::time::Duration;
use tokio::time::sleep;

use files::{cache_save_load, config_parse};
use api::requests::{BooruRequest, RATING};

#[tokio::main]
async fn main() {
	let a = &config_parse::CONFIGURATIONS;
	println!("{:?}", a.get_setting("rating").await.unwrap());
	cache_save_load::load_cache_from_file().await.expect("я обязательно это исправлю");

	tokio::spawn(cache_save_load::auto_dump_cache(1));

	let client: reqwest::Client = reqwest::Client::new();

	let request: BooruRequest = BooruRequest::new()
			//.set_tag("blue_archive".to_string())
			.randomize()
			.set_limit(4)
			.set_rating(RATING::S)
			.set_tag("width:>=1920".to_string())
			.set_tag("height:<1500".to_string())
			.set_tag("nekopara".to_string());

	match request.build() {
		api::requests::REQUEST::Request(req) => {
			println!("{}", req);
		}
		api::requests::REQUEST::RandomTemplate([req, q], lim) => {
			println!("{}", req);
			println!("{}", q);
			println!("{}", lim);
		}
	}

	let a = request.build().get_images(&client).await.expect("я это обязательно исправлю");
	println!("{:#?}", a);

	sleep(Duration::from_secs(30)).await;

}