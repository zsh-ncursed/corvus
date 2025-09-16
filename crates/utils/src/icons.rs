#[derive(Debug, Clone, Copy)]
pub enum IconColor {
    Blue,
    Rgb(u8, u8, u8),
    Magenta,
    Yellow,
    Cyan,
    Red,
    White,
    Gray,
}

pub fn get_icon_for_file(name: &str, is_dir: bool) -> &'static str {
    if is_dir {
        return ""; // Folder icon
    }
    match name.split('.').last() {
        Some("rs") => "",   // Rust
        Some("js") => "",   // JavaScript
        Some("html") => "", // HTML
        Some("css") => "",  // CSS
        Some("json") => "", // JSON
        Some("md") => "",   // Markdown
        Some("toml") => "", // TOML
        Some("lock") => "", // Lock
        Some("git") | Some("gitignore") => "", // Git
        // Audio
        Some("mp3") | Some("wav") | Some("flac") => "🎵",
        // Video
        Some("mp4") | Some("avi") | Some("mkv") | Some("mov") => "🎞",
        Some("zip") | Some("rar") | Some("7z") | Some("tar") | Some("gz") => "", // Archive
        Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("webp") | Some("ico") => "", // Image
        Some("pdf") => "",   // PDF
        Some("txt") => "",   // Text file
        _ => "",           // Default file
    }
}

pub fn get_color_for_file(name: &str, is_dir: bool) -> IconColor {
    if is_dir {
        return IconColor::Blue;
    }
    match name.split('.').last() {
        Some("rs") => IconColor::Rgb(220, 100, 80),   // Rust
        Some("js") => IconColor::Rgb(240, 220, 130),  // JavaScript
        Some("html") => IconColor::Rgb(227, 79, 38), // HTML
        Some("css") => IconColor::Rgb(38, 77, 228),  // CSS
        Some("json") => IconColor::Rgb(255, 204, 0), // JSON
        Some("md") => IconColor::White,   // Markdown
        Some("toml") => IconColor::Rgb(183, 113, 53), // TOML
        Some("lock") => IconColor::Rgb(200, 200, 200), // Lock
        Some("git") | Some("gitignore") => IconColor::Rgb(240, 80, 50), // Git
        // Audio
        Some("mp3") | Some("wav") | Some("flac") => IconColor::Magenta,
        // Video
        Some("mp4") | Some("avi") | Some("mkv") | Some("mov") => IconColor::Yellow,
        Some("zip") | Some("rar") | Some("7z") | Some("tar") | Some("gz") => IconColor::Rgb(172, 63, 49), // Archive
        Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("webp") | Some("ico") => IconColor::Cyan, // Image
        Some("pdf") => IconColor::Red,   // PDF
        Some("txt") => IconColor::White,   // Text file
        _ => IconColor::Gray,           // Default file
    }
}
