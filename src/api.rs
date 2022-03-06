use std::marker::PhantomData;

use crate::db::{self, Claims, DB};
use derive_more::From;
use rocket::catcher::Result;
use rocket::http::{Accept, Status};
use rocket::request::{self, FromRequest, Outcome, Request};
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};

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
pub fn doctors(_auth: AccountGuard<ADMIN>) -> Json<Vec<i32>> {
    let db = DB::default().unwrap();
    Json::from(db.doctors().unwrap())
}

#[get("/doctors/<doctor_id>")]
pub fn doctor_info(doctor_id: i32) -> Json<db::DoctorInfo> {
    let db = DB::default().unwrap();
    Json::from(db.get_doctor_info(doctor_id).unwrap())
}

pub struct ADMIN {}
impl db::AccountTypeCheck for ADMIN {
    fn is_valid(claims: &db::Claims) -> bool {
        claims.account_type == db::AccountType::Admin
    }
}

pub struct DOCTOR {}
impl db::AccountTypeCheck for DOCTOR {
    fn is_valid(claims: &db::Claims) -> bool {
        claims.account_type == db::AccountType::Doctor
    }
}

pub struct PATIENT {}
impl db::AccountTypeCheck for PATIENT {
    fn is_valid(claims: &db::Claims) -> bool {
        claims.account_type == db::AccountType::Patient
    }
}

pub struct ALL {}
impl db::AccountTypeCheck for ALL {
    fn is_valid(claims: &db::Claims) -> bool {
        true
    }
}

pub struct AccountGuard<T>
where
    T: db::AccountTypeCheck,
{
    phantom: PhantomData<T>,
    claims: Claims,
}

#[derive(Debug)]
pub enum AuthError {
    Missing,
    Invalid,
}

#[rocket::async_trait]
impl<'r, T> FromRequest<'r> for AccountGuard<T>
where
    T: db::AccountTypeCheck,
{
    type Error = AuthError;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        match req.headers().get_one("Authorization") {
            None => Outcome::Failure((Status::Unauthorized, AuthError::Missing)),
            Some(auth) => {
                let jwt = auth.split(" ").last().unwrap();
                let claims = db::decode_jwt(jwt).unwrap();

                if !claims.is_valid::<T>() {
                    Outcome::Failure((Status::Unauthorized, AuthError::Invalid))
                } else {
                    Outcome::Success(Self {
                        claims: claims,
                        phantom: PhantomData,
                    })
                }
            }
        }
    }
}
