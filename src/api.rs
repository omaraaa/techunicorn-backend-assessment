use derive_more::From;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use crate::db::{self, DB};

#[post("/register", format = "json", data = "<data>")]
pub fn register(data: Json<db::RegisterData>) {
    let db = DB::default().unwrap();
    db.register_account(data.0).unwrap();
}

#[post("/login", format = "json", data = "<data>")]
pub fn login(data: Json<db::LoginData>) -> String {
    let db = DB::default().unwrap();
    db.login(data.0).unwrap()
}

#[get("/doctors")]
pub fn doctors() -> Json<Vec<i32>> {
    let db = DB::default().unwrap();
    Json::from(db.doctors().unwrap())
}
