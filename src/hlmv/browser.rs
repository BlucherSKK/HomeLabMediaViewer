use std::path::{Path, PathBuf};
use rocket::response::content::RawHtml;
use rocket::State;
use crate::hlmv::{
    db::MediaDb, fs::{abspath, dir_is_empty, is_spesial_file}, lang::translate, thumb::{FileType, get_file_type, get_thumb}
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
            r#"<a href="/browser/{}/" id="back">📁 [{}]</a>"#,
            parent, translate(crate::hlmv::lang::LOCALEMSG::UI_Back)
        ));
    }

    if let Ok(entries) = std::fs::read_dir(&full_path) {
        let mut entries: Vec<_> = entries.flatten().collect();
        entries.sort_by_key(|e| !e.path().is_dir());

        for entry in entries {
            let p = entry.path();
            let name = p.file_name().unwrap_or_default().to_string_lossy();
            let item_rel_path = p.strip_prefix(&base_dir).unwrap_or(&p);
            let mut item_rel_str = item_rel_path.to_str()
            .expect(translate(crate::hlmv::lang::LOCALEMSG::ParseEr))
            .to_string();

            if p.is_dir() {

                let thumb_url = format!("/cache/{}",
                                       if dir_is_empty(&p).expect(translate(crate::hlmv::lang::LOCALEMSG::ElfDirUnfound))
                                       {"empty-dir.png"} else {"dir.png"});
                let preview = format!(
                    r#"<img src="{}" loading="lazy" alt="{}" onerror="this.src='/cache/default.png'">"#,
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
                let mut root_live = "live";
                let root_media = "media-files";


                if name.starts_with('.') { continue; }

                let id = db.get_id_by_path(item_rel_path)
                    .expect(translate(crate::hlmv::lang::LOCALEMSG::DataBaseEr));
                let thumb_name = get_thumb(db, item_rel_path);
                if is_spesial_file(&p) {
                    item_rel_str = id.to_string();
                    root_live = "live-byid";
                }

                let thumb_url = format!("/cache/{}", thumb_name);
                let preview = format!(
                    r#"<img src="{}" loading="lazy" alt="{}" onerror="this.src='/cache/default.png'">"#,
                    thumb_url, name
                );
                println!("{}", preview);

                match  get_file_type(&p){
                    FileType::Video => {
                        items_html.push_str(&format!(
                            r#"<div class="item" href>
                            <a href="/{}/{}">
                            <div class="preview-wrapper">
                            {}
                            </div>
                            <div class="name">{}</div>
                            </a>
                            </div>"#,
                            root_live, item_rel_str, preview, name
                        ));
                    },
                    _ => {
                        items_html.push_str(&format!(
                            r#"<div class="item" href>
                            <a href="/{}/{}">
                            <div class="preview-wrapper">
                            {}
                            </div>
                            <div class="name">{}</div>
                            </a>
                            </div>"#,
                            root_media, item_rel_str, preview, name
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
