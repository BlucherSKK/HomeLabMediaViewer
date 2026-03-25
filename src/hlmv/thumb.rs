use crate::hlmv::db::MediaDb;
use crate::hlmv::fs::abspath;
use crate::hlmv::lang::{translate, LOCALEMSG};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

const TEXT_ICON: &[u8] = include_bytes!("../assets/text.png");
const DEFAULT_ICON: &[u8] = include_bytes!("../assets/default.png");
const MUSIC_ICON: &[u8] = include_bytes!("../assets/music.png");
const EMPTY_DIR_ICON: &[u8] = include_bytes!("../assets/empty-dir.png");
const DIR_ICON: &[u8] = include_bytes!("../assets/dir.png");


/// @description Инициализирует стандартные иконки приложения в директории кеша.
pub fn init_default_icon() -> std::io::Result<()> {
    init_icon(abspath("cache/text.png"), TEXT_ICON)?;
    init_icon(abspath("cache/default.png"), DEFAULT_ICON)?;
    init_icon(abspath("cache/music.png"), MUSIC_ICON)?;
    init_icon(abspath("cache/dir.png"), DIR_ICON)?;
    init_icon(abspath("cache/empty-dir.png"), EMPTY_DIR_ICON)?;
    Ok(())
}

pub enum FileType {
    Text,
    Image,
    Video,
    Music,
    Subtitles,
    Unknown,
}

pub fn get_file_type(rel_path: &Path) -> FileType {
    let ext = rel_path.extension()
    .and_then(|s| s.to_str())
    .unwrap_or_default()
    .to_lowercase();

    match ext.as_str() {
        "jpg" | "jpeg" | "png" | "webp" | "gif" | "svg" | "bmp" | "tiff" | "heic" | "avif" | "ico" => FileType::Image,
        "mp4" | "mkv" | "webm" | "avi" | "mov" | "wmv" | "flv" | "m4v" | "3gp" | "ogv" => FileType::Video,
        "txt" | "md" | "rs" | "cpp" | "c" | "h" | "hpp" | "cs" | "py" | "js" | "ts" | "go" |
        "xml" | "yml" | "yaml" | "toml" | "json" | "html" | "css" | "sh" | "bat" | "sql" => FileType::Text,
        "flac" | "mp3" | "alac" | "wav" | "m4a" | "ogg" | "opus" | "aac" | "wma" => FileType::Music,
        "srt" | "vtt" | "ass" | "ssa" | "sub" | "idx" => FileType::Subtitles,
        _ => FileType::Unknown,
    }
}
/// @description Возвращает имя файла миниатюры. Генерирует её, если она отсутствует в кеше.
/// @param db Состояние базы данных для поиска или регистрации ID файла.
/// @param rel_path Путь к файлу, для которого нужна миниатюра.
pub fn get_thumb(db: &MediaDb, rel_path: &Path) -> String {
    match get_file_type(rel_path) {
        FileType::Text => "text.png".to_string(),
        FileType::Music => "music.png".to_string(),
        FileType::Unknown => "default.png".to_string(),
        _ => {
            let id = db.get_id_by_path(rel_path)
            .expect(translate(LOCALEMSG::DataBaseEr));

            let thumb_name = format!("{}.jpg", id);
            let cache_dir = abspath("cache");
            let thumb_path = cache_dir.join(&thumb_name);

            if !thumb_path.exists() || fs::metadata(&thumb_path).map(|m| m.len()).unwrap_or(0) == 0 {
                let _ = fs::create_dir_all(&cache_dir);
                if let Err(e) = create_thumbnail(rel_path, &thumb_path) {
                    eprintln!("Ошибка создания миниатюры для {:?}: {}", rel_path, e);
                    return "default.png".to_string();
                }
            }
            thumb_name
        }
    }

}

fn init_icon<P: AsRef<Path>>(path: P, buff: &[u8]) -> std::io::Result<()> {
    let path = path.as_ref();
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = fs::File::create(path)?;
        file.write_all(buff)?;
    }
    Ok(())
}

fn create_thumbnail(src: &Path, dst: &Path) -> std::io::Result<()> {

    let msrc: PathBuf = if src.is_absolute() {
        src.to_path_buf()
    } else {
        abspath("media").join(src)
    };

    if !msrc.exists() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("File not found: {:?}", msrc)));
    }

    match get_file_type(&msrc) {
        FileType::Image => {
            let img = image::open(msrc)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            img.thumbnail(150, 150)
            .save_with_format(dst, image::ImageFormat::Jpeg)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            Ok(())
        }
        FileType::Video => {
            let output = Command::new("ffmpeg")
            .args([
                "-y",
                "-i", msrc.to_str().unwrap(),
                  "-ss", "00:00:01",
                  "-frames:v", "1",
                  "-vf", "scale=150:150:force_original_aspect_ratio=increase,crop=150:150",
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
