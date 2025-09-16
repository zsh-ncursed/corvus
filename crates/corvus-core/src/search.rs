use std::path::PathBuf;
use crate::app_state::DirEntry;
use tokio::io::AsyncReadExt;

#[derive(Debug, Clone, Default)]
pub enum SearchMode {
    #[default]
    Name,
    Content,
    Metadata,
}

pub struct SearchEngine;

impl SearchEngine {
    pub fn search_entries(entries: &[DirEntry], query: &str, mode: &SearchMode) -> Vec<DirEntry> {
        match mode {
            SearchMode::Name => {
                entries.iter()
                    .filter(|entry| {
                        entry.name.to_lowercase().contains(&query.to_lowercase())
                    })
                    .cloned()
                    .collect()
            },
            SearchMode::Content => {
                // For content search, we would need to read file contents
                // This is a simplified implementation for now
                entries.iter()
                    .filter(|entry| {
                        if entry.is_dir {
                            false
                        } else {
                            entry.name.to_lowercase().contains(&query.to_lowercase())
                        }
                    })
                    .cloned()
                    .collect()
            },
            SearchMode::Metadata => {
                // For metadata search, we would check file metadata
                // This is a simplified implementation for now
                entries.iter()
                    .filter(|entry| {
                        entry.name.to_lowercase().contains(&query.to_lowercase())
                    })
                    .cloned()
                    .collect()
            }
        }
    }
    
    pub async fn search_content(path: &PathBuf, query: &str) -> Result<bool, std::io::Error> {
        // Check if the path is a file
        let metadata = tokio::fs::metadata(path).await?;
        if metadata.is_dir() {
            return Ok(false);
        }
        
        // Read the file content
        let mut file = tokio::fs::File::open(path).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;
        
        // Search for the query in the content
        Ok(contents.to_lowercase().contains(&query.to_lowercase()))
    }
    
    pub async fn search_in_directory(
        directory: &PathBuf, 
        query: &str, 
        mode: &SearchMode
    ) -> Result<Vec<DirEntry>, std::io::Error> {
        let mut results = Vec::new();
        
        // Read directory entries
        let mut entries = tokio::fs::read_dir(directory).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let is_dir = entry.file_type().await?.is_dir();
            
            match mode {
                SearchMode::Name => {
                    if name.to_lowercase().contains(&query.to_lowercase()) {
                        results.push(DirEntry { name, path, is_dir });
                    }
                },
                SearchMode::Content => {
                    if !is_dir {
                        if let Ok(matches) = Self::search_content(&path, query).await {
                            if matches {
                                results.push(DirEntry { name, path, is_dir });
                            }
                        }
                    }
                },
                SearchMode::Metadata => {
                    // For metadata search, we would check file metadata
                    // This is a simplified implementation for now
                    if name.to_lowercase().contains(&query.to_lowercase()) {
                        results.push(DirEntry { name, path, is_dir });
                    }
                }
            }
        }
        
        Ok(results)
    }
}