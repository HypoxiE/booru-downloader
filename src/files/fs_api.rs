use directories::ProjectDirs;
use std::path::PathBuf;

pub fn get_cache_path(name: &str) -> PathBuf {
	if let Some(proj_dirs) = ProjectDirs::from("com", "HypoxiE", "catgirl-parser") {
		let mut path: PathBuf = proj_dirs.config_dir().to_path_buf();
		std::fs::create_dir_all(&path).expect("Failed to create .config directory");
		path.push(name);
		path
	} else {
		panic!("Cannot determine project directory for {}", name);
	}
}