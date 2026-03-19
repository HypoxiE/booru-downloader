use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;

pub type YandereRoot = Vec<Yandere>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Yandere {
	pub id: i64,
	pub tags: String,
	#[serde(rename = "created_at")]
	pub created_at: Option<i64>,
	#[serde(rename = "updated_at")]
	pub updated_at: Option<i64>,
	#[serde(rename = "creator_id")]
	pub creator_id: Option<i64>,
	#[serde(rename = "approver_id")]
	pub approver_id: Option<i64>,
	pub author: String,
	pub change: Option<i64>,
	pub source: Option<String>,
	pub score: Option<i64>,
	pub md5: String,
	#[serde(rename = "file_size")]
	pub file_size: i64,
	#[serde(rename = "file_ext")]
	pub file_ext: String,
	#[serde(rename = "file_url")]
	pub file_url: String,
	#[serde(rename = "is_shown_in_index")]
	pub is_shown_in_index: bool,
	#[serde(rename = "preview_url")]
	pub preview_url: String,
	#[serde(rename = "preview_width")]
	pub preview_width: i64,
	#[serde(rename = "preview_height")]
	pub preview_height: i64,
	#[serde(rename = "actual_preview_width")]
	pub actual_preview_width: i64,
	#[serde(rename = "actual_preview_height")]
	pub actual_preview_height: i64,
	#[serde(rename = "sample_url")]
	pub sample_url: String,
	#[serde(rename = "sample_width")]
	pub sample_width: i64,
	#[serde(rename = "sample_height")]
	pub sample_height: i64,
	#[serde(rename = "sample_file_size")]
	pub sample_file_size: i64,
	#[serde(rename = "jpeg_url")]
	pub jpeg_url: String,
	#[serde(rename = "jpeg_width")]
	pub jpeg_width: i64,
	#[serde(rename = "jpeg_height")]
	pub jpeg_height: i64,
	#[serde(rename = "jpeg_file_size")]
	pub jpeg_file_size: i64,
	pub rating: String,
	#[serde(rename = "is_rating_locked")]
	pub is_rating_locked: bool,
	#[serde(rename = "has_children")]
	pub has_children: bool,
	#[serde(rename = "parent_id")]
	pub parent_id: Option<i64>,
	pub status: String,
	#[serde(rename = "is_pending")]
	pub is_pending: bool,
	pub width: i64,
	pub height: i64,
	#[serde(rename = "is_held")]
	pub is_held: bool,
	#[serde(rename = "frames_pending_string")]
	pub frames_pending_string: String,
	#[serde(rename = "frames_pending")]
	pub frames_pending: Vec<Value>,
	#[serde(rename = "frames_string")]
	pub frames_string: String,
	pub frames: Vec<Value>,
	#[serde(rename = "is_note_locked")]
	pub is_note_locked: bool,
	#[serde(rename = "last_noted_at")]
	pub last_noted_at: i64,
	#[serde(rename = "last_commented_at")]
	pub last_commented_at: i64,
}
