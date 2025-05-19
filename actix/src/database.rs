// SPDX-FileCopyrightText: 2023 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use log::info;
use rusqlite::{Connection, Error};
use serde::Serialize;

// Struct for encoding a DB row
#[derive(Serialize)]
pub struct DBRow {
    shortlink: String,
    longlink: String,
    hits: i64,
    expiry_time: i64,
}

// Find a single URL
pub fn find_url(
    shortlink: &str,
    db: &Connection,
    needhits: bool,
) -> (Option<String>, Option<i64>, Option<i64>) {
    let now = chrono::Utc::now().timestamp();
    let query = if needhits {
        "SELECT long_url, hits, expiry_time FROM urls WHERE short_url = ?1 AND (expiry_time > ?2 OR expiry_time = 0)"
    } else {
        "SELECT long_url FROM urls WHERE short_url = ?1 AND (expiry_time > ?2 OR expiry_time = 0)"
    };
    let mut statement = db
        .prepare_cached(query)
        .expect("Error preparing SQL statement for find_url.");
    statement
        .query_row((shortlink, now), |row| {
            let longlink = row.get("long_url").ok();
            let hits = row.get("hits").ok();
            let expiry_time = row.get("expiry_time").ok();
            Ok((longlink, hits, expiry_time))
        })
        .unwrap_or_default()
}

// Get all URLs in DB
pub fn getall(db: &Connection) -> Vec<DBRow> {
    let now = chrono::Utc::now().timestamp();
    let mut statement = db
        .prepare_cached(
            "SELECT * FROM urls WHERE expiry_time > ?1 OR expiry_time = 0 ORDER BY id ASC",
        )
        .expect("Error preparing SQL statement for getall.");

    let mut data = statement
        .query([now])
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
            expiry_time: row.get("expiry_time").unwrap_or_default(),
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
pub fn add_link(
    shortlink: String,
    longlink: String,
    expiry_delay: i64,
    db: &Connection,
) -> Result<i64, Error> {
    let now = chrono::Utc::now().timestamp();
    let expiry_time = if expiry_delay == 0 {
        0
    } else {
        now + expiry_delay
    };

    db.execute(
        "INSERT INTO urls (long_url, short_url, hits, expiry_time) VALUES (?1, ?2, ?3, ?4)",
        (longlink, shortlink, 0, expiry_time),
    )
    .map(|_| expiry_time)
}

// Clean expired links
pub fn cleanup(db: &Connection) {
    let now = chrono::Utc::now().timestamp();

    let mut statement = db
        .prepare_cached("SELECT short_url FROM urls WHERE ?1 > expiry_time AND expiry_time > 0")
        .expect("Error preparing SQL statement for cleanup.");

    let mut data = statement
        .query([now])
        .expect("Error executing query for cleanup.");

    while let Some(row) = data.next().expect("Error reading fetched rows.") {
        let shortlink: String = row
            .get("short_url")
            .expect("Error reading shortlink off a row.");
        info!("Expired link marked for deletion: {shortlink}");
    }

    db.execute(
        "DELETE FROM urls WHERE expiry_time < ?1 AND expiry_time > 0",
        [now],
    )
    .inspect(|&u| {
        if u > 0 {
            info!(
                "{u} expired link{} deleted.",
                if u == 1 { " was" } else { "s were" }
            )
        }
    })
    .expect("Error cleaning expired links.");
}

// Delete and existing link
pub fn delete_link(shortlink: String, db: &Connection) -> bool {
    if let Ok(delta) = db.execute("DELETE FROM urls WHERE short_url = ?1", [shortlink]) {
        delta > 0
    } else {
        false
    }
}

pub fn open_db(path: String) -> Connection {
    // Set current user_version. Should be incremented on change of schema.
    let user_version = 1;

    let db = Connection::open(path).expect("Unable to open database!");

    // It would be 0 if table does not exist, and 1 if it does
    let table_exists: usize = db
        .query_row_and_then(
            "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = 'urls'",
            [],
            |row| row.get(0),
        )
        .expect("Error querying existence of table.");

    // Create table if it doesn't exist
    db.execute(
        "CREATE TABLE IF NOT EXISTS urls (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            long_url TEXT NOT NULL,
            short_url TEXT NOT NULL,
            hits INTEGER NOT NULL,
            expiry_time INTEGER NOT NULL DEFAULT 0
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

    let current_user_version: u32 = if table_exists == 0 {
        // It would mean that the table is newly created i.e. has the desired schema
        user_version
    } else {
        db.query_row_and_then("SELECT user_version FROM pragma_user_version", [], |row| {
            row.get(0)
        })
        .unwrap_or_default()
    };

    // Migration 1: Add expiry_time, introduced in 6.0.0
    if current_user_version < 1 {
        db.execute(
            "ALTER TABLE urls ADD COLUMN expiry_time INTEGER NOT NULL DEFAULT 0",
            [],
        )
        .expect("Unable to apply migration 1.");
    }

    // Create index on expiry_time for faster lookups
    db.execute(
        "CREATE INDEX IF NOT EXISTS idx_expiry_time ON urls (expiry_time)",
        [],
    )
    .expect("Unable to create index on expiry_time.");

    // Set the user version
    db.pragma_update(None, "user_version", user_version)
        .expect("Unable to set user_version.");

    db
}
