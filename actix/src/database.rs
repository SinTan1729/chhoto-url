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

// Find a single URL for /api/expand
pub fn find_url(shortlink: String, db: &Connection) -> (Option<String>, Option<i64>, Option<i64>) {
    // Long link, hits, expiry time
    let now = chrono::Utc::now().timestamp();
    let query = "SELECT long_url, hits, expiry_time FROM urls
                 WHERE short_url = ?1 
                 AND (expiry_time = 0 OR expiry_time > ?2)";
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
pub fn getall(
    db: &Connection,
    page_after: Option<String>,
    page_no: Option<i64>,
    page_size: Option<i64>,
) -> Vec<DBRow> {
    let now = chrono::Utc::now().timestamp();
    let query = if page_after.is_some() {
        "SELECT short_url, long_url, hits, expiry_time FROM (
            SELECT t.id, t.short_url, t.long_url, t.hits, t.expiry_time FROM urls AS t 
            JOIN urls AS u ON u.short_url = ?1 
            WHERE t.id < u.id AND (t.expiry_time = 0 OR t.expiry_time > ?2) 
            ORDER BY t.id DESC LIMIT ?3
         ) ORDER BY id ASC"
    } else if page_no.is_some() {
        "SELECT short_url, long_url, hits, expiry_time FROM (
            SELECT id, short_url, long_url, hits, expiry_time FROM urls 
            WHERE expiry_time= 0 OR expiry_time > ?1 
            ORDER BY id DESC LIMIT ?2 OFFSET ?3
         ) ORDER BY id ASC"
    } else if page_size.is_some() {
        "SELECT short_url, long_url, hits, expiry_time FROM (
            SELECT id, short_url, long_url, hits, expiry_time FROM urls
            WHERE expiry_time = 0 OR expiry_time > ?1 
            ORDER BY id DESC LIMIT ?2
         ) ORDER BY id ASC"
    } else {
        "SELECT short_url, long_url, hits, expiry_time
         FROM urls WHERE expiry_time = 0 OR expiry_time > ?1 
         ORDER BY id ASC"
    };
    let mut statement = db
        .prepare_cached(query)
        .expect("Error preparing SQL statement for getall.");

    let mut data = if let Some(pos) = page_after {
        let size = page_size.unwrap_or(10);
        statement
            .query((pos, now, size))
            .expect("Error executing query for getall: curson pagination.")
    } else if let Some(num) = page_no {
        let size = page_size.unwrap_or(10);
        statement
            .query((now, size, (num - 1) * size))
            .expect("Error executing query for getall: offset pagination.")
    } else if let Some(size) = page_size {
        statement
            .query((now, size))
            .expect("Error executing query for getall: offset pagination (default).")
    } else {
        statement
            .query([now])
            .expect("Error executing query for getall: no pagination.")
    };

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

// Add a hit when site is visited during link resolution
pub fn find_and_add_hit(shortlink: String, db: &Connection) -> Option<String> {
    let now = chrono::Utc::now().timestamp();
    let mut statement = db
        .prepare_cached(
            "UPDATE urls 
             SET hits = hits + 1 
             WHERE short_url = ?1 AND (expiry_time = 0 OR expiry_time > ?2)
             RETURNING long_url",
        )
        .expect("Error preparing SQL statement for add_hit.");
    statement
        .query_one((shortlink, now), |row| row.get("long_url"))
        .ok()
}

