use rusqlite::{Connection, OptionalExtension, Result, params};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct FileRecord {
    pub id: i32,
    pub file_path: PathBuf,
    pub timestamp: i64,
    pub volume: i32,
    pub last_accessed: i64,
}

#[derive(Clone)]
pub struct MediaDb {
    conn: Arc<Mutex<Connection>>,
}

impl MediaDb {
    /*
     * @description Открывает соединение с базой данных и инициализирует таблицу истории воспроизведения.
     * @param path Путь к файлу базы данных SQLite.
     */
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS playback_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_path TEXT NOT NULL UNIQUE,
                timestamp INTEGER NOT NULL,
                volume INTEGER NOT NULL CHECK(volume >= 1 AND volume <= 100),
                     last_accessed INTEGER NOT NULL DEFAULT (strftime('%s','now'))
        )",
        [],
        )?;
        Ok(MediaDb {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn current_time() -> i64 {
        SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
    }

    /*
     * @description Возвращает ID записи в базе данных по заданному пути к файлу.
     * @param path Путь к файлу для поиска.
     */
    pub fn get_id_by_path<P: AsRef<Path>>(&self, path: P) -> Result<Option<i32>> {
        let path_str = path.as_ref().to_string_lossy();
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id FROM playback_history WHERE file_path = ?1",
            params![path_str],
            |row| row.get(0),
        ).optional()
    }

    pub fn get_path_by_id(&self, id: i32) -> Result<Option<PathBuf>> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT file_path FROM playback_history WHERE id = ?1",
            params![id],
            |row| {
                let path_str: String = row.get(0)?;
                Ok(PathBuf::from(path_str))
            },
        ).optional()
    }

    /*
     * @description Получает данные о воспроизведении и обновляет время последнего доступа к файлу.
     * @param path Путь к медиафайлу.
     */
    pub fn get_playback<P: AsRef<Path>>(&self, path: P) -> Result<Option<FileRecord>> {
        let path_str = path.as_ref().to_string_lossy();
        let now = Self::current_time();
        let conn = self.conn.lock().unwrap();
        let updated = conn.execute(
            "UPDATE playback_history SET last_accessed = ?1 WHERE file_path = ?2",
            params![now, path_str],
        )?;
        if updated == 0 {
            return Ok(None);
        }
        conn.query_row(
            "SELECT id, file_path, timestamp, volume, last_accessed FROM playback_history WHERE file_path = ?1",
            params![path_str],
            |row| {
                Ok(FileRecord {
                    id: row.get(0)?,
                   file_path: PathBuf::from(row.get::<_, String>(1)?),
                   timestamp: row.get(2)?,
                   volume: row.get(3)?,
                   last_accessed: row.get(4)?,
                })
            },
        ).optional()
    }

    /*
     * @description Сохраняет или обновляет информацию о прогрессе воспроизведения файла.
     * @param path Путь к файлу.
     * @param ts Текущая временная метка (прогресс) в секундах.
     * @param vol Уровень громкости.
     */
    pub fn upload<P: AsRef<Path>>(&self, path: P, ts: i64, vol: i32) -> Result<i32> {
        let path_str = path.as_ref().to_string_lossy();
        let now = Self::current_time();
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "INSERT INTO playback_history (file_path, timestamp, volume, last_accessed)
        VALUES (?1, ?2, ?3, ?4)
        ON CONFLICT(file_path) DO UPDATE SET
        timestamp = excluded.timestamp,
        volume = excluded.volume,
        last_accessed = excluded.last_accessed
        RETURNING id",
        params![path_str, ts, vol, now],
        |row| row.get(0),
        )
    }

    /*
     * @description Обновляет информацию о прогрессе воспроизведения файла по его ID.
     * @param id Идентификатор записи в БД.
     * @param ts Текущая временная метка (прогресс) в секундах.
     * @param vol Уровень громкости.
     */
    pub fn upload_by_id(&self, id: i32, ts: i64, vol: i32) -> Result<()> {
        let now = Self::current_time();
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "UPDATE playback_history
            SET timestamp = ?1,
            volume = ?2,
            last_accessed = ?3
            WHERE id = ?4"
        )?;
        let updated_rows = stmt.execute(params![ts, vol, now, id])?;
        if updated_rows == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        Ok(())
    }

    /*
     * @description Возвращает список последних воспроизведенных файлов.
     * @param n Количество записей для выборки.
     */
    pub fn get_recent(&self, n: usize) -> Result<Vec<FileRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, file_path, timestamp, volume, last_accessed
            FROM playback_history ORDER BY last_accessed DESC LIMIT ?1",
        )?;
        let records_iter = stmt.query_map(params![n as i64], |row| {
            Ok(FileRecord {
                id: row.get(0)?,
               file_path: PathBuf::from(row.get::<_, String>(1)?),
               timestamp: row.get(2)?,
               volume: row.get(3)?,
               last_accessed: row.get(4)?,
            })
        })?;
        records_iter.collect()
    }

    /*
     * @description Получает полную информацию о файле по его ID и обновляет время доступа.
     * @param id Идентификатор записи.
     */
    pub fn get_info_by_id(&self, id: i32) -> Result<Option<FileRecord>> {
        let now = Self::current_time();
        let conn = self.conn.lock().unwrap();
        let updated = conn.execute(
            "UPDATE playback_history SET last_accessed = ?1 WHERE id = ?2",
            params![now, id],
        )?;
        if updated == 0 {
            return Ok(None);
        }
        conn.query_row(
            "SELECT id, file_path, timestamp, volume, last_accessed
            FROM playback_history WHERE id = ?1",
            params![id],
            |row| {
                Ok(FileRecord {
                    id: row.get(0)?,
                   file_path: PathBuf::from(row.get::<_, String>(1)?),
                   timestamp: row.get(2)?,
                   volume: row.get(3)?,
                   last_accessed: row.get(4)?,
                })
            },
        ).optional()
    }
}
