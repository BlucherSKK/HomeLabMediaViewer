use std::env;
use std::path::PathBuf;

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
