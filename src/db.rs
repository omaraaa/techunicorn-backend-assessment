use rusqlite::{params, Connection};

use derive_more::From;
use serde::{Deserialize, Serialize};
use std::result::Result;

#[derive(Debug, From)]
pub enum Error {
    DBError(rusqlite::Error),
    HashingError(argon2::Error),
}

pub fn init_db() -> Result<(), Error> {
    let conn = Connection::open("./database.db3")?;
    conn.execute_batch(include_str!("./schema.sql"))?;

    Ok(())
}

pub fn doctors() -> Result<Vec<i32>, Error> {
    let conn = Connection::open("./database.db3")?;
    let mut stmt = conn.prepare("SELECT id FROM doctor")?;
    let q = stmt.query_map([], |row| row.get::<usize, i32>(0))?;
    Ok(q.collect::<Result<_, _>>()?)
}

pub struct Account {
    email: String,
}

#[derive(Serialize, Deserialize)]
pub enum AccountType {
    Patient,
    Doctor,
    Admin,
}

#[derive(Serialize, Deserialize)]
pub struct RegisterData {
    name: String,
    email: String,
    password: String,
    account_type: AccountType,
}

use argon2::{self, Config};

pub fn registerAccount(data: RegisterData) -> Result<(), Error> {
    let conn = Connection::open("./database.db3")?;

    let salt = b"should be random";
    let config = Config::default();
    let passhash = argon2::hash_encoded(data.password.as_bytes(), salt, &config)?;

    let mut stmnt = conn.prepare(
        "INSERT INTO account (fullname, email, passhash) VALUES (?1, ?2, ?3) RETURNING id",
    )?;

    let mut itr = stmnt.query_map(params![data.name, data.email, passhash], |r| {
        r.get::<_, i32>(0)
    })?;
    let id = itr.next().unwrap()?;

    match data.account_type {
        Doctor => {
            registerDoctor(id)?;
        }
    }

    Ok(())
}

fn registerDoctor(id: i32) -> Result<(), Error> {
    let conn = Connection::open("./database.db3")?;
    conn.execute("INSERT INTO doctor(id) values (?1)", params![id])?;
    Ok(())
}
