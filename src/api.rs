use rocket::serde::{json::Json, Deserialize, Serialize};

use crate::db;

#[post("/register", data = "<data>")]
pub fn register(data: Json<db::RegisterData>) -> Result<(), ()> {
    db::registerAccount(data.0).unwrap();
    Ok(())
}

#[get("/doctors")]
pub fn doctors() -> Json<Vec<i32>> {
    Json::from(db::doctors().unwrap())
}
