use sqlite::{open, Row};

pub fn find_url(shortlink: &str) -> String {
    let db = open("./urls.sqlite").expect("Unable to open database!");

    let query = "SELECT long_url FROM urls WHERE short_url = ?";

    let statement: Vec<Row> = db
        .prepare(query)
        .unwrap()
        .into_iter()
        .bind((1, shortlink))
        .unwrap()
        .map(|row| row.unwrap())
        .collect();

    let mut longlink = "";
    if statement.len() == 1 {
        longlink = statement[0].read::<&str, _>("long_url");
    }

    String::from(longlink)
}
