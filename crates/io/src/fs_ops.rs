use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tokio::fs;
use tokio::sync::mpsc;
use uuid::Uuid;
use std::fs::File;
use std::io::Write;
use zip::{ZipWriter, write::FileOptions};
use tar::Builder;
use flate2::write::GzEncoder;
use flate2::Compression;

#[derive(Debug)]
pub enum ProgressEvent {
    Update(f32),
    Completed,
    Error(String),
}

pub async fn copy_file_task(
    task_id: Uuid,
    src: PathBuf,
    dest: PathBuf,
    progress_tx: mpsc::Sender<(Uuid, ProgressEvent)>,
) {
    let result = fs::copy(&src, &dest).await;
    match result {
        Ok(_) => {
            let _ = progress_tx.send((task_id, ProgressEvent::Completed)).await;
        }
        Err(e) => {
            let _ = progress_tx.send((task_id, ProgressEvent::Error(e.to_string()))).await;
        }
    }
}

pub async fn chown_task(
    task_id: Uuid,
    path: PathBuf,
    owner: String,
    progress_tx: mpsc::Sender<(Uuid, ProgressEvent)>,
) {
    let result = tokio::process::Command::new("sudo")
        .arg("chown")
        .arg(owner)
        .arg(path)
        .output()
        .await;

    match result {
        Ok(output) => {
            if output.status.success() {
                let _ = progress_tx.send((task_id, ProgressEvent::Completed)).await;
            } else {
                let error_message = String::from_utf8_lossy(&output.stderr).to_string();
                let _ = progress_tx.send((task_id, ProgressEvent::Error(error_message))).await;
            }
        }
        Err(e) => {
            let _ = progress_tx.send((task_id, ProgressEvent::Error(e.to_string()))).await;
        }
    }
}

pub async fn chmod_task(
    task_id: Uuid,
    path: PathBuf,
    mode: u32,
    progress_tx: mpsc::Sender<(Uuid, ProgressEvent)>,
) {
    let result = fs::set_permissions(&path, std::fs::Permissions::from_mode(mode)).await;
    match result {
        Ok(_) => {
            let _ = progress_tx.send((task_id, ProgressEvent::Completed)).await;
        }
        Err(e) => {
            let _ = progress_tx.send((task_id, ProgressEvent::Error(e.to_string()))).await;
        }
    }
}

pub async fn create_directory_task(
    task_id: Uuid,
    path: PathBuf,
    progress_tx: mpsc::Sender<(Uuid, ProgressEvent)>,
) {
    let result = fs::create_dir(path).await;
    match result {
        Ok(_) => {
            let _ = progress_tx.send((task_id, ProgressEvent::Completed)).await;
        }
        Err(e) => {
            let _ = progress_tx.send((task_id, ProgressEvent::Error(e.to_string()))).await;
        }
    }
}

pub async fn create_file_task(
    task_id: Uuid,
    path: PathBuf,
    progress_tx: mpsc::Sender<(Uuid, ProgressEvent)>,
) {
    let result = fs::File::create(path).await;
    match result {
        Ok(_) => {
            let _ = progress_tx.send((task_id, ProgressEvent::Completed)).await;
        }
        Err(e) => {
            let _ = progress_tx.send((task_id, ProgressEvent::Error(e.to_string()))).await;
        }
    }
}

pub async fn delete_item_task(
    task_id: Uuid,
    path: PathBuf,
    progress_tx: mpsc::Sender<(Uuid, ProgressEvent)>,
) {
    let result = if path.is_dir() {
        fs::remove_dir_all(&path).await
    } else {
        fs::remove_file(&path).await
    };

    match result {
        Ok(_) => {
            let _ = progress_tx.send((task_id, ProgressEvent::Completed)).await;
        }
        Err(e) => {
            let _ = progress_tx.send((task_id, ProgressEvent::Error(e.to_string()))).await;
        }
    }
}

pub async fn move_item_task(
    task_id: Uuid,
    src: PathBuf,
    dest: PathBuf,
    progress_tx: mpsc::Sender<(Uuid, ProgressEvent)>,
) {
    let result = fs::rename(&src, &dest).await;
    match result {
        Ok(_) => {
            let _ = progress_tx.send((task_id, ProgressEvent::Completed)).await;
        }
        Err(e) => {
            let _ = progress_tx.send((task_id, ProgressEvent::Error(e.to_string()))).await;
        }
    }
}

use tokio::io::{AsyncBufReadExt, BufReader};

const PREVIEW_LINE_COUNT: usize = 100;

pub async fn load_text_preview(path: PathBuf) -> Result<String, String> {
    let file = tokio::fs::File::open(path).await.map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let mut content = String::new();
    for _ in 0..PREVIEW_LINE_COUNT {
        match lines.next_line().await {
            Ok(Some(line)) => {
                content.push_str(&line);
                content.push('\n');
            }
            Ok(None) => break, // End of file
            Err(e) => return Err(e.to_string()),
        }
    }

    Ok(content)
}

