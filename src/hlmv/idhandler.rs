use crate::hlmv::{
    db::MediaDb,
    models::MediaInfo,
    thumb::get_thumb,
};
use rocket::fs::NamedFile;
use rocket::response::content::RawHtml;
use rocket::serde::json::Json;
use rocket::State;

/// @description Отдает медиафайл пользователю по его идентификатору в базе данных.
/// @param id Идентификатор файла.
/// @param db Состояние базы данных.
#[get("/media-files-byid/<id>")]
pub async fn serve_media_byid(id: i32, db: &State<MediaDb>) -> Option<NamedFile> {
    let path = db.get_path_by_id(id).ok().flatten()?;
    NamedFile::open(path).await.ok()
}

/// @description Обеспечивает стриминг медиаконтента по идентификатору файла.
/// @param id Идентификатор файла.
/// @param db Состояние базы данных.
#[get("/stream-byid/<id>")]
pub async fn stream_byid(id: i32, db: &State<MediaDb>) -> Option<NamedFile> {
    let path = db.get_path_by_id(id).ok().flatten()?;
    NamedFile::open(path).await.ok()
}

/// @description Возвращает метаданные и информацию о воспроизведении файла по его ID.
/// @param id Идентификатор файла.
/// @param db Состояние базы данных.
#[get("/info-byid/<id>")]
pub fn get_info_byid(id: i32, db: &State<MediaDb>) -> Option<Json<MediaInfo>> {
    db.get_info_by_id(id).ok().flatten().map(Json)
}

/// @description Обновляет информацию о прогрессе воспроизведения и громкости для указанного ID.
/// @param id Идентификатор файла.
/// @param info JSON-объект с данными о позиции и громкости.
/// @param db Состояние базы данных.
#[post("/info-upload-byid/<id>", data = "<info>")]
pub fn upload_info_byid(id: i32, info: Json<MediaInfo>, db: &State<MediaDb>) {
    if let Err(e) = db.upload_by_id(id, info.timestamp, info.volume) {
        eprintln!("Ошибка при сохранении в БД для ID {}: {}", id, e);
    }
}

/// @description Генерирует HTML-страницу плеера, подставляя в неё ID медиафайла.
/// @param id Идентификатор файла для воспроизведения.
/// @param _db Состояние базы данных (используется для проверки существования).
#[get("/live-byid/<id>")]
pub async fn player_byid(id: i32, _db: &State<MediaDb>) -> Option<RawHtml<String>> {
    let html = include_str!("../web/index.html");
    let processed_html = html.replace("{{MEDIA_ID}}", &id.to_string());
    Some(RawHtml(processed_html))
}
