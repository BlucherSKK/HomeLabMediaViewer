use crate::hlmv::lang::translate;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// @description Преобразует относительный путь в абсолютный относительно исполняемого файла.
/// @param rel_path Относительный путь.
pub fn abspath(rel_path: &str) -> PathBuf {
    let mut path = env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    path.pop();
    path.push(rel_path);
    path
}

/// @description Преобразует относительный путь в статическую строку с абсолютным путем.
/// @param rel_path Относительный путь.
pub fn abspath_str(rel_path: &str) -> &'static str {
    let mut path = env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    path.pop();
    path.push(rel_path);
    let path_str = path.to_string_lossy().into_owned();
    Box::leak(path_str.into_boxed_str())
}

/// @description Проверяет, является ли указанная директория пустой.
/// @param path Путь к проверяемой директории.
pub fn dir_is_empty(path: &Path) -> io::Result<bool> {
    let mut entries = fs::read_dir(path)?;
    Ok(entries.next().is_none())
}

pub fn is_spesial_file(path: &Path) -> bool {
    let forbidden = &['!', '@', '#', '$'];
    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
        return file_name.chars().any(|c| forbidden.contains(&c));
    }
    false
}