pub async fn unmount_task(
    task_id: Uuid,
    path: PathBuf,
    progress_tx: mpsc::Sender<(Uuid, ProgressEvent)>,
) {
    let result = tokio::process::Command::new("umount")
        .arg(path)
        .output()
        .await;

    match result {
        Ok(output) => {
            if output.status.success() {
                let _ = progress_tx.send((task_id, ProgressEvent::Completed)).await;
            } else {
                let error_message = String::from_utf8_lossy(&output.stderr).to_string();
                let _ = progress_tx.send((task_id, ProgressEvent::Error(error_message))).await;
            }
        }
        Err(e) => {
            let _ = progress_tx.send((task_id, ProgressEvent::Error(e.to_string()))).await;
        }
    }
}

pub async fn archive_task(
    task_id: Uuid,
    paths: Vec<PathBuf>,
    dest: PathBuf,
    format: String,
    progress_tx: mpsc::Sender<(Uuid, ProgressEvent)>,
) {
    let result = match format.as_str() {
        "zip" => create_zip_archive(&paths, &dest).await,
        "tar" => create_tar_archive(&paths, &dest, false).await,
        "tar.gz" => create_tar_archive(&paths, &dest, true).await,
        _ => Err(format!("Unsupported archive format: {}", format)),
    };

    match result {
        Ok(_) => {
            let _ = progress_tx.send((task_id, ProgressEvent::Completed)).await;
        }
        Err(e) => {
            let _ = progress_tx.send((task_id, ProgressEvent::Error(e))).await;
        }
    }
}

async fn create_zip_archive(paths: &[PathBuf], dest: &PathBuf) -> Result<(), String> {
    use std::io::BufWriter;
    
    let file = File::create(dest).map_err(|e| e.to_string())?;
    let mut zip = ZipWriter::new(BufWriter::new(file));
    
    for path in paths {
        if path.is_dir() {
            add_dir_to_zip(&mut zip, path, path).map_err(|e| e.to_string())?;
        } else {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            zip.start_file(file_name, FileOptions::default())
                .map_err(|e| e.to_string())?;
            let content = std::fs::read(path).map_err(|e| e.to_string())?;
            zip.write_all(&content).map_err(|e| e.to_string())?;
        }
    }
    
    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

fn add_dir_to_zip<W: Write + std::io::Seek>(
    zip: &mut ZipWriter<W>,
    base_path: &PathBuf,
    dir_path: &PathBuf,
) -> zip::result::ZipResult<()> {
    use zip::write::FileOptions;
    
    for entry in std::fs::read_dir(dir_path).map_err(|_| zip::result::ZipError::Io(std::io::Error::new(std::io::ErrorKind::Other, "Failed to read directory")))? {
        let entry = entry.map_err(|_| zip::result::ZipError::Io(std::io::Error::new(std::io::ErrorKind::Other, "Failed to read directory entry")))?;
        let path = entry.path();
        
        let relative_path = path.strip_prefix(base_path).map_err(|_| zip::result::ZipError::Io(std::io::Error::new(std::io::ErrorKind::Other, "Failed to create relative path")))?;
        let path_str = relative_path.to_str().ok_or(zip::result::ZipError::Io(std::io::Error::new(std::io::ErrorKind::Other, "Invalid path")))?;
        
        if path.is_dir() {
            zip.add_directory(path_str, FileOptions::default())?;
            add_dir_to_zip(zip, base_path, &path)?;
        } else {
            zip.start_file(path_str, FileOptions::default())?;
            let content = std::fs::read(&path).map_err(|_| zip::result::ZipError::Io(std::io::Error::new(std::io::ErrorKind::Other, "Failed to read file")))?;
            zip.write_all(&content).map_err(|_| zip::result::ZipError::Io(std::io::Error::new(std::io::ErrorKind::Other, "Failed to write file")))?;
        }
    }
    Ok(())
}

async fn create_tar_archive(paths: &[PathBuf], dest: &PathBuf, compressed: bool) -> Result<(), String> {
    let file = File::create(dest).map_err(|e| e.to_string())?;
    
    if compressed {
        let enc = GzEncoder::new(file, Compression::default());
        let mut builder = Builder::new(enc);
        
        for path in paths {
            if path.is_dir() {
                builder.append_dir_all(path.file_name().unwrap(), path).map_err(|e| e.to_string())?;
            } else {
                builder.append_path(path).map_err(|e| e.to_string())?;
            }
        }
        
        builder.into_inner()
            .map_err(|e| e.to_string())?
            .finish()
            .map_err(|e| e.to_string())?;
    } else {
        let mut builder = Builder::new(file);
        
        for path in paths {
            if path.is_dir() {
                builder.append_dir_all(path.file_name().unwrap(), path).map_err(|e| e.to_string())?;
            } else {
                builder.append_path(path).map_err(|e| e.to_string())?;
            }
        }
        
        builder.into_inner().map_err(|e| e.to_string())?;
    }
    
    Ok(())
}