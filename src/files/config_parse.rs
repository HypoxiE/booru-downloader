use mlua::{FromLua, Function, Lua, LuaSerdeExt, Table};
use serde_json;

use std::fs;
use std::fs::File;
use std::io::Write;
use std::error::Error;
use std::sync::Mutex;

use std::path::PathBuf;
use once_cell::sync::Lazy;

use std::collections::HashMap;

use crate::api::requests::BooruRequest;
use crate::files::fs_api;
use crate::models::api_responses::APIResponses;

static DEFAULT_CONFIG: &str = include_str!("default_config.lua");
static CONFIG_FILE: Lazy<PathBuf> = Lazy::new(|| fs_api::get_cache_path("config.lua"));

pub static CONFIGURATIONS: Lazy<Mutex<LuaMapper>> = Lazy::new(|| {
	Mutex::new(LuaMapper::new().unwrap())
});

pub struct LuaMapper {
	lua: Lua,
	mappings: Table,

	cached_functions: HashMap<String, Function>,
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

		let lua: Lua = Lua::new_with(mlua::StdLib::ALL_SAFE, mlua::LuaOptions::default())?;
		let json_encode = lua.create_function(|_, val: mlua::Value| {
			Ok(serde_json::to_string(&val).unwrap())
		})?;
		lua.globals().set("json_encode", json_encode)?;

		let mappings: Table = lua.load(lua_data).eval()?;

		let cached_functions: HashMap<String, Function> = HashMap::<String, Function>::new();

		Ok(Self { lua, mappings, cached_functions })
	}

	pub fn map_api_responses(&mut self, domain: &str, json: &String) -> APIResponses {
		//println!("{}", json);
		let json: serde_json::Value = serde_json::from_str(json).unwrap_or_else(|e| panic!("Invalid JSON from {}: {}\nOriginal error: {}", domain, json, e));
			

		let key: String = format!("sites.{}.parse_response", domain);
		
		let func: Function = match self.cached_functions.get(&key) {
			Some(func) => {func.clone()},
			None => {
				let sites: mlua::Table = self.mappings.get("sites")
					.expect("Key 'sites' is not found in config.lua");
				let site: mlua::Table = sites.get(domain)
					.expect(&format!("Key 'sites.{}' is not found in config.lua", domain));
				let func: Function = site.get("parse_response")
					.expect(&format!("Function '{}' is not found in config.lua", key));

				self.cached_functions.insert(key.clone(), func.clone());
				func
			}
		};
		//println!("{:?}", json);
		let lua_json: mlua::Value = self.lua.to_value(&json).expect("Cannot cast json to lua format");
		let result: mlua::Value = func.call(&lua_json).expect(&format!("Cannot call 'sites.{}.parse_response' function with (json) argument", domain));

		let responses: APIResponses = self.lua.from_value(result).expect(&format!("Lua returns invalid data: {:?}", lua_json));

		responses
	}

	pub fn generate_url(&mut self, request: &BooruRequest) -> (String, String) {
		let domain: &String = &request.domain;

		let key: String = format!("sites.{}.make_request", domain);
		let func: Function = match self.cached_functions.get(&key) {
			Some(func) => {func.clone()},
			None => {
				let sites: mlua::Table = self.mappings.get("sites")
					.expect("Key 'sites' is not found in config.lua");
				let site: mlua::Table = sites.get(domain.to_owned())
					.expect(&format!("Key 'sites.{}' is not found in config.lua", domain));
				let func: Function = site.get("make_request")
					.expect(&format!("Function '{}' is not found in config.lua", key));

				self.cached_functions.insert(key.clone(), func.clone());
				func
			}
		};
		//println!("lim: {}, page: {}, tags: {:?}", request.limit, request.page, request.tags);
		func.call((request.limit, request.page, request.is_random, request.get_rating(), request.tags.to_owned())).expect(&format!("Cannot call 'sites.{}.make_request' function with (json) argument", domain))
	}

	pub fn get_api_parameter<T: mlua::FromLua>(&self, domain: &str, field: &str) -> T {
		let sites: mlua::Table = self.mappings.get("sites").expect("Key 'sites' is not found in config.lua");
		let site_config: mlua::Table = sites.get(domain).expect(&format!("Key 'sites.{}' is not found in config.lua", domain));
		let value: T = site_config.get(field).expect(&format!("Key 'sites.{}.{}' is not found in config.lua", domain, field));
		value
	}

	#[allow(unused)]
	pub fn get_api_parameter_json(&self, domain: &str, field: &str) -> anyhow::Result<String> {
		let sites: mlua::Table = self.mappings.get("sites")?;
		let site_config: mlua::Table = sites.get(domain)?;
		let value: mlua::Value = site_config.get(field)?;

		let table = match value {
			mlua::Value::Table(t) => t,
			_ => anyhow::bail!("Field '{}' is not a Lua table", field),
		};

		let json = self.lua.from_value::<serde_json::Value>(mlua::Value::Table(table)).expect(
			&format!("Error: cannot make json from {:?}", field)
		);
		Ok(serde_json::to_string(&json)?)
	}

	pub fn get_api_parameters_table(&self, domain: &str, field: &str) -> HashMap<String, String> {
		let sites: mlua::Table = self.mappings.get("sites").expect("Key 'sites' is not found in config.lua");
		let site_config: mlua::Table = sites.get(domain).expect(&format!("Key 'sites.{}' is not found in config.lua", domain));
		let headers: mlua::Table = site_config.get(field).expect(&format!("Key 'sites.{}.{}' is not found in config.lua", domain, field));

		let mut map: HashMap<String, String> = HashMap::new();
		for pair in headers.pairs::<String, String>() {
			if let Ok((k, v)) = pair {
				map.insert(k, v);
			}
		}

		map
	}

	#[allow(unused)]
	pub fn get_setting<T: FromLua>(&self, field: &str) -> T {
		let config: mlua::Table = self.mappings.get("configuration").expect("Key 'configuration' is not found in config.lua");
		let value: T = config.get(field).expect(&format!("Key 'configuration.{}' is not found in config.lua", field));
		value
	}
}