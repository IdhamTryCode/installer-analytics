use std::path::{Path, PathBuf};

pub const ENV_TEMPLATE: &str = include_str!("../env_template");

pub fn find_file(filename: &str) -> bool {
    if Path::new(filename).exists() {
        return true;
    }

    let parent_path = format!("../../{}", filename);
    Path::new(&parent_path).exists()
}

pub fn project_root() -> PathBuf {
    let mut current = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    if current
        .to_str()
        .map(|s| s.contains("target"))
        .unwrap_or(false)
    {
        if let Some(parent) = current.parent().and_then(|p| p.parent()) {
            current = parent.to_path_buf();
        }
    }

    current
}
