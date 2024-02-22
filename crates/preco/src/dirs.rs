use directories::ProjectDirs;
use std::path::PathBuf;

pub(crate) fn get_cache_dir() -> PathBuf {
    ProjectDirs::from("", "", "preco")
        .unwrap()
        .cache_dir()
        .to_path_buf()
}

pub(crate) fn get_checkouts_dir() -> PathBuf {
    get_cache_dir().join("checkouts")
}
