use rusqlite::Connection;

pub fn find_url(shortlink: &str, db: &Connection) -> String {
    let mut statement = db
        .prepare_cached("SELECT long_url FROM urls WHERE short_url = ?1")
        .unwrap();

    let links = statement
        .query_map([shortlink], |row| row.get("long_url"))
        .unwrap();

    let mut longlink = String::new();
    for link in links {
        longlink = link.unwrap();
    }

    longlink
}

pub fn getall(db: &Connection) -> Vec<String> {
    let mut statement = db.prepare_cached("SELECT * FROM urls").unwrap();

    let mut data = statement.query([]).unwrap();

    let mut links: Vec<String> = Vec::new();
    while let Some(row) = data.next().unwrap() {
        let short_url: String = row.get("short_url").unwrap();
        let long_url: String = row.get("long_url").unwrap();
        let hits: i64 = row.get("hits").unwrap();
        links.push(format!("{short_url},{long_url},{hits}"));
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

pub fn delete_link(shortlink: String, db: &Connection) {
    db.execute("DELETE FROM urls WHERE short_url = ?1", [shortlink])
        .unwrap();
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
