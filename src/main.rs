#[macro_use] extern crate rocket;

use std;
mod hlmv;



#[get("/")]
fn index() -> _ {
    rocket::response::content::RawHtml(include_str!("./web/index.html"))
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}
