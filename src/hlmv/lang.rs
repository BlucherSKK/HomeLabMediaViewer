use std;

pub enum LOCALEMSG {
    ElfDirUnfound,
    DataBaseInitFail,
    DataBaseEr,
    InitEr,
    DBuploadFail,
    DBgetIDFail,
    ThumbCrate,
    FileCreated,
    DirEmptyChekFail,
}



pub fn translate(msg: LOCALEMSG) -> &'static str {
    let lang: &'static str = match std::env::var("LANG") {
        Ok(val) => Box::leak(val.into_boxed_str()),
        Err(_) => "en_US.UTF-8",
    };

    if lang.starts_with("ru") {
        match msg {
        LOCALEMSG::ElfDirUnfound => "Ошибка определения местоположения исполняемого файла",
        LOCALEMSG::DataBaseInitFail => "Ошибка инициализации базы данных",
        LOCALEMSG::DataBaseEr => "Ошибка базы данных:",
        LOCALEMSG::InitEr => "Ошибка инициализации приложения",
        LOCALEMSG::DBuploadFail => "Ошибкак добавления записи в базу данных",
        LOCALEMSG::DBgetIDFail => "Ошибка получения препросмотра, используем дефолтный ассет",
        LOCALEMSG::ThumbCrate => "Ошибка создания файла предпросмотра",
        LOCALEMSG::FileCreated => "Создан файл",
        LOCALEMSG::DirEmptyChekFail => "Неудалось проверить содержимое директории",
    }
    } else {
        match msg {
        LOCALEMSG::ElfDirUnfound => "Error: unavaible find executable file",
        LOCALEMSG::DataBaseInitFail => "Error: can not init data base",
        LOCALEMSG::DataBaseEr => "Data base error:",
        LOCALEMSG::InitEr => "Error initialasing application",
        LOCALEMSG::DBuploadFail => "Error: can not upload new entity in data base",
        LOCALEMSG::DBgetIDFail => "Error: can not get prewie, using default asset",
        LOCALEMSG::ThumbCrate => "Error: can not create prewie",
        LOCALEMSG::FileCreated => "File created",
        LOCALEMSG::DirEmptyChekFail => "Error: can not check directory",
    }
    }
}
