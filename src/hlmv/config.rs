use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::State;
// Нам больше не нужно явно импортировать Arc в обработчике,
// так как MediaDb сама управляет своим состоянием.
use crate::hlmv::db::MediaDb;
use crate::hlmv::lang::{LOCALEMSG, translate};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct PlayerConfig {
    pub path: String,
    pub volume: i32,
    pub current_time: i64,
}

// --- Обработчик Rocket ---
#[post("/uploadconf", format = "json", data = "<config>")]
pub fn upload_handler(
    config: Json<PlayerConfig>,
    db: &State<MediaDb>, // Просто передаем MediaDb, она уже потокобезопасна
) -> &'static str {
    let cfg = config.into_inner();

    // Клонируем MediaDb. Это дешево, так как внутри просто копируется Arc.
    let db_cloned = db.inner().clone();

    println!("Received config for: {}", cfg.path);

    // Используем spawn_blocking, так как внутри MediaDb.upload()
    // происходит захват Mutex и блокирующее I/O SQLite.
    rocket::tokio::task::spawn_blocking(move || {
        if let Err(e) = db_cloned.upload(&cfg.path, cfg.current_time, cfg.volume) {
            eprintln!("{} {}",translate(LOCALEMSG::DataBaseEr) ,e);
        } else {
            println!("Successfully saved config to Database!");
        }
    });

    "Accepted"
}
