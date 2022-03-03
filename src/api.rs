use rocket::serde::{json::Json, Deserialize, Serialize};

use crate::db;

#[derive(Serialize, Deserialize)]
pub struct Doctor {}

#[post("/register", data = "<form>")]
pub fn register(form: Json<RegisterData>) -> Result<(), ()> {
    db::registerAccount(form).unwrap();
}

#[derive(Serialize, Deserialize)]
pub struct LoginData {
    name: String,
    account_type: AccountType,
}

#[post("/login", data = "<form>")]
pub fn login(form: Json<LoginData>) -> Result<(), ()> {
    todo!()
}

#[get("/doctors")]
pub fn doctors() -> Json<Vec<i32>> {
    Json::from(db::doctors().unwrap())
}
