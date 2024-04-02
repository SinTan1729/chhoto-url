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
        .expect("Error preparing SQL statement for find_url.");

    statement
        .query_row([shortlink], |row| row.get("long_url"))
        .ok()
}

pub fn getall(db: &Connection) -> Vec<DBRow> {
    let mut statement = db
        .prepare_cached("SELECT * FROM urls")
        .expect("Error preparing SQL statement for getall.");

    let mut data = statement
        .query([])
        .expect("Error executing query for getall.");

    let mut links: Vec<DBRow> = Vec::new();
    while let Some(row) = data.next().expect("Error reading fetched rows.") {
        let row_struct = DBRow {
            shortlink: row
                .get("short_url")
                .expect("Error reading shortlink from row."),
            longlink: row
                .get("long_url")
                .expect("Error reading shortlink from row."),
            hits: row.get("hits").expect("Error reading shortlink from row."),
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
    .expect("Error updating hit count.");
}

pub fn add_link(shortlink: String, longlink: String, db: &Connection) -> bool {
    db.execute(
        "INSERT INTO urls (long_url, short_url, hits) VALUES (?1, ?2, ?3)",
        (longlink, shortlink, 0),
    )
    .is_ok()
}

pub fn delete_link(shortlink: String, db: &Connection) -> bool {
    if let Ok(delta) = db.execute("DELETE FROM urls WHERE short_url = ?1", [shortlink]) {
        delta > 0
    } else {
        false
    }
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
    .expect("Unable to initialize empty database.");
    db
}
