use std::env;
use std::path::PathBuf;
use std::fs;
use std::path::Path;
use std::io;

pub fn abspath(rel_path: &str) -> PathBuf {
    let mut path = env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    path.pop();
    path.push(rel_path);
    path
}

pub fn abspath_str(rel_path: &str) -> &'static str {
    let mut path = env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    path.pop();
    path.push(rel_path);
    let path_str = path.to_string_lossy().into_owned();
    Box::leak(path_str.into_boxed_str())
}



pub fn dir_is_empty(path: &Path) -> io::Result<bool> {
    // Читаем содержимое директории
    let mut entries = fs::read_dir(path)?;

    // Если next() вернул None, значит директория пуста
    Ok(entries.next().is_none())
}
