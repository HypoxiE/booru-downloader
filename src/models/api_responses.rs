use serde_derive::Deserialize;
use serde_derive::Serialize;

pub type APIResponses = Vec<APIResponse>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct APIResponse {
	pub id: i64,
	pub tags: Vec<String>,
	#[serde(rename = "created_at")]
	pub created_at: i64,
	pub author: Option<String>,
	pub rating: String,

	pub score: Option<i64>,
	pub md5: String,
	#[serde(rename = "file_ext")]
	pub file_ext: String,

	#[serde(rename = "file_url")]
	pub file_url: String,
	#[serde(rename = "sample_url")]
	pub sample_url: String,
	#[serde(rename = "preview_url")]
	pub preview_url: String,

	pub width: i64,
	pub height: i64,
	#[serde(rename = "sample_width")]
	pub sample_width: i64,
	#[serde(rename = "sample_height")]
	pub sample_height: i64,
	#[serde(rename = "preview_width")]
	pub preview_width: i64,
	#[serde(rename = "preview_height")]
	pub preview_height: i64,
	

	#[serde(rename = "file_size")]
	pub file_size: i64,
	#[serde(rename = "sample_file_size")]
	pub sample_file_size: i64,


	#[serde(rename = "jpeg_url")]
	pub jpeg_url: Option<String>,
	#[serde(rename = "jpeg_width")]
	pub jpeg_width: Option<i64>,
	#[serde(rename = "jpeg_height")]
	pub jpeg_height: Option<i64>,
	#[serde(rename = "jpeg_file_size")]
	pub jpeg_file_size: Option<i64>,


	#[serde(rename = "has_children")]
	pub has_children: bool,
	#[serde(rename = "parent_id")]
	pub parent_id: Option<i64>,
}
