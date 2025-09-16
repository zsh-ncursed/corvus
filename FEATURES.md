# Corvus Features

## Core Features

### Three-Column Layout
Corvus features a three-column interface designed for efficient file management:
- **Left Pane:** Quick access to XDG user folders, bookmarks, and mounted devices.
- **Middle Pane:** Main file list with support for sorting and filtering.
- **Right Pane:** Asynchronous preview for text files, images, and PDF documents.

### Asynchronous Operations
All file operations (copy, move, delete) are handled asynchronously in the background, ensuring the UI remains responsive even during heavy file operations.

### Tabbed Interface
Manage multiple directories simultaneously with a tabbed interface. Switch between tabs using keyboard shortcuts or mouse clicks.

### Search Functionality
Search for files by name or content with real-time filtering. The search dialog provides instant results as you type.

### Session Persistence
Corvus automatically saves and restores your session state between application launches, including open tabs, directory paths, and bookmarks.

## Advanced Features

### Asynchronous Previews
Previews for images (PNG, JPEG, etc.) and PDF documents are rendered asynchronously with progressive rendering:
- A low-resolution thumbnail is shown almost instantly
- The full-resolution version is then rendered and replaces the thumbnail
- Currently supports the Kitty graphics protocol for image previews

### Extensible Architecture
Corvus is built with extensibility in mind:
- Modular crate structure for easy maintenance and development
- Plugin system (work in progress) for adding new functionality

### Highly Configurable
Customize Corvus to your liking:
- Keybindings can be customized via `config.toml`
- Themes and color schemes can be configured
- Bookmarks can be defined in the configuration file

### Cross-Platform
Built with Rust, Corvus runs on multiple platforms including Linux, macOS, and Windows.
