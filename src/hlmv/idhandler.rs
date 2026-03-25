use crate::hlmv::{
    db::{MediaDb, FileRecord},
    fs::abspath
};
use rocket::fs::NamedFile;
use rocket::response::content::RawHtml;
use rocket::serde::{json::Json, Serialize, Deserialize};
use rocket::State;
use std::path::PathBuf;

// Добавляем поддержку сериализации для вашей структуры
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct MediaInfo {
    pub timestamp: i64,
    pub volume: i32,
}

/// @description Отдает медиафайл пользователю по его идентификатору.
#[get("/media-files-byid/<id>")]
pub async fn serve_media_byid(id: i32, db: &State<MediaDb>) -> Option<NamedFile> {
    let path = db.get_path_by_id(id).ok().flatten()?;
    let base_dir = abspath("./media");
    let full_path = base_dir.join(&path);
    NamedFile::open(full_path).await.ok()
}

/// @description Обеспечивает стриминг медиаконтента (поддерживает Range Requests).
#[get("/stream-byid/<id>")]
pub async fn stream_byid(id: i32, db: &State<MediaDb>) -> Option<NamedFile> {
    let path = db.get_path_by_id(id).ok().flatten()?;
    let base_dir = abspath("./media");
    let full_path = base_dir.join(&path);
    NamedFile::open(full_path).await.ok()
}

/// @description Возвращает метаданные (прогресс и громкость) по ID.
#[get("/info-byid/<id>")]
pub fn get_info_byid(id: i32, db: &State<MediaDb>) -> Option<Json<MediaInfo>> {
    db.get_info_by_id(id).ok().flatten().map(FileRecord::into_media_info)
}

/// @description Обновляет информацию о прогрессе. Принимает JSON, соответствующий FileRecord.
#[post("/info-upload-byid/<id>", data = "<info>")]
pub fn upload_info_byid(id: i32, info: Json<MediaInfo>, db: &State<MediaDb>) {
    // Используем данные из присланной структуры
    if let Err(e) = db.upload_by_id(id, info.timestamp, info.volume) {
        eprintln!("Ошибка при сохранении в БД для ID {}: {}", id, e);
    }
}

/// @description Генерирует HTML-страницу плеера.
#[get("/live-byid/<id>")]
pub async fn player_byid(id: i32, _db: &State<MediaDb>) -> Option<RawHtml<String>> {
    // Вшиваем HTML из файла
    let html = include_str!("../web/index.html");
    // Подставляем ID в шаблон
    let processed_html = html
    .replace("{{MEDIA_ID}}", &id.to_string())
    .replace("{{STREAM_PATH}}", format!("/stream-byid/{}", id).as_str())
    .replace("{{MEDIA_PATH}}", format!("{}", id).as_str())
    .replace("{{INFO_PATH}}", format!("/info-byid/{}", id).as_str())
    .replace("{{INFO_UPLOAD_PATH}}", format!("/info-upload-byid/{}", id).as_str())
    ;
    Some(RawHtml(processed_html))
}
