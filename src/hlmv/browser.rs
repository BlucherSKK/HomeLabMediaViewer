use std::path::{Path, PathBuf};
use rocket::response::content::RawHtml;
use rocket::State;
use crate::hlmv::{
    db::MediaDb, fs::{abspath, dir_is_empty}, lang::translate, thumb::{get_file_type, get_thumb, FileType}
};

/*
 * @description Генерирует HTML-страницу файлового браузера для указанного пути.
 * @param rel_path Относительный путь к директории, которую нужно отобразить.
 * @param db Состояние базы данных медиа для получения миниатюр.
*/
pub fn render_browser(rel_path: PathBuf, db: &State<MediaDb>) -> RawHtml<String> {
    let template = include_str!("../web/browser.html");
    let base_dir = abspath("./media");
    let full_path = base_dir.join(&rel_path);
    let mut items_html = String::new();
    let mut back = String::new();

    if rel_path.as_os_str() != "" && rel_path.as_os_str() != "." {
        let parent = rel_path.parent().unwrap_or(Path::new("")).to_string_lossy();
        back.push_str(&format!(
            r#"<a href="/browser/{}" id="back">📁 [Назад]</a>"#,
            parent
        ));
    }

    if let Ok(entries) = std::fs::read_dir(&full_path) {
        let mut entries: Vec<_> = entries.flatten().collect();
        entries.sort_by_key(|e| !e.path().is_dir());

        for entry in entries {
            let p = entry.path();
            let name = p.file_name().unwrap_or_default().to_string_lossy();
            let item_rel_path = p.strip_prefix(&base_dir).unwrap_or(&p);
            let item_rel_str = item_rel_path.to_string_lossy();

            if p.is_dir() {
                let thumb_url = format!("/cache/{}",
                                       if dir_is_empty(&p).expect(translate(crate::hlmv::lang::LOCALEMSG::ElfDirUnfound))
                                       {"empty-dir.png"} else {"dir.png"});
                let preview = format!(
                    r#"<img src="{}" loading="lazy" alt="{}" onerror="this.src='/static/default.jpg'">"#,
                    thumb_url, name
                );

                items_html.push_str(&format!(
                    r#"<div class="item" href>
                    <a href="/browser/{}">
                    <div class="preview-wrapper">
                    {}
                    </div>
                    <div class="name">{}</div>
                    </a>
                    </div>"#,
                    item_rel_str, preview, name
                ));
            } else {
                if name.starts_with('.') { continue; }

                let thumb_name = get_thumb(db, &p);
                let thumb_url = format!("/cache/{}", thumb_name);
                let preview = format!(
                    r#"<img src="{}" loading="lazy" alt="{}" onerror="this.src='/static/default.jpg'">"#,
                    thumb_url, name
                );

                match  get_file_type(&p){
                    FileType::Video => {
                        items_html.push_str(&format!(
                            r#"<div class="item" href>
                            <a href="/live/{}">
                            <div class="preview-wrapper">
                            {}
                            </div>
                            <div class="name">{}</div>
                            </a>
                            </div>"#,
                            item_rel_str, preview, name
                        ));
                    },
                    _ => {
                        items_html.push_str(&format!(
                            r#"<div class="item" href>
                            <a href="/media-files/{}">
                            <div class="preview-wrapper">
                            {}
                            </div>
                            <div class="name">{}</div>
                            </a>
                            </div>"#,
                            item_rel_str, preview, name
                        ));
                    }
                }
            }
        }
    }

    let output = template
    .replace("{{PATH}}", &rel_path.to_string_lossy())
    .replace("{{CONTENT}}", &items_html)
    .replace("{{BACK}}", &back);

    RawHtml(output)
}
