#[macro_use]
extern crate rocket;
extern crate argon2;
extern crate derive_more;

mod api;
mod db;

#[rocket::main]
async fn main() -> Result<(), rocket::error::Error> {
    let db = db::DB::default().unwrap();
    db.init_schema().unwrap();

    rocket::build()
        .mount("/", routes![api::register, api::login, api::doctors])
        .launch()
        .await
}
