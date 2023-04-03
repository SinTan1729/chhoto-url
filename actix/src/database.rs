use rusqlite::Connection;

pub fn find_url(shortlink: &str) -> String {
    let db = Connection::open("./urls.sqlite").expect("Unable to open database!");

    let mut statement = db
        .prepare_cached("SELECT long_url FROM urls WHERE short_url = ?1")
        .unwrap();

    let links = statement
        .query_map([shortlink], |row| Ok(row.get("long_url")?))
        .unwrap();

    let mut longlink = "".to_string();
    for link in links {
        longlink = link.unwrap();
    }

    longlink
}

pub fn getall() -> Vec<String> {
    let db = Connection::open("./urls.sqlite").expect("Unable to open database!");
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

pub fn add_hit(shortlink: &str) -> () {
    let db = Connection::open("./urls.sqlite").expect("Unable to open database!");
    db.execute(
        "UPDATE urls SET hits = hits + 1 WHERE short_url = ?1",
        [shortlink],
    )
    .unwrap();
}

pub fn add_link(shortlink: String, longlink: String) -> bool {
    let db = Connection::open("./urls.sqlite").expect("Unable to open database!");

    match db.execute(
        "INSERT INTO urls (long_url, short_url, hits) VALUES (?1, ?2, ?3)",
        (longlink, shortlink, 0),
    ) {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn delete_link(shortlink: String) -> () {
    let db = Connection::open("./urls.sqlite").expect("Unable to open database!");
    db.execute("DELETE FROM urls WHERE short_url = ?1", [shortlink])
        .unwrap();
}
