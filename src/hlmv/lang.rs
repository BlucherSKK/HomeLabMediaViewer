use std;

pub enum LOCALEMSG {
    ElfDirUnfound,
    DataBaseInitFail,
}



pub fn translate(msg: LOCALEMSG) -> &'static str {
    let lang: &'static str = match std::env::var("LANG") {
        Ok(val) => Box::leak(val.into_boxed_str()),
        Err(_) => "en_US.UTF-8", // Значение по умолчанию, если переменная не найдена
    };

    if lang.starts_with("ru") {
        match msg {
        LOCALEMSG::ElfDirUnfound => "Ошибка определения местоположения исполняемого файла",
        LOCALEMSG::DataBaseInitFail => "Ошибка инициализации базы данных",
    }
    } else {
        match msg {
        LOCALEMSG::ElfDirUnfound => "Error: unavaible find executable file",
        LOCALEMSG::DataBaseInitFail => "Error: can not init data base",
    }
    }
}
