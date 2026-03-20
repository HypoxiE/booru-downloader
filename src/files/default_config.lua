--pub id: i64,
--pub tags: Vec<String>,
--pub created_at: i64,
--pub author: Option<String>,
--pub rating: String,

--pub score: Option<i64>,
--pub md5: String,
--pub file_ext: String,

--pub file_url: String,
--pub sample_url: String,
--pub preview_url: String,

--pub width: i64,
--pub height: i64,
--pub sample_width: i64,
--pub sample_height: i64,
--pub preview_width: i64,
--pub preview_height: i64,

--pub file_size: i64,
--pub sample_file_size: i64,

--pub jpeg_url: Option<String>,
--pub jpeg_width: Option<i64>,
--pub jpeg_height: Option<i64>,
--pub jpeg_file_size: Option<i64>,

--pub has_children: bool,
--pub parent_id: Option<i64>,

return {
	configuration = {
		rating = "s",
	},

	sites = {
		["https://yande.re/post.json"] = {

			parse = function(json)
				local out = {}

				for _, item in ipairs(json) do

					local tags = {}
					for tag in string.gmatch(item.tags, "%S+") do
						table.insert(tags, tag)
					end
					
					table.insert(out, {
						id = item.id,
						tags = tags,
						created_at = item.created_at,
						author = item.author,
						rating = item.rating,
						score = item.score,
						md5 = item.md5,
						file_ext = item.file_ext,
						file_url = item.file_url,
						sample_url = item.sample_url,
						preview_url = item.preview_url,
						width = item.width,
						height = item.height,
						sample_width = item.sample_width,
						sample_height = item.sample_height,
						preview_width = item.preview_width,
						preview_height = item.preview_height,
						file_size = item.file_size,
						sample_file_size = item.sample_file_size,
						jpeg_url = item.jpeg_url,
						jpeg_width = item.jpeg_width,
						jpeg_height = item.jpeg_height,
						jpeg_file_size = item.jpeg_file_size,
						has_children = item.has_children or false,
						parent_id = item.parent_id,
					})
				end
				return out
			end,
		},

		["another-site.org"] = {
			parse = function(json)
				local out = {}
				for _, entry in ipairs(json.data) do
					table.insert(out, {
						id = entry.identifier,
						tags = entry.tags,
						created_at = entry.time,
						author = entry.user,
						rating = entry.rating_level,
						score = entry.score_value,
						md5 = entry.md5_hash,
						file_ext = entry.extension,
						file_url = entry.urls.file,
						sample_url = entry.urls.sample,
						preview_url = entry.urls.preview,
						width = entry.dim.width,
						height = entry.dim.height,
						sample_width = entry.dim_sample.width,
						sample_height = entry.dim_sample.height,
						preview_width = entry.dim_preview.width,
						preview_height = entry.dim_preview.height,
						file_size = entry.sizes.file,
						sample_file_size = entry.sizes.sample,
						jpeg_url = entry.jpeg and entry.jpeg.url,
						jpeg_width = entry.jpeg and entry.jpeg.width,
						jpeg_height = entry.jpeg and entry.jpeg.height,
						jpeg_file_size = entry.jpeg and entry.jpeg.size,
						has_children = entry.children_count > 0,
						parent_id = entry.parent,
					})
				end
				return out
			end,
		}
	}
}