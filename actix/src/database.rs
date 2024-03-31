use rusqlite::Connection;
use serde::Serialize;

#[derive(Serialize)]
pub struct DBRow {
    shortlink: String,
    longlink: String,
    hits: i64,
}

pub fn find_url(shortlink: &str, db: &Connection) -> Option<String> {
    let mut statement = db
        .prepare_cached("SELECT long_url FROM urls WHERE short_url = ?1")
        .unwrap();

    statement
        .query_row([shortlink], |row| row.get("long_url"))
        .ok()
}

pub fn getall(db: &Connection) -> Vec<DBRow> {
    let mut statement = db.prepare_cached("SELECT * FROM urls").unwrap();

    let mut data = statement.query([]).unwrap();

    let mut links: Vec<DBRow> = Vec::new();
    while let Some(row) = data.next().unwrap() {
        let row_struct = DBRow {
            shortlink: row.get("short_url").unwrap(),
            longlink: row.get("long_url").unwrap(),
            hits: row.get("hits").unwrap(),
        };
        links.push(row_struct);
    }

    links
}

pub fn add_hit(shortlink: &str, db: &Connection) {
    db.execute(
        "UPDATE urls SET hits = hits + 1 WHERE short_url = ?1",
        [shortlink],
    )
    .unwrap();
}

pub fn add_link(shortlink: String, longlink: String, db: &Connection) -> bool {
    db.execute(
        "INSERT INTO urls (long_url, short_url, hits) VALUES (?1, ?2, ?3)",
        (longlink, shortlink, 0),
    )
    .is_ok()
}

pub fn delete_link(shortlink: String, db: &Connection) -> bool {
    let out = db.execute("DELETE FROM urls WHERE short_url = ?1", [shortlink]);
    out.is_ok() && (out.unwrap() > 0)
}

pub fn open_db(path: String) -> Connection {
    let db = Connection::open(path).expect("Unable to open database!");
    // Create table if it doesn't exist
    db.execute(
        "CREATE TABLE IF NOT EXISTS urls (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            long_url TEXT NOT NULL,
            short_url TEXT NOT NULL,
            hits INTEGER NOT NULL
            )",
        [],
    )
    .unwrap();
    db
}
