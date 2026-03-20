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
			max_limit = 1000, -- maximum images, given api
			arguments = "",
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

		["https://danbooru.donmai.us/posts.json"] = {
			max_limit = 200,
			arguments = "",
			parse = function(json)
				local out = {}

				local function iso_to_unix(iso)
					local year, month, day, hour, min, sec, ms, tz_sign, tz_hour, tz_min =
						iso:match("(%d+)%-(%d+)%-(%d+)T(%d+):(%d+):(%d+)%.(%d+)([%+%-])(%d+):(%d+)")
					
					year = tonumber(year)
					month = tonumber(month)
					day = tonumber(day)
					hour = tonumber(hour)
					min = tonumber(min)
					sec = tonumber(sec)

					tz_hour = tonumber(tz_hour)
					tz_min = tonumber(tz_min)
					
					local tz_offset = tz_hour * 3600 + tz_min * 60
					if tz_sign == "-" then tz_offset = -tz_offset end

					local timestamp = os.time({year=year, month=month, day=day, hour=hour, min=min, sec=sec})
					
					return timestamp - tz_offset
				end

				for _, item in ipairs(json.data) do

					local tags = {}
					for tag in string.gmatch(item.tags, "%S+") do
						table.insert(tags, tag)
					end

					local sample_width = nil
					local sample_height = nil
					local preview_width = nil
					local preview_height = nil
					if item.media_assets then
						for _, im in ipairs(item.media_asset.variants) do
							if im.type == "180x180" then
								preview_width = im.width
								preview_height = im.height
							end
							if im.type == "sample" then
								sample_width = im.width
								sample_height = im.height
							end
						end
					end

					table.insert(out, {
						id = item.id,
						tags = tags,
						created_at = iso_to_unix(item.created_at),
						author = item.tag_string_artist ~= "" and item.tag_string_artist or nil,
						rating = item.rating,
						score = item.score,

						md5 = item.md5,
						file_ext = item.file_ext,

						file_url = item.file_url or item.source,
						sample_url = item.large_file_url or item.file_url or item.source,
						preview_url = item.preview_file_url or item.file_url or item.source,

						width = item.image_width,
						height = item.image_height,
						sample_width = sample_width or item.image_width,
						sample_height = sample_height or item.image_height,
						preview_width = preview_width or item.image_width,
						preview_height = preview_height or item.image_height,

						file_size = item.file_size,
						sample_file_size = item.file_size,

						jpeg_url = nil,
						jpeg_width = nil,
						jpeg_height = nil,
						jpeg_file_size = nil,

						has_children = false,
						parent_id = nil,
					})
				end
				return out
			end,
		}
	}
}