use chrono::{Date, DateTime, Duration, FixedOffset, NaiveDate};
use rusqlite::{params, Connection, Row, Transaction, TransactionBehavior};

use derive_more::From;
use serde::{Deserialize, Serialize};
use std::{
    cell::{Ref, RefCell},
    ops::Deref,
    rc::Rc,
    result::Result,
};

use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use std::time::{SystemTime, UNIX_EPOCH};

use argon2::{self, Config};

use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, From)]
pub enum Error {
    DBError(rusqlite::Error),
    HashingError(argon2::Error),
    JWTError(jsonwebtoken::errors::Error),
    InvalidPassword,
}

pub fn password_hash(password: &[u8]) -> Result<String, Error> {
    let salt = b"should be random";
    let config = Config::default();
    Ok(argon2::hash_encoded(password, salt, &config)?)
}

pub fn verify_password(hash: &str, password: &[u8]) -> Result<bool, Error> {
    Ok(argon2::verify_encoded(&hash, password)?)
}

#[derive(Debug)]
pub struct DB<'a> {
    con: Connection,
    tx: Option<Transaction<'a>>,
}

impl DB<'_> {
    pub fn default() -> Result<Self, Error> {
        Ok(Self {
            con: Connection::open("./database.db3")?,
            tx: None,
        })
    }

    pub fn init(path: Option<&str>) -> Result<Self, Error> {
        Ok(Self {
            con: match path {
                Some(p) => Connection::open(p)?,
                _ => Connection::open_in_memory()?,
            },
            tx: None,
        })
    }

    pub fn init_schema(&self) -> Result<(), Error> {
        self.con().execute_batch(include_str!("./schema.sql"))?;
        Ok(())
    }

    pub fn con(&self) -> &Connection {
        return &self.con;
    }

    pub fn doctors(&self) -> Result<Vec<i32>, Error> {
        let mut stmt = self
            .con()
            .prepare("SELECT id FROM account where account_type=?1")?;
        let q = stmt.query_map(params![AccountType::Doctor as i32], |row| {
            row.get::<usize, i32>(0)
        })?;
        Ok(q.collect::<Result<_, _>>()?)
    }

    pub fn doctors_stats(
        &self,
        day: Date<FixedOffset>,
    ) -> Result<Vec<DoctorAppointmentStats>, Error> {
        let mut stmt = self.con().prepare(
            "
            SELECT account.id, sum(appointment.duration_mins) as s, count(appointment.doctor) as c 
            FROM account 
            LEFT JOIN appointment
            ON appointment.doctor = account.id 
            GROUP BY account.id
            HAVING account_type=?1 
            AND date(starting_date) = ?2
            AND appointment_status != ?3
            ",
        )?;
        let q = stmt.query_map(
            params![
                AccountType::Doctor as i32,
                day.format("%Y-%m-%d").to_string(),
                AppointmentStatus::Cancelled as i32
            ],
            |row| {
                Ok(DoctorAppointmentStats {
                    doctor_id: row.get(0)?,
                    booked_mins: row.get(1)?,
                    appointments_count: row.get(2)?,
                })
            },
        )?;
        Ok(q.collect::<Result<_, _>>()?)
    }

    pub fn register(&self, data: RegisterData) -> Result<(), Error> {
        let id = {
            let mut stmnt = self.con().prepare(
                "INSERT INTO account (fullname, email, passhash, account_type) VALUES (?1, ?2, ?3, ?4) RETURNING id",
            )?;

            let passhash = password_hash(data.password.as_bytes())?;

            stmnt.query_row(
                params![data.name, data.email, passhash, data.account_type as i32],
                |r| r.get(0),
            )?
        };

        self.register_account_type(data.account_type, id)?;

        Ok(())
    }

    fn register_account_type(&self, account_type: AccountType, id: i32) -> Result<(), Error> {
        let sql = match account_type {
            AccountType::Doctor => "INSERT INTO doctor(id) values (?1)",
            AccountType::Patient => "INSERT INTO patient(id) values (?1)",
            AccountType::Admin => "INSERT INTO admin(id) values (?1)",
        };

        self.con().execute(sql, params![id])?;
        Ok(())
    }

    pub fn login(&self, data: LoginData) -> Result<JWT, Error> {
        let mut stmt = self.con().prepare(
            "SELECT id, account_type, passhash 
        FROM account 
        WHERE account.email=?1
        ",
        )?;

        let claims: Claims = stmt.query_row(params![data.email], |row: &Row| {
            let password_hash: String = row.get(2)?;

            match verify_password(&password_hash, data.password.as_bytes()) {
                Ok(false) => return Ok(Err(Error::InvalidPassword)),
                Err(e) => return Ok(Err(e)),
                Ok(true) => {}
            }

            Ok(Ok(Claims {
                sub: row.get(0)?,
                account_type: AccountType::try_from(row.get::<_, i32>(1)?).unwrap(),
                iat: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            }))
        })??;

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret("secret".as_ref()),
        )?;

        Ok(token)
    }

    pub fn get_doctor_info(&self, doctor_id: i32) -> Result<DoctorInfo, Error> {
        let mut stmnt = self.con().prepare("SELECT fullname, specialty, details FROM account LEFT JOIN doctor ON account.id = doctor.id WHERE account.id = ?1")?;
        let info = stmnt.query_row(params![doctor_id], |row| {
            Ok(DoctorInfo {
                name: row.get(0)?,
                specialty: row.get(1)?,
                details: row.get(2)?,
            })
        })?;
        Ok(info)
    }

    pub fn get_doctor_appointments(
        &self,
        doctor_id: i32,
        day: DateTime<FixedOffset>,
    ) -> Result<Vec<Appointment>, Error> {
        let mut stmnt = self
            .con()
            .prepare("SELECT * FROM appointment WHERE doctor = ?1 and date(starting_date) = ?2")?;

        let mut q = stmnt.query_map(
            params![doctor_id, day.format("%Y-%m-%d").to_string()],
            appointment_from_row,
        )?;

        Ok(q.collect::<Result<_, _>>()?)
    }

    pub fn get_patient_appointments_history(
        &self,
        patient_id: i32,
    ) -> Result<Vec<Appointment>, Error> {
        let mut stmnt = self
            .con()
            .prepare("SELECT * FROM appointment WHERE patient = ?1")?;

        let mut q = stmnt.query_map(params![patient_id], appointment_from_row)?;

        Ok(q.collect::<Result<_, _>>()?)
    }

    pub fn book_appointment(&self, request: AppointmentRequest) -> Result<i32, Error> {
        let mut stmnt = self
            .con()
            .prepare(
                "INSERT INTO appointment(doctor, patient, appointment_status, start_date, duration_mins)
             VALUES (?1, ?2, ?3, ?4, ?5) RETURNING id",
            )
            .unwrap();

        let q: i32 = stmnt.query_row(
            params![
                request.doctor_id,
                request.patient_id,
                AppointmentStatus::Booked as i32,
                request.start_date.timestamp() as i32,
                request.duration
            ],
            |row| row.get(0),
        )?;

        Ok(q)
    }

    pub fn is_valid_appointment_request(
        &self,
        doctor_id: i32,
        start_date: &DateTime<FixedOffset>,
        duration: i32,
    ) -> Result<bool, Error> {
        if duration < 15 || duration > 120 {
            return Ok(false);
        }

        let stats = self.get_doctor_stats(doctor_id, start_date.date())?;
        if stats.appointments_count >= 12 || stats.booked_mins > 8 * 60 {
            return Ok(false);
        }

        return Ok(true);
    }

    pub fn get_doctor_stats(
        &self,
        doctor_id: i32,
        day: Date<FixedOffset>,
    ) -> Result<DoctorAppointmentStats, Error> {
        let mut stmnt = self.con().prepare(
            "SELECT doctor, count(*), sum(duration) FROM appointments
                 GROUP BY doctor
                 HAVING doctor = ?1 and date(starting_date) = ?2 and appointment_status != ?3",
        )?;

        let q = stmnt.query_row(
            params![
                doctor_id,
                day.format("%Y-%m-%d").to_string(),
                AppointmentStatus::Cancelled as i32
            ],
            |row| {
                Ok(DoctorAppointmentStats {
                    doctor_id: row.get(0)?,
                    appointments_count: row.get(1)?,
                    booked_mins: row.get(2)?,
                })
            },
        )?;

        Ok(q)
    }

    pub fn get_appointment(&self, appointment_id: i32) -> Result<Appointment, Error> {
        let mut stmnt = self
            .con()
            .prepare("SELECT * FROM appointment where id = ?1")?;

        let q = stmnt.query_row(params![appointment_id], appointment_from_row)?;

        Ok(q)
    }

    pub fn set_appointment_status(
        &self,
        appointment_id: i32,
        status: AppointmentStatus,
    ) -> Result<(), Error> {
        let mut stmnt = self.con().prepare(
            "
            UPDATE appointment SET appointment_status = ?1 WHERE id = ?2
        ",
        )?;

        stmnt.execute(params![status as i32, appointment_id])?;

        Ok(())
    }
}

