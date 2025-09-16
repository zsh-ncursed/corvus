use std::path::Path;
use walkdir::WalkDir;

pub fn get_directory_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .min_depth(1)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.metadata().ok())
        .filter(|metadata| metadata.is_file())
        .map(|metadata| metadata.len())
        .sum()
}
