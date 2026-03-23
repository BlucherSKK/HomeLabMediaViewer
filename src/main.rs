#[macro_use] extern crate rocket;

use std::path::PathBuf;
use std::io;
use rocket::response::content::RawHtml;
use rocket::fs::NamedFile;
use rocket::State;
use crate::hlmv::db::MediaDb;

mod hlmv;
use crate::hlmv::{
    config::upload_handler,
    lang::translate,
    browser::render_browser,
    fs::abspath
};

/*
 * @description Отображает корневую директорию медиа-браузера.
 * @param db Состояние базы данных для генерации миниатюр.
 */
#[get("/")]
pub fn browser_root(db: &State<MediaDb>) -> RawHtml<String> {
    render_browser(PathBuf::new(), db)
}

/*
 * @description Отображает содержимое указанной директории в медиа-браузере.
 * @param path Относительный путь к папке.
 * @param db Состояние базы данных для генерации миниатюр.
 */
#[get("/browser/<path..>")]
pub fn browser_dir(path: PathBuf, db: &State<MediaDb>) -> RawHtml<String> {
    render_browser(path, db)
}

/*
 * @description Предоставляет доступ к оригинальным медиа-файлам по их пути.
 * @param file Относительный путь к файлу в директории media.
 */
#[get("/media-files/<file..>")]
pub async fn serve_media(file: PathBuf) -> io::Result<NamedFile> {
    let base = abspath("media");
    let full_path = base.join(file);
    NamedFile::open(full_path).await
}

/*
 * @description Отдает файлы миниатюр из кэша приложения.
 * @param file Имя файла миниатюры.
 */
#[get("/cache/<file..>")]
pub async fn serve_cache(file: PathBuf) -> Option<NamedFile> {
    let base = abspath("cache");
    let full_path = base.join(file);
    NamedFile::open(full_path).await.ok()
}

/*
 * @description Конфигурирует и запускает сервер Rocket, инициализируя компоненты приложения.
 */
#[launch]
fn rocket() -> _ {
    let app = hlmv::App::init().expect(translate(hlmv::lang::LOCALEMSG::InitEr));
    rocket::build()
    .manage(app.db)
    .mount("/", routes![
        browser_root,
        browser_dir,
        serve_media,
        serve_cache
    ])
    .mount("/api", routes![
        upload_handler
    ])
}