fn appointment_from_row(row: &Row) -> Result<Appointment, rusqlite::Error> {
    let appointment = Appointment {
        id: row.get(0)?,
        doctor_id: row.get(1)?,
        patient_id: row.get(2)?,
        status: AppointmentStatus::try_from(row.get::<_, i32>(3)?).unwrap(),
        start_date: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?).unwrap(),
        duration: row.get(5)?,
    };

    Ok(appointment)
}

pub fn decode_jwt(jwt: &str) -> Result<Claims, Error> {
    let token = decode::<Claims>(
        jwt,
        &DecodingKey::from_secret("secret".as_ref()),
        &Validation::new(Algorithm::HS256),
    )?;

    Ok(token.claims)
}

#[derive(
    Debug, Serialize, Deserialize, Clone, Copy, IntoPrimitive, TryFromPrimitive, PartialEq, Eq,
)]
#[repr(i32)]
pub enum AccountType {
    Patient = 0,
    Doctor = 1,
    Admin = 2,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RegisterData {
    name: String,
    email: String,
    password: String,
    account_type: AccountType,
}

#[derive(Serialize, Deserialize)]
pub struct LoginData {
    email: String,
    password: String,
}

pub type JWT = String;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,
    pub account_type: AccountType,
    pub iat: u64,
}

