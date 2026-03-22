mod hlmv;
use rusqlite::{params, Connection, Result};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct FileRecord {
    pub id: i32,
    pub file_path: String,
    pub timestamp: i64,
    pub volume: i32,
    pub last_accessed: i64,
}

pub struct MediaDb {
    conn: Connection,
}

impl MediaDb {
    pub fn open(path: &str) -> Result<Self> {
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

        Ok(MediaDb { conn })
    }

    fn current_time() -> i64 {
        SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
    }

    /// Обновляет время доступа и возвращает полную структуру записи
    pub fn get_playback(&self, path: &str) -> Result<Option<FileRecord>> {
        let now = Self::current_time();

        // 1. Обновляем время последнего обращения
        let updated = self.conn.execute(
            "UPDATE playback_history SET last_accessed = ?1 WHERE file_path = ?2",
            params![now, path],
        )?;

        // Если записей обновлено 0, значит такого пути нет в базе
        if updated == 0 {
            return Ok(None);
        }

        // 2. Выбираем все данные по этому пути
        let mut stmt = self.conn.prepare(
            "SELECT id, file_path, timestamp, volume, last_accessed
            FROM playback_history WHERE file_path = ?1"
        )?;

        let mut rows = stmt.query_map(params![path], |row| {
            Ok(FileRecord {
                id: row.get(0)?,
               file_path: row.get(1)?,
               timestamp: row.get(2)?,
               volume: row.get(3)?,
               last_accessed: row.get(4)?,
            })
        })?;

        if let Some(record) = rows.next() {
            return Ok(Some(record?));
        }

        Ok(None)
    }

    pub fn upload(&self, path: &str, ts: i64, vol: i32) -> Result<()> {
        let now = Self::current_time();

        // Используем ON CONFLICT для автоматического переключения между INSERT и UPDATE
        self.conn.execute(
            "INSERT INTO playback_history (file_path, timestamp, volume, last_accessed)
        VALUES (?1, ?2, ?3, ?4)
        ON CONFLICT(file_path) DO UPDATE SET
        timestamp = excluded.timestamp,
        volume = excluded.volume,
        last_accessed = excluded.last_accessed",
        params![path, ts, vol, now],
        )?;

        Ok(())
    }

    pub fn get_recent(&self, n: usize) -> Result<Vec<FileRecord>> {
        // Подготавливаем запрос на выборку всех полей
        let mut stmt = self.conn.prepare(
            "SELECT id, file_path, timestamp, volume, last_accessed
            FROM playback_history
            ORDER BY last_accessed DESC
            LIMIT ?1"
        )?;

        // Маппим строки БД в структуры Rust
        let records_iter = stmt.query_map(params![n as i64], |row| {
            Ok(FileRecord {
                id: row.get(0)?,
               file_path: row.get(1)?,
               timestamp: row.get(2)?,
               volume: row.get(3)?,
               last_accessed: row.get(4)?,
            })
        })?;

        // Собираем итератор в вектор, обрабатывая возможные ошибки каждой строки
        let mut records = Vec::new();
        for record in records_iter {
            records.push(record?);
        }

        Ok(records)
    }
}
