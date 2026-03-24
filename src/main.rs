#[macro_use] extern crate rocket;

use std::path::{PathBuf, Path};
use std::io;
use rocket::response::content::{RawCss, RawHtml};
use rocket::fs::NamedFile;
use rocket::State;
use crate::hlmv::db::MediaDb;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::http::Header;
use rocket::{Request, Response};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::fairing::AdHoc;

mod hlmv;
use crate::hlmv::thumb::init_default_icon;
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

#[get("/style")]
pub fn get_style() -> RawCss<&'static str>{
    RawCss(include_str!("web/style.css"))
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

#[derive(Serialize, Deserialize)]
struct MediaInfo {
    timestamp: f64,
    volume: u8,
}

// 1. Стриминг видео
// NamedFile в Rocket автоматически поддерживает Range-запросы (перемотку)
#[get("/stream/<path..>")]
async fn stream(path: PathBuf) -> Option<NamedFile> {
    let base_path = abspath("media"); // Папка с видео
    NamedFile::open(base_path.join(path)).await.ok()
}

// 2. Получение инфо (заглушка)
#[get("/info/<_path..>")]
fn get_info(_path: PathBuf) -> Json<MediaInfo> {
    Json(MediaInfo {
        timestamp: 10.5, // Начнем с 10-й секунды для теста
         volume: 80,
    })
}

// 3. Сохранение инфо
#[post("/info-upload/<path..>", data = "<info>")]
fn upload_info(path: PathBuf, info: Json<MediaInfo>) {
    println!("Saved for {:?}: time {}, vol {}", path, info.timestamp, info.volume);
    // Тут можно сохранить в БД или JSON файл
}

// 4. Главная страница плеера
#[get("/live/<path..>")]
async fn player(path: PathBuf) -> Option<rocket::response::content::RawHtml<String>> {
    let html = include_str!("./web/index.html");
    // Вставляем путь в JS для удобства (простая замена в шаблоне)
    let processed_html = html.replace("{{PATH}}", path.to_str().unwrap_or(""));
    Some(rocket::response::content::RawHtml(processed_html))
}

/*
 * @description Конфигурирует и запускает сервер Rocket, инициализируя компоненты приложения.
 */
#[launch]
fn rocket() -> _ {
    let app = hlmv::App::init().expect(translate(hlmv::lang::LOCALEMSG::InitEr));
    init_default_icon();
    rocket::build()
    .attach(AdHoc::on_response("CORS & Timing", |_, res| Box::pin(async move {
        res.set_header(Header::new("Timing-Allow-Origin", "*"));
        res.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        res.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, OPTIONS"));
        res.set_header(Header::new("Access-Control-Allow-Headers", "Content-Type"));
    })))
    .manage(app.db)
    .mount("/", routes![
        get_style,
        browser_root,
        browser_dir,
        serve_media,
        serve_cache,
        stream, get_info, upload_info, player
    ])
    .mount("/api", routes![
        upload_handler
    ])
}
