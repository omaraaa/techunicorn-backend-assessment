use std::marker::PhantomData;

use crate::db::{self, AccountType, Appointment, Claims, DB};
use chrono::{DateTime, Duration, FixedOffset};
use derive_more::From;
use rocket::http::{Accept, Status};
use rocket::request::{self, FromRequest, Outcome, Request};
use rocket::response::status::{BadRequest, Forbidden};
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};

use std::result::Result;

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

#[get("/doctors/<doctor_id>/slots", format = "json", data = "<input>")]
pub fn doctor_booked_slots(
    doctor_id: i32,
    input: Json<DateInput>,
    auth: AccountGuard<ALL>,
) -> Json<Vec<BookedTimeslotsView>> {
    let db = DB::default().unwrap();
    let date = chrono::DateTime::parse_from_str(&input.date, "%Y-%m-%d").unwrap();
    let appointments = db.get_doctor_appointments(doctor_id, date).unwrap();

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

#[derive(Debug, Serialize, Deserialize)]
pub struct BookInput {
    start_date: DateTime<FixedOffset>,
    duration: i32,
}

#[post("/doctors/<doctor_id>/book", format = "json", data = "<input>")]
pub fn book_doctor(
    doctor_id: i32,
    input: Json<BookInput>,
    auth: AccountGuard<PATIENT>,
) -> Result<Json<i32>, BadRequest<String>> {
    let db = DB::default().unwrap();
    let valid_request = db
        .is_valid_appointment_request(doctor_id, &input.start_date, input.duration)
        .unwrap();

    if valid_request {
        let appointment_id = db
            .book_appointment(db::AppointmentRequest {
                doctor_id: doctor_id,
                patient_id: auth.claims.sub,
                start_date: input.start_date,
                duration: input.duration,
            })
            .unwrap();
        Ok(Json::from(appointment_id))
    } else {
        Err(BadRequest(Some("doctor unavailable".to_string())))
    }
}

#[post("/appointments/<appointment_id>/cancel")]
pub fn cancel_appointment(
    appointment_id: i32,
    auth: AccountGuard<DOCTOR_ADMIN>,
) -> Result<(), Forbidden<String>> {
    let db = DB::default().unwrap();
    let appointment = db.get_appointment(appointment_id).unwrap();

    match auth.claims.account_type {
        db::AccountType::Admin => db
            .set_appointment_status(appointment_id, db::AppointmentStatus::Cancelled)
            .unwrap(),
        db::AccountType::Doctor if auth.claims.sub == appointment.doctor_id => db
            .set_appointment_status(appointment_id, db::AppointmentStatus::Cancelled)
            .unwrap(),
        _ => return Err(Forbidden::<String>(Some("Not Authorized".to_string()))),
    }

    Ok(())
}

#[get("/doctors/available", format = "json", data = "<input>")]
pub fn available_doctors(
    input: Json<DateInput>,
    _auth: AccountGuard<ALL>,
) -> Json<Vec<db::DoctorAppointmentStats>> {
    let db = DB::default().unwrap();
    let date = chrono::DateTime::parse_from_str(&input.date, "%Y-%m-%d").unwrap();
    Json::from(
        db.doctors_stats(date.date())
            .unwrap()
            .into_iter()
            .filter(|stats| stats.booked_mins < 8 * 60 && stats.appointments_count < 12)
            .collect::<Vec<db::DoctorAppointmentStats>>(),
    )
}

#[get("/appointments/<appointment_id>")]
pub fn appointment_details(
    appointment_id: i32,
    auth: AccountGuard<ALL>,
) -> Result<Json<Appointment>, Forbidden<String>> {
    let db = DB::default().unwrap();
    let ap = db.get_appointment(appointment_id).unwrap();

    match auth.claims.account_type {
        db::AccountType::Admin => Ok(Json::from(ap)),
        db::AccountType::Doctor if auth.claims.sub == ap.doctor_id => Ok(Json::from(ap)),
        db::AccountType::Patient if auth.claims.sub == ap.patient_id => Ok(Json::from(ap)),

        _ => return Err(Forbidden::<String>(Some("Not Authorized".to_string()))),
    }
}

#[get("/patients/<patient_id>/history")]
pub fn patient_history(
    patient_id: i32,
    auth: AccountGuard<ALL>,
) -> Result<Json<Vec<Appointment>>, Forbidden<String>> {
    if auth.claims.account_type == AccountType::Patient && auth.claims.sub != patient_id {
        return Err(Forbidden::<String>(Some("Not Authorized".to_string())));
    }

    let db = DB::default().unwrap();
    let patient_appointments = db.get_patient_appointments_history(patient_id).unwrap();

    Ok(Json::from(patient_appointments))
}

#[get("/doctors/by_top_appointments", format = "json", data = "<input>")]
pub fn stats_top_appointments(
    input: Json<DateInput>,
    _auth: AccountGuard<ADMIN>,
) -> Json<Vec<db::DoctorAppointmentStats>> {
    let db = DB::default().unwrap();
    let date = chrono::DateTime::parse_from_str(&input.date, "%Y-%m-%d").unwrap();
    let mut stats = db.doctors_stats(date.date()).unwrap();

    stats.sort_by(|a, b| b.appointments_count.cmp(&a.appointments_count));
    let max = stats[0].appointments_count;
    Json::from(
        stats
            .into_iter()
            .filter(|a| a.appointments_count == max)
            .collect::<Vec<db::DoctorAppointmentStats>>(),
    )
}

#[get("/doctors/with_six_hours_plus", format = "json", data = "<input>")]
pub fn stats_greaterthan_hours(
    input: Json<DateInput>,
    _auth: AccountGuard<ADMIN>,
) -> Json<Vec<db::DoctorAppointmentStats>> {
    let db = DB::default().unwrap();
    let date = chrono::DateTime::parse_from_str(&input.date, "%Y-%m-%d").unwrap();
    let stats = db.doctors_stats(date.date()).unwrap();

    Json::from(
        stats
            .into_iter()
            .filter(|a| a.booked_mins >= 6 * 60)
            .collect::<Vec<db::DoctorAppointmentStats>>(),
    )
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DateInput {
    date: String,
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

pub struct DOCTOR_ADMIN {}
impl db::ValidClaimsChecker for DOCTOR_ADMIN {
    fn is_valid(claims: &db::Claims) -> bool {
        claims.account_type == db::AccountType::Doctor
            || claims.account_type == db::AccountType::Admin
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
