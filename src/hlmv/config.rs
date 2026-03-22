use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::State;
use std::sync::Arc;
use crate::hlmv::db::{MediaDb};

// 1. Обновляем структуру, чтобы она соответствовала параметрам функции базы данных
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct PlayerConfig {
    pub path: String,      // Путь к файлу теперь обязателен
    pub volume: i32,       // Изменили на i32 для соответствия БД
    pub current_time: i64, // Изменили на i64 для соответствия БД
}

// --- Обработчик Rocket ---
#[post("/uploadconf", format = "json", data = "<config>")]
pub fn upload_handler(
    config: Json<PlayerConfig>,
    db: &State<Arc<MediaDb>>, // Получаем доступ к БД через State
) -> &'static str {
    let cfg = config.into_inner();
    let db_cloned = Arc::clone(db.inner());

    println!("Received config for: {}", cfg.path);

    // Поскольку запись в БД синхронная и тяжелая,
    // используем spawn_blocking (это правильнее для синхронного I/O в Tokio)
    rocket::tokio::task::spawn_blocking(move || {
        if let Err(e) = db_cloned.upload(&cfg.path, cfg.current_time, cfg.volume) {
            eprintln!("Database error: {}", e);
        } else {
            println!("Successfully saved config to Database!");
        }
    });

    "Accepted"
}
