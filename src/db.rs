use rusqlite::{params, Connection, Result};

pub fn init_db() -> Result<()> {
    let conn = Connection::open("./database.db3")?;
    conn.execute(include_str!("./schema.sql"), [])?;

    Ok(())
}

pub fn doctors() -> Result<Vec<i32>> {
    let conn = Connection::open("./database.db3")?;
    let mut stmt = conn.prepare("SELECT id FROM doctor")?;

    let q = stmt.query_map([], |row| row.get::<usize, i32>(0))?;
    Ok(q.map(|f| f.unwrap()).collect())
}
