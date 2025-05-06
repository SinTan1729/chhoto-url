// SPDX-FileCopyrightText: 2023 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use rusqlite::{Connection, Error};
use serde::Serialize;

// Struct for encoding a DB row
#[derive(Serialize)]
pub struct DBRow {
    shortlink: String,
    longlink: String,
    hits: i64,
}

// Find a single URL
pub fn find_url(shortlink: &str, db: &Connection, needhits: bool) -> (Option<String>, Option<i64>) {
    let query = if needhits {
        "SELECT long_url, hits FROM urls WHERE short_url = ?1"
    } else {
        "SELECT long_url FROM urls WHERE short_url = ?1"
    };
    let mut statement = db
        .prepare_cached(query)
        .expect("Error preparing SQL statement for find_url.");

    let longlink = statement
        .query_row([shortlink], |row| row.get("long_url"))
        .ok();
    let hits = statement.query_row([shortlink], |row| row.get("hits")).ok();
    (longlink, hits)
}

// Get all URLs in DB
pub fn getall(db: &Connection) -> Vec<DBRow> {
    let mut statement = db
        .prepare_cached("SELECT * FROM urls ORDER BY id ASC")
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

// Add a hit when site is visited
pub fn add_hit(shortlink: &str, db: &Connection) {
    db.execute(
        "UPDATE urls SET hits = hits + 1 WHERE short_url = ?1",
        [shortlink],
    )
    .expect("Error updating hit count.");
}

// Insert a new link
pub fn add_link(shortlink: String, longlink: String, db: &Connection) -> Result<usize, Error> {
    db.execute(
        "INSERT INTO urls (long_url, short_url, hits) VALUES (?1, ?2, ?3)",
        (longlink, shortlink, 0),
    )
}

// Delete and existing link
pub fn delete_link(shortlink: String, db: &Connection) -> bool {
    if let Ok(delta) = db.execute("DELETE FROM urls WHERE short_url = ?1", [shortlink]) {
        delta > 0
    } else {
        false
    }
}

// Open the DB, and create schema if missing
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
        // expiry_time is added later during migration 1
        [],
    )
    .expect("Unable to initialize empty database.");

    // Create index on short_url for faster lookups
    db.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_short_url ON urls (short_url)",
        [],
    )
    .expect("Unable to create index on short_url.");

    let current_user_version: u32 = db
        .query_row_and_then("SELECT user_version FROM pragma_user_version", [], |row| {
            row.get(0)
        })
        .unwrap_or_default();

    // Migration 1: Add expiry_time, introduced in 5.9.0
    if current_user_version < 1 {
        db.execute("ALTER TABLE urls ADD COLUMN expiry_time INTEGER", [])
            .expect("Unable to apply migration 1.");
    }

    // Set current user_version. Should be incremented on change of schema.
    db.execute("PRAGMA user_version = 1", [])
        .expect("Unable to set user_version.");

    db
}
