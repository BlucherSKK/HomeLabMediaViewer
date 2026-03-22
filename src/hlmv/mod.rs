pub mod db;
pub mod fs;
pub mod config;
pub mod lang;

use std::env;
use std::path::PathBuf;

use crate::hlmv::lang::{LOCALEMSG, translate};

pub struct App<'a> {
    pub config_dir: &'a str,
    pub cache_dir: &'a str,
    pub media_folders: Vec<&'a str>,
    pub db: crate::hlmv::db::MediaDb,
}

impl<'a> App<'a> {
    pub fn init() -> std::io::Result<Self> {
        let exe_dir = std::env::current_exe().expect(translate(LOCALEMSG::ElfDirUnfound));


        let db = crate::hlmv::db::MediaDb::open(crate::hlmv::fs::abspath_str("./config/db.splite"))
            .expect(translate(LOCALEMSG::DataBaseInitFail));

        // Вспомогательная функция для подготовки папок
        // Возвращает &'static str через leak, так как пути нужны на весь период работы App
        #[warn(unused_mut)]
        let mut ensure_dir = |rel_path: &str| -> std::io::Result<&'static str> {
            let full_path = crate::hlmv::fs::abspath(rel_path);
            if !full_path.exists() {
                std::fs::create_dir_all(&full_path)?;
            }
            // Превращаем String в &'static str
            let path_str = full_path.to_string_lossy().into_owned();
            Ok(Box::leak(path_str.into_boxed_str()))
        };

        let config_dir = ensure_dir("config")?;
        let cache_dir = ensure_dir("cache")?;

        let main_media = ensure_dir("media")?;
        let mut media_folders = vec![main_media];

        let entries = std::fs::read_dir(&exe_dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("media_") {
                        let abs_p_path = crate::hlmv::fs::abspath(name);
                        let abs_p_str = abs_p_path.to_string_lossy().into_owned();
                        let leaked_str: &'static str = Box::leak(abs_p_str.into_boxed_str());

                        if !media_folders.contains(&leaked_str) {
                            media_folders.push(leaked_str);
                        }
                    }
                }
            }
        }

        media_folders.sort();

        Ok(App {
            config_dir,
            cache_dir,
            media_folders,
            db,
        })
    }



}