// Insert a new link
pub fn add_link(
    shortlink: &str,
    longlink: &str,
    expiry_delay: i64,
    db: &Connection,
) -> Option<i64> {
    let now = chrono::Utc::now().timestamp();
    let expiry_time = if expiry_delay == 0 {
        0
    } else {
        now + expiry_delay
    };

    let mut statement = db
        .prepare_cached(
            "INSERT INTO urls
             (long_url, short_url, hits, expiry_time)
             VALUES (?1, ?2, 0, ?3)
             ON CONFLICT(short_url) DO UPDATE 
             SET long_url = ?1, hits = 0, expiry_time = ?3 
             WHERE short_url = ?2 AND expiry_time <= ?4 AND expiry_time > 0",
        )
        .expect("Error preparing SQL statement for add_link.");
    let delta = statement
        .execute((longlink, shortlink, expiry_time, now))
        .expect("There was an unexpected error while inserting link.");
    if delta == 1 {
        Some(expiry_time)
    } else {
        None
    }
}

// Edit an existing link
pub fn edit_link(
    shortlink: &str,
    longlink: &str,
    reset_hits: bool,
    db: &Connection,
) -> Result<usize, Error> {
    let now = chrono::Utc::now().timestamp();
    let query = if reset_hits {
        "UPDATE urls 
         SET long_url = ?1, hits = 0 
         WHERE short_url = ?2 AND (expiry_time = 0 OR expiry_time > ?3)"
    } else {
        "UPDATE urls 
         SET long_url = ?1 
         WHERE short_url = ?2 AND (expiry_time = 0 OR expiry_time > ?3)"
    };
    let mut statement = db
        .prepare_cached(query)
        .expect("Error preparing SQL statement for edit_link.");
    statement.execute((longlink, shortlink, now))
}

// Clean expired links
pub fn cleanup(db: &Connection) {
    let now = chrono::Utc::now().timestamp();
    info!("Starting database cleanup.");

    let mut statement = db
        .prepare_cached("DELETE FROM urls WHERE ?1 >= expiry_time AND expiry_time > 0")
        .expect("Error preparing SQL statement for cleanup.");
    statement
        .execute([now])
        .inspect(|&u| match u {
            0 => (),
            1 => info!("1 link was deleted."),
            _ => info!("{u} links were deleted."),
        })
        .expect("Error cleaning expired links.");

    let mut pragma_statement = db
        .prepare_cached("PRAGMA optimize")
        .expect("Error preparing SQL statement for pragma optimize.");
    pragma_statement
        .execute([])
        .expect("Unable to optimize database");
    info!("Optimized database.")
}

// Delete an existing link
pub fn delete_link(shortlink: String, db: &Connection) -> bool {
    let mut statement = db
        .prepare_cached("DELETE FROM urls WHERE short_url = ?1")
        .expect("Error preparing SQL statement for delete_link.");
    if let Ok(delta) = statement.execute([shortlink]) {
        delta > 0
    } else {
        false
    }
}

pub fn open_db(path: String, use_wal_mode: bool) -> Connection {
    // Set current user_version. Should be incremented on change of schema.
    let user_version = 1;

    let db = Connection::open(path).expect("Unable to open database!");

    // It would be 0 if table does not exist, and 1 if it does
    let table_exists: usize = db
        .query_row_and_then(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'urls'",
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
        .expect("Unable to set pragma: user_version.");
    // Set WAL mode if specified
    let (journal_mode, synchronous) = if use_wal_mode {
        ("WAL", "NORMAL")
    } else {
        ("DELETE", "FULL")
    };
    db.pragma_update(None, "journal_mode", journal_mode)
        .expect("Unable to set pragma: journal_mode.");
    db.pragma_update(None, "synchronous", synchronous)
        .expect("Unable to set pragma: synchronous.");
    // Set some further optimizations and run vacuum
    db.pragma_update(None, "temp_store", "memory")
        .expect("Unable to set pragma: temp_store.");
    db.pragma_update(None, "journal_size_limit", "8388608")
        .expect("Unable to set pragma: journal_size_limit.");
    db.pragma_update(None, "mmap_size", "16777216")
        .expect("Unable to set pragma: mmap_size.");
    db.execute("VACUUM", []).expect("Unable to vacuum database");
    db.execute("PRAGMA optimize=0x10002", [])
        .expect("Error running pragma optimize.");

    db
}
