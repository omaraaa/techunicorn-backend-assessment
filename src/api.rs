use rocket::serde::{json::Json, Deserialize, Serialize};

use crate::db;

#[derive(Serialize, Deserialize)]
pub enum AccountType {
    Patient,
    Doctor,
    Admin,
}

#[derive(Serialize, Deserialize)]
pub struct Doctor {
    id: i64,
}

#[derive(Serialize, Deserialize)]
pub struct RegisterForm {
    name: String,
    account_type: AccountType,
}

#[derive(Serialize, Deserialize)]
pub struct LoginForm {
    name: String,
    account_type: AccountType,
}

#[post("/register", data = "<form>")]
pub fn register(form: Json<RegisterForm>) -> Result<(), ()> {
    todo!()
}

#[post("/login", data = "<form>")]
pub fn login(form: Json<LoginForm>) -> Result<(), ()> {
    todo!()
}

#[get("/doctors")]
pub fn doctors() -> Json<Vec<i32>> {
    Json::from(db::doctors().unwrap())
}
