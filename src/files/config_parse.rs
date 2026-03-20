use mlua::{Lua, LuaSerdeExt, Function, Table};
use serde_json;

use std::fs;
use std::fs::File;
use std::io::Write;
use std::error::Error;

use std::path::PathBuf;
use once_cell::sync::Lazy;

use crate::files::fs_api;
use crate::models::api_responses::APIResponses;

static DEFAULT_CONFIG: &str = include_str!("default_config.lua");
static CONFIG_FILE: Lazy<PathBuf> = Lazy::new(|| fs_api::get_cache_path("config.lua"));

pub static CONFIGURATIONS: Lazy<LuaMapper> = Lazy::new(|| LuaMapper::new().unwrap());

pub struct LuaMapper {
	lua: Lua,
	mappings: Table,
}
impl LuaMapper {
	pub fn new() -> Result<Self, Box<dyn Error>> {
		let lua_data: String = match fs::read(&*CONFIG_FILE) {
			Ok(data) => {
				if data.is_empty() {
					let mut file: File = File::create(&*CONFIG_FILE)?;
					file.write_all(DEFAULT_CONFIG.as_bytes())?;
					DEFAULT_CONFIG.to_string()
				} else {
					String::from_utf8(data)?
				}
			}
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
				let mut file: File = File::create(&*CONFIG_FILE)?;
				file.write_all(DEFAULT_CONFIG.as_bytes())?;
				DEFAULT_CONFIG.to_string()
			}
			Err(e) => {
				return Err(Box::new(e));
			}
		};

		let lua: Lua = Lua::new();
		let mappings: Table = lua.load(lua_data).eval()?;

		Ok(Self { lua, mappings })
	}

	pub fn map_api_responses(&self, domain: &str, json: &String) -> anyhow::Result<APIResponses> {
		let json: serde_json::Value = match serde_json::from_str(json) {
			Ok(value) => value,
			Err(error) => {
				return Err(anyhow::anyhow!("Invalid json from {}: {} \n Original error: {}", domain, json, error.to_string()));
			}
		};

		let sites: mlua::Table = self.mappings.get("sites").expect("я это обязательно исправлю");
		let site: mlua::Table = sites.get(domain).expect("я это обязательно исправлю");
		let func: Function = site.get("parse").expect("я это обязательно исправлю");

		let lua_json: mlua::Value = self.lua.to_value(&json).expect("я это обязательно исправлю");
		let result: mlua::Value = func.call(&lua_json).expect("я это обязательно исправлю");

		let responses: APIResponses = self.lua.from_value(result).expect(&format!("Lua returns invalid data: {:?}", lua_json));

		Ok(responses)
	}

	pub fn get_image_data(&self, domain: &str, field: &str) -> anyhow::Result<String> {
		let sites: mlua::Table = self.mappings.get("sites")?;
		let site_config: mlua::Table = sites.get(domain)?;
		let value: String = site_config.get(field)?;
		Ok(value)
	}

	pub fn get_setting(&self, field: &str) -> anyhow::Result<String> {
		let config: mlua::Table = self.mappings.get("configuration")?;
		let value: String = config.get(field)?;
		Ok(value)
	}
}