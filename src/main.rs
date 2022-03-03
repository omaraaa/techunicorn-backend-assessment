#[macro_use]
extern crate rocket;
extern crate argon2;
extern crate derive_more;

mod api;
mod db;

#[rocket::main]
async fn main() -> Result<(), rocket::error::Error> {
    db::init_db().unwrap();

    rocket::build()
        .mount("/", routes![api::register, api::doctors])
        .launch()
        .await
}
