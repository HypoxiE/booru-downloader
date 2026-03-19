mod api;
mod models;

use api::yandere::{BooruRequest, RATING, REQUEST};

#[tokio::main]
async fn main() {
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
}