impl Claims {
    pub fn is_valid<T>(&self) -> bool
    where
        T: ValidClaimsChecker,
    {
        T::is_valid(self)
    }
}

pub trait ValidClaimsChecker {
    fn is_valid(claims: &Claims) -> bool;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DoctorInfo {
    name: String,
    specialty: String,
    details: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Appointment {
    id: i32,
    pub doctor_id: i32,
    pub patient_id: i32,
    pub start_date: DateTime<FixedOffset>,
    pub duration: i32,
    pub status: AppointmentStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppointmentRequest {
    pub doctor_id: i32,
    pub patient_id: i32,
    pub start_date: DateTime<FixedOffset>,
    pub duration: i32,
}

#[derive(
    Debug, Serialize, Deserialize, Clone, Copy, IntoPrimitive, TryFromPrimitive, PartialEq, Eq,
)]
#[repr(i32)]
pub enum AppointmentStatus {
    Booked,
    Cancelled,
    Done,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DoctorAppointmentStats {
    pub doctor_id: i32,
    pub appointments_count: i32,
    pub booked_mins: i32,
}

#[cfg(test)]
mod tests {

    use serde::{Deserialize, Serialize};

    use super::{LoginData, RegisterData, DB};

    #[derive(Serialize, Deserialize)]
    struct MockData {
        registerations: Vec<RegisterData>,
    }

    #[test]
    fn test_registeration() {
        let db = DB::init(None).unwrap();
        db.init_schema().unwrap();

        let mock: MockData = serde_json::from_str(include_str!("./mock.json")).unwrap();

        let r = mock.registerations.get(0).unwrap().clone();
        db.register(r.clone()).unwrap();
        let jwt = db
            .login(LoginData {
                email: r.email.clone(),
                password: r.password.clone(),
            })
            .unwrap();

        println!("{}", jwt);
    }
}
