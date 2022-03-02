#[macro_use]
extern crate rocket;

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
