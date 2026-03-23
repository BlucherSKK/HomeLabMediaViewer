use crate::hlmv::db::MediaDb;
use crate::hlmv::fs::abspath;
use crate::hlmv::lang::{translate, LOCALEMSG};
use std::fs;
use std::path::Path;
use std::process::Command;

fn create_thumbnail(src: &Path, dst: &Path) -> std::io::Result<()> {
    let ext = src.extension()
    .and_then(|s| s.to_str())
    .unwrap_or_default()
    .to_lowercase();
    match ext.as_str() {
        "jpg" | "jpeg" | "png" | "webp" | "gif" => {
            let img = image::open(src)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            img.thumbnail(64, 64)
            .save_with_format(dst, image::ImageFormat::Jpeg)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            Ok(())
        }
        "mp4" | "mkv" | "webm" | "avi" | "mov" => {
            let output = Command::new("ffmpeg")
            .args([
                "-y", "-ss", "00:00:01", "-i", src.to_str().unwrap(),
                  "-frames:v", "1", "-vf", "scale=64:64:force_original_aspect_ratio=increase,crop=64:64",
                  dst.to_str().unwrap(),
            ])
            .output()?;
            if output.status.success() {
                Ok(())
            } else {
                let err = String::from_utf8_lossy(&output.stderr);
                Err(std::io::Error::new(std::io::ErrorKind::Other, format!("FFmpeg error: {err}")))
            }
        }
        _ => Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Unsupported format")),
    }
}

/*
 * @description Получает имя файла миниатюры. Если она отсутствует в кеше, запускает процесс генерации.
 * @param db Состояние базы данных для поиска или регистрации ID файла.
 * @param rel_path Путь к файлу, для которого нужна миниатюра.
 */
pub fn get_thumb(db: &MediaDb, rel_path: &Path) -> String {
    let id = db.get_id_by_path(rel_path)
    .ok()
    .flatten()
    .unwrap_or_else(|| {
        db.upload(rel_path, 0, 100).unwrap_or_else(|_| {
            println!("{}", translate(LOCALEMSG::DBgetIDFail));
            0
        })
    });
    let thumb_name = format!("{}.jpg", id);
    let cache_dir = abspath("cache");
    let thumb_path = cache_dir.join(&thumb_name);
    if !thumb_path.exists() || fs::metadata(&thumb_path).map(|m| m.len()).unwrap_or(0) == 0 {
        let _ = fs::create_dir_all(&cache_dir);
        if let Err(e) = create_thumbnail(rel_path, &thumb_path) {
            eprintln!("Ошибка создания миниатюры для {:?}: {}", rel_path, e);
            return "default.jpg".to_string();
        }
    }
    thumb_name
}
