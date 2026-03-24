pub mod db;
pub mod fs;
pub mod config;
pub mod lang;
pub mod browser;
pub mod thumb;
pub mod idhandler;

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
    /*
     * @description Инициализирует структуру приложения, создает необходимые директории и подключает базу данных.
     * @return Возвращает экземпляр App или ошибку ввода-вывода.
     */
    pub fn init() -> std::io::Result<Self> {
        let _exe_dir = std::env::current_exe().expect(translate(LOCALEMSG::ElfDirUnfound));
        let mut ensure_dir = |rel_path: &str| -> std::io::Result<&'static str> {
            let full_path = crate::hlmv::fs::abspath(rel_path);
            if !full_path.exists() {
                std::fs::create_dir_all(&full_path)?;
            }
            let path_str = full_path.to_string_lossy().into_owned();
            Ok(Box::leak(path_str.into_boxed_str()))
        };
        let config_dir = ensure_dir("config")?;
        let cache_dir = ensure_dir("cache")?;
        let main_media = ensure_dir("media")?;
        let db_path = format!("{}/db.splite", config_dir);
        let db = crate::hlmv::db::MediaDb::open(&db_path)
        .expect(translate(LOCALEMSG::DataBaseInitFail));
        let mut media_folders = vec![main_media];
        media_folders.sort();
        Ok(App {
            config_dir,
            cache_dir,
            media_folders,
            db,
        })
    }
}
