// SPDX-License-Identifier: MPL-2.0
//! File system operations for My Language
//!
//! Provides file and directory manipulation functions.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

/// Read entire file as bytes
pub fn read_bytes(path: &str) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|e| format!("Failed to read file: {}", e))
}

/// Read entire file as string
pub fn read_string(path: &str) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))
}

/// Write bytes to file (creates or overwrites)
pub fn write_bytes(path: &str, data: &[u8]) -> Result<(), String> {
    fs::write(path, data).map_err(|e| format!("Failed to write file: {}", e))
}

/// Write string to file (creates or overwrites)
pub fn write_string(path: &str, data: &str) -> Result<(), String> {
    fs::write(path, data).map_err(|e| format!("Failed to write file: {}", e))
}

/// Append bytes to file
pub fn append_bytes(path: &str, data: &[u8]) -> Result<(), String> {
    OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
        .and_then(|mut file| file.write_all(data))
        .map_err(|e| format!("Failed to append to file: {}", e))
}

/// Append string to file
pub fn append_string(path: &str, data: &str) -> Result<(), String> {
    append_bytes(path, data.as_bytes())
}

/// Check if file exists
pub fn exists(path: &str) -> bool {
    Path::new(path).exists()
}

/// Check if path is a file
pub fn is_file(path: &str) -> bool {
    Path::new(path).is_file()
}

/// Check if path is a directory
pub fn is_dir(path: &str) -> bool {
    Path::new(path).is_dir()
}

/// Get file size in bytes
pub fn file_size(path: &str) -> Result<u64, String> {
    fs::metadata(path)
        .map(|m| m.len())
        .map_err(|e| format!("Failed to get file size: {}", e))
}

/// Create a directory
pub fn create_dir(path: &str) -> Result<(), String> {
    fs::create_dir(path).map_err(|e| format!("Failed to create directory: {}", e))
}

/// Create a directory and all parent directories
pub fn create_dir_all(path: &str) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|e| format!("Failed to create directories: {}", e))
}

/// Remove a file
pub fn remove_file(path: &str) -> Result<(), String> {
    fs::remove_file(path).map_err(|e| format!("Failed to remove file: {}", e))
}

/// Remove an empty directory
pub fn remove_dir(path: &str) -> Result<(), String> {
    fs::remove_dir(path).map_err(|e| format!("Failed to remove directory: {}", e))
}

/// Remove a directory and all its contents
pub fn remove_dir_all(path: &str) -> Result<(), String> {
    fs::remove_dir_all(path).map_err(|e| format!("Failed to remove directory: {}", e))
}

/// Copy a file
pub fn copy_file(from: &str, to: &str) -> Result<u64, String> {
    fs::copy(from, to).map_err(|e| format!("Failed to copy file: {}", e))
}

/// Move/rename a file or directory
pub fn rename(from: &str, to: &str) -> Result<(), String> {
    fs::rename(from, to).map_err(|e| format!("Failed to rename: {}", e))
}

/// List directory contents
pub fn list_dir(path: &str) -> Result<Vec<String>, String> {
    fs::read_dir(path)
        .map_err(|e| format!("Failed to read directory: {}", e))?
        .map(|entry| {
            entry
                .map(|e| e.path().to_string_lossy().to_string())
                .map_err(|e| format!("Failed to read entry: {}", e))
        })
        .collect()
}

/// Get absolute path
pub fn absolute_path(path: &str) -> Result<String, String> {
    fs::canonicalize(path)
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| format!("Failed to get absolute path: {}", e))
}

/// Get current working directory
pub fn current_dir() -> Result<String, String> {
    std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| format!("Failed to get current directory: {}", e))
}

/// Change current working directory
pub fn change_dir(path: &str) -> Result<(), String> {
    std::env::set_current_dir(path)
        .map_err(|e| format!("Failed to change directory: {}", e))
}

/// Get file extension
pub fn extension(path: &str) -> Option<String> {
    Path::new(path)
        .extension()
        .map(|s| s.to_string_lossy().to_string())
}

/// Get file name without path
pub fn file_name(path: &str) -> Option<String> {
    Path::new(path)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
}

/// Get file stem (name without extension)
pub fn file_stem(path: &str) -> Option<String> {
    Path::new(path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
}

/// Get parent directory
pub fn parent_dir(path: &str) -> Option<String> {
    Path::new(path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
}

/// Join path components
pub fn join_path(base: &str, component: &str) -> String {
    PathBuf::from(base)
        .join(component)
        .to_string_lossy()
        .to_string()
}

/// Create a temporary file
pub fn temp_file() -> Result<(String, File), String> {
    let temp_dir = std::env::temp_dir();
    let file_name = format!("my-lang-{}.tmp", std::process::id());
    let path = temp_dir.join(file_name);
    let file = File::create(&path)
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    Ok((path.to_string_lossy().to_string(), file))
}

/// File metadata
pub struct FileInfo {
    pub path: String,
    pub size: u64,
    pub is_file: bool,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub readonly: bool,
}

/// Get file metadata
pub fn file_info(path: &str) -> Result<FileInfo, String> {
    let metadata = fs::metadata(path)
        .map_err(|e| format!("Failed to get file info: {}", e))?;

    Ok(FileInfo {
        path: path.to_string(),
        size: metadata.len(),
        is_file: metadata.is_file(),
        is_dir: metadata.is_dir(),
        is_symlink: metadata.is_symlink(),
        readonly: metadata.permissions().readonly(),
    })
}

/// Walk directory recursively
pub fn walk_dir(path: &str) -> Result<Vec<String>, String> {
    let mut result = Vec::new();
    walk_dir_impl(Path::new(path), &mut result)?;
    Ok(result)
}

fn walk_dir_impl(path: &Path, result: &mut Vec<String>) -> Result<(), String> {
    if path.is_dir() {
        for entry in fs::read_dir(path)
            .map_err(|e| format!("Failed to read directory: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let entry_path = entry.path();
            result.push(entry_path.to_string_lossy().to_string());
            if entry_path.is_dir() {
                walk_dir_impl(&entry_path, result)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_operations() {
        assert_eq!(extension("file.txt"), Some("txt".to_string()));
        assert_eq!(file_name("/path/to/file.txt"), Some("file.txt".to_string()));
        assert_eq!(file_stem("/path/to/file.txt"), Some("file".to_string()));
    }

    #[test]
    fn test_join_path() {
        let joined = join_path("/home/user", "file.txt");
        assert!(joined.contains("file.txt"));
    }
}
