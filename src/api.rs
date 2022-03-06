use std::marker::PhantomData;

use crate::db::{self, Claims, DB};
use chrono::{DateTime, Duration, FixedOffset};
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
    db.register(data.0).unwrap();
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

#[derive(Debug, Serialize, Deserialize)]
pub struct BookedTimeslotsView {
    doctor_id: i32,
    patient_id: Option<i32>,
    start_date: DateTime<FixedOffset>,
    duration: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BookedSlotsInput {
    day: String,
}

#[get("/doctors/<doctor_id>/slots", format = "json", data = "<input>")]
pub fn doctor_booked_slots(
    doctor_id: i32,
    input: Json<BookedSlotsInput>,
    auth: AccountGuard<ALL>,
) -> Json<Vec<BookedTimeslotsView>> {
    let db = DB::default().unwrap();
    let day = chrono::DateTime::parse_from_str(&input.day, "%Y-%m-%d").unwrap();
    let appointments = db.get_doctor_appointments(doctor_id, day).unwrap();

    Json::from(
        appointments
            .into_iter()
            .map(|a| BookedTimeslotsView {
                doctor_id: a.doctor_id,
                patient_id: match auth.claims.account_type {
                    db::AccountType::Patient => None,
                    _ => Some(a.patient_id),
                },
                start_date: a.start_date,
                duration: a.duration,
            })
            .collect::<Vec<BookedTimeslotsView>>(),
    )
}

pub struct ADMIN {}
impl db::ValidClaimsChecker for ADMIN {
    fn is_valid(claims: &db::Claims) -> bool {
        claims.account_type == db::AccountType::Admin
    }
}

pub struct DOCTOR {}
impl db::ValidClaimsChecker for DOCTOR {
    fn is_valid(claims: &db::Claims) -> bool {
        claims.account_type == db::AccountType::Doctor
    }
}

pub struct PATIENT {}
impl db::ValidClaimsChecker for PATIENT {
    fn is_valid(claims: &db::Claims) -> bool {
        claims.account_type == db::AccountType::Patient
    }
}

pub struct ALL {}
impl db::ValidClaimsChecker for ALL {
    fn is_valid(claims: &db::Claims) -> bool {
        true
    }
}

pub struct AccountGuard<T>
where
    T: db::ValidClaimsChecker,
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
    T: db::ValidClaimsChecker,
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
