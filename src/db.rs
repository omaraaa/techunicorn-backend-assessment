use rusqlite::{params, Connection};

use derive_more::From;
use serde::{Deserialize, Serialize};
use std::result::Result;

use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, From)]
pub enum Error {
    DBError(rusqlite::Error),
    HashingError(argon2::Error),
    JWTError(jsonwebtoken::errors::Error),
}

#[derive(Debug)]
pub struct DB {
    con: Connection,
}

impl DB {
    pub fn default() -> Result<Self, Error> {
        Ok(Self {
            con: Connection::open("./database.db3")?,
        })
    }

    pub fn init(path: Option<&str>) -> Result<Self, Error> {
        match path {
            Some(p) => Ok(Self {
                con: Connection::open(p)?,
            }),
            None => Ok(Self {
                con: Connection::open_in_memory()?,
            }),
        }
    }

    pub fn init_schema(&self) -> Result<(), Error> {
        self.con.execute_batch(include_str!("./schema.sql"))?;
        Ok(())
    }

    pub fn doctors(&self) -> Result<Vec<i32>, Error> {
        let mut stmt = self
            .con
            .prepare("SELECT id FROM account where account_type=?1")?;
        let q = stmt.query_map(params![AccountType::Doctor as i32], |row| {
            row.get::<usize, i32>(0)
        })?;
        Ok(q.collect::<Result<_, _>>()?)
    }

    pub fn register_account(&self, data: RegisterData) -> Result<(), Error> {
        let id = {
            let mut stmnt = self.con.prepare(
                "INSERT INTO account (fullname, email, passhash, account_type) VALUES (?1, ?2, ?3, ?4) RETURNING id",
            )?;

            let passhash = password_hash(data.password)?;

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

        self.con.execute(sql, params![id])?;
        Ok(())
    }

    pub fn login(&self, data: LoginData) -> Result<String, Error> {
        let hash = password_hash(data.password)?;
        let conn = Connection::open("./database.db3")?;
        let mut stmt = conn.prepare(
            "SELECT email, account_type 
        FROM account 
        WHERE account.email=?1 AND account.passhash=?2
        ",
        )?;

        let claims: Claims = stmt.query_row(
            params![data.email, hash],
            |row| -> Result<Claims, rusqlite::Error> {
                Ok(Claims {
                    sub: row.get(0)?,
                    account_type: row.get(1)?,
                    iat: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                })
            },
        )?;

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret("secret".as_ref()),
        )?;

        Ok(token)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
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

use argon2::{self, Config};

pub fn password_hash(password: String) -> Result<String, Error> {
    let salt = b"should be random";
    let config = Config::default();
    Ok(argon2::hash_encoded(password.as_bytes(), salt, &config)?)
}

#[derive(Serialize, Deserialize)]
pub struct LoginData {
    email: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    account_type: i32,
    iat: u64,
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
        db.register_account(r.clone()).unwrap();
        let jwt = db
            .login(LoginData {
                email: r.email.clone(),
                password: r.password.clone(),
            })
            .unwrap();

        println!("{}", jwt);
    }
}
