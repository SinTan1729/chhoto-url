// SPDX-FileCopyrightText: 2023 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use log::{debug, error, info};
use rusqlite::{fallible_iterator::FallibleIterator, named_params, Connection};
use serde::Serialize;
use std::rc::Rc;

use crate::services::ChhotoError::{self, ClientError, ServerError};

// Some constants
const APPLICATION_ID: u32 = 0x63686874; // Hex for chht, MUST NEVER BE CHANGED
const USER_VERSION: u32 = 3; // Should be incremented on change of schema

// Struct for encoding a DB row
#[derive(Serialize)]
pub struct DBRow {
    shortlink: String,
    longlink: String,
    hits: i64,
    expiry_time: i64,
    notes: String,
}

// Struct for creating a get query
struct QueryHelper {
    prefix: String,
    joins: String,
    conditions: String,
    suffix: String,
}

// Find a single URL for /api/expand
pub fn find_url(
    shortlink: &str,
    db: &Connection,
) -> Result<(String, i64, i64, String), ChhotoError> {
    // Long link, hits, expiry time
    let now = chrono::Utc::now().timestamp();
    let query = "SELECT long_url, hits, expiry_time, notes FROM urls
                 WHERE short_url = :short
                 AND (expiry_time = 0 OR expiry_time > :now)";
    let Ok(mut statement) = db.prepare_cached(query) else {
        error!("Error preparing SQL statement for find_url.");
        return Err(ServerError);
    };
    statement
        .query_row(named_params! {":short": shortlink, ":now": now}, |row| {
            Ok((
                row.get("long_url")?,
                row.get("hits")?,
                row.get("expiry_time")?,
                row.get("notes")?,
            ))
        })
        .map_err(|_| ChhotoError::ClientError {
            reason: "The shortlink does not exist on the server!".to_string(),
        })
}

// Get all URLs in DB
pub fn getall(
    db: &Connection,
    page_after: Option<&str>,
    page_no: Option<i64>,
    page_size: Option<i64>,
    filter: Option<String>,
) -> Rc<[DBRow]> {
    let now = chrono::Utc::now().timestamp();
    let size = page_size.unwrap_or(10);

    let mut params: Vec<(&str, &dyn rusqlite::ToSql)> = vec![(":now", &now)];

    let (position, page); // Needed to circumvent lifetime issues
    let mut query_helper = if let Some(pos) = page_after {
        position = pos;
        params.push((":pos", &position));
        params.push((":size", &size));
        QueryHelper {
            prefix: "( SELECT t.id, t.short_url, t.long_url, t.hits, t.expiry_time, t.notes FROM urls AS t".to_string(),
            joins: "JOIN urls AS u ON u.short_url = :pos".to_string(),
            conditions: "WHERE t.id < u.id AND ( t.expiry_time = 0 OR t.expiry_time > :now".to_string(),
            suffix: ") ORDER BY t.id DESC LIMIT :size ) as t".to_string(),
        }
    } else if let Some(num) = page_no {
        page = (num - 1) * size;
        params.push((":page", &page));
        params.push((":size", &size));
        QueryHelper {
            prefix: "( SELECT t.id, t.short_url, t.long_url, t.hits, t.expiry_time, t.notes FROM urls AS t".to_string(),
            joins: String::new(),
            conditions: "WHERE ( t.expiry_time = 0 OR t.expiry_time > :now )".to_string(),
            suffix: "ORDER BY t.id DESC LIMIT :size OFFSET :page ) as t".to_string(),
        }
    } else if page_size.is_some() {
        params.push((":size", &size));
        QueryHelper {
            prefix: "( SELECT t.id, t.short_url, t.long_url, t.hits, t.expiry_time, t.notes FROM urls AS t".to_string(),
            joins: String::new(),
            conditions: "WHERE ( t.expiry_time = 0 OR t.expiry_time > :now )".to_string(),
            suffix: "ORDER BY t.id DESC LIMIT :size ) as t".to_string(),
        }
    } else {
        QueryHelper {
            prefix: "urls AS t".to_string(),
            joins: String::new(),
            conditions: "WHERE ( t.expiry_time = 0 OR t.expiry_time > :now )".to_string(),
            suffix: String::new(),
        }
    };

    let filter_string; // Needed to circumvent lifetime issues
    if let Some(fil) = filter {
        filter_string = fil;
        query_helper
            .joins
            .push_str(" JOIN urls_fts AS f ON t.id = f.rowid");
        query_helper
            .conditions
            .push_str(" AND urls_fts MATCH :filter");
        params.push((":filter", &filter_string));
    }

    let query = format!(
        "SELECT t.short_url, t.long_url, t.hits, t.expiry_time, t.notes FROM {} {} {} {} ORDER BY t.id ASC",
        query_helper.prefix, query_helper.joins, query_helper.conditions, query_helper.suffix,
    );
    let Ok(mut statement) = db.prepare_cached(&query) else {
        error!("Error preparing SQL statement for getall.");
        return [].into();
    };
    let raw_data = statement.query(params.as_slice());

    let Ok(data) = raw_data else {
        error!("Error running SQL statement for getall: {query}");
        return [].into();
    };

    let links: Rc<[DBRow]> = data
        .map(|row| {
            Ok(DBRow {
                shortlink: row.get("short_url")?,
                longlink: row.get("long_url")?,
                hits: row.get("hits")?,
                expiry_time: row.get("expiry_time")?,
                notes: row.get("notes").unwrap_or_default(),
            })
        })
        .collect()
        .unwrap_or_else(|err| {
            error!("Error processing fetched rows: {err}");
            [].into()
        });

    links
}

// Add a hit when site is visited during link resolution
pub fn find_and_add_hit(shortlink: &str, db: &Connection) -> Result<String, ()> {
    let now = chrono::Utc::now().timestamp();
    let Ok(mut statement) = db.prepare_cached(
        "UPDATE urls 
             SET hits = hits + 1 
             WHERE short_url = :short AND (expiry_time = 0 OR expiry_time > :now)
             RETURNING long_url",
    ) else {
        error!("Error preparing SQL statement for add_hit.");
        return Err(());
    };
    statement
        .query_one(named_params! {":short": shortlink, ":now": now}, |row| {
            row.get("long_url")
        })
        .map_err(|_| ())
}

// Insert a new link
pub fn add_link(
    shortlink: &str,
    longlink: &str,
    expiry_delay: i64,
    notes: &str,
    db: &Connection,
) -> Result<i64, ChhotoError> {
    let now = chrono::Utc::now().timestamp();
    let expiry_time = if expiry_delay == 0 {
        0
    } else {
        now + expiry_delay
    };

    let Ok(mut statement) = db.prepare_cached(
        "INSERT INTO urls
             (long_url, short_url, hits, expiry_time, notes)
             VALUES (:long, :short, 0, :expiry, :notes)
             ON CONFLICT(short_url) DO UPDATE 
             SET long_url = :long, hits = 0, expiry_time = :expiry, notes = :notes
             WHERE short_url = :short AND expiry_time <= :now AND expiry_time > 0",
    ) else {
        error!("Error preparing SQL statement for add_link.");
        return Err(ServerError);
    };
    match statement.execute(
        named_params! {":long": longlink, ":short": shortlink, ":expiry": expiry_time, ":now": now, ":notes" : notes},
    ) {
        Ok(1) => Ok(expiry_time),
        Ok(_) => Err(ClientError {
            reason: "Short URL is already in use!".to_string(),
        }),
        Err(e) => {
            error!("There was some error while adding the link ({shortlink}, {longlink}, {expiry_delay}): {e}");
            Err(ServerError)
        }
    }
}

// Edit an existing link
pub fn edit_link(
    shortlink: &str,
    longlink: &str,
    reset_hits: bool,
    expiry_time: Option<i64>,
    notes: Option<&str>,
    db: &Connection,
) -> Result<usize, ()> {
    let now = chrono::Utc::now().timestamp();
    let mut params: Vec<(&str, &dyn rusqlite::ToSql)> =
        vec![(":long", &longlink), (":short", &shortlink), (":now", &now)];

    let hits_query = if reset_hits { ", hits = 0" } else { "" };
    let (note, expiry); // Needed to circumvent lifetime issues
    let notes_query = if let Some(n) = notes {
        note = n;
        params.push((":notes", &note));
        ", notes = :notes"
    } else {
        ""
    };
    let expiry_query = if let Some(exp) = expiry_time {
        expiry = exp;
        params.push((":expiry", &expiry));
        ", expiry_time = :expiry"
    } else {
        ""
    };

    let query = format!(
        "UPDATE urls
         SET long_url = :long{hits_query}{expiry_query}{notes_query}
         WHERE short_url = :short AND (expiry_time = 0 OR expiry_time > :now)"
    );
    let Ok(mut statement) = db.prepare_cached(&query) else {
        error!("Error preparing SQL statement for edit_link.");
        return Err(());
    };

    statement
        .execute(params.as_slice())
        .inspect_err(|err| {
            error!(
                "Got an error while editing link ({shortlink}, {longlink}, {reset_hits}): {err}"
            );
        })
        .map_err(|_| ())
}

// Clean expired links
pub fn cleanup(db: &Connection, use_wal_mode: bool) {
    let now = chrono::Utc::now().timestamp();
    debug!("Starting database cleanup.");

    let mut statement = db
        .prepare_cached("DELETE FROM urls WHERE :now >= expiry_time AND expiry_time > 0")
        .expect("Error preparing SQL statement for cleanup.");
    statement
        .execute(named_params! {":now" : now})
        .inspect(|&u| match u {
            0 => (),
            1 => info!("1 expired link was deleted."),
            _ => info!("{u} expired links were deleted."),
        })
        .expect("Error cleaning expired links.");

    if use_wal_mode {
        let mut pragma_statement = db
            .prepare_cached("PRAGMA wal_checkpoint(TRUNCATE)")
            .expect("Error preparing SQL statement for pragma: wal_checkpoint.");
        pragma_statement
            .query_one([], |row| row.get::<usize, isize>(1))
            .ok()
            .filter(|&v| v != -1)
            .expect("Unable to create WAL checkpoint.");
    }
    let mut pragma_statement = db
        .prepare_cached("PRAGMA optimize")
        .expect("Error preparing SQL statement for pragma: optimize.");
    pragma_statement
        .execute([])
        .expect("Unable to optimize database.");
    debug!("Optimized database.")
}

// Delete an existing link
pub fn delete_link(shortlink: &str, db: &Connection) -> Result<(), ChhotoError> {
    let Ok(mut statement) = db.prepare_cached("DELETE FROM urls WHERE short_url = :short") else {
        error!("Error preparing SQL statement for delete_link.");
        return Err(ServerError);
    };
    match statement.execute(named_params! {":short" : shortlink}) {
        Ok(delta) if delta > 0 => Ok(()),
        _ => Err(ClientError {
            reason: "The shortlink was not found, and could not be deleted.".to_string(),
        }),
    }
}

// Open DB, and apply migrations if necessary
pub fn open_db(path: &str, use_wal_mode: bool, ensure_acid: bool) -> Connection {
    let db = Connection::open(path).expect("Unable to open database!");

    let tables_list: Rc<[String]> = {
        let mut statement = db
            .prepare(
                "SELECT name
                 FROM sqlite_master
                 WHERE type = 'table' AND name NOT LIKE 'sqlite_%'
                 ORDER BY name",
            )
            .expect("Error preparing statement for listing tables.");
        statement
            .query_map([], |row| row.get("name"))
            .unwrap()
            .filter_map(Result::ok)
            .collect()
    };

    let urls_table_exists = tables_list.iter().any(|s| s.as_str() == "urls");
    let urls_fts_table_exists = tables_list.iter().any(|s| s.as_str() == "urls_fts");

    let current_user_version: u32 = if !urls_table_exists {
        // It would mean that the table is newly created i.e. has the desired schema
        USER_VERSION
    } else {
        db.query_row_and_then("SELECT user_version FROM pragma_user_version", [], |row| {
            row.get(0)
        })
        .unwrap_or_default()
    };

    let current_application_id: u32 = db
        .query_row_and_then(
            "SELECT application_id FROM pragma_application_id",
            [],
            |row| row.get(0),
        )
        .unwrap_or_default();
    if current_application_id > 0 || (urls_table_exists && current_user_version > 1) {
        assert_eq!(
            current_application_id, APPLICATION_ID,
            "Incorrect application_id: The database file seems to belong to some other application."
        )
    }

    // Create table if it doesn't exist
    db.execute(
        "CREATE TABLE IF NOT EXISTS urls (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            long_url TEXT NOT NULL,
            short_url TEXT NOT NULL,
            hits INTEGER NOT NULL,
            expiry_time INTEGER NOT NULL DEFAULT 0,
            notes TEXT
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

    // Migration 1: Add expiry_time, introduced in 6.0.0
    if current_user_version < 1 {
        db.execute(
            "ALTER TABLE urls ADD COLUMN expiry_time INTEGER NOT NULL DEFAULT 0",
            [],
        )
        .expect("Unable to apply migration 1.");
    }

    // Migration 2: Add notes, introduced in 7.0.0
    if current_user_version < 3 {
        db.execute("ALTER TABLE urls ADD COLUMN notes TEXT", [])
            .expect("Unable to apply migration 2.");
    }

    // Create FTS5 table if it doesn't exist, and also create triggers
    db.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS urls_fts USING fts5(
             long_url, short_url, notes,
             content='urls',
             content_rowid='id',
             tokenize='trigram remove_diacritics 2'
         )",
        [],
    )
    .expect("Unable to create FTS table.");
    if !urls_fts_table_exists {
        db.execute("INSERT INTO urls_fts(urls_fts) VALUES ('rebuild')", [])
            .expect("Unable to populate FTS table.");
        let fts_triggers = [
            "CREATE TRIGGER IF NOT EXISTS urls_insert
                 AFTER INSERT ON urls BEGIN
                 INSERT INTO urls_fts(rowid, long_url, short_url, notes)
                 VALUES (new.id, new.long_url, new.short_url, new.notes);
             END",
            "CREATE TRIGGER IF NOT EXISTS urls_delete
                 AFTER DELETE ON urls BEGIN
                 INSERT INTO urls_fts(urls_fts, rowid, long_url, short_url, notes)
                 VALUES('delete', old.id, old.long_url, old.short_url, old.notes);
             END",
            "CREATE TRIGGER IF NOT EXISTS urls_update
             AFTER UPDATE ON urls BEGIN
                 INSERT INTO urls_fts(urls_fts, rowid, long_url, short_url, notes)
                 VALUES('delete', old.id, old.long_url, old.short_url, old.notes);
                 INSERT INTO urls_fts(rowid, long_url, short_url, notes)
                 VALUES (new.id, new.long_url, new.short_url, new.notes);
             END",
        ];
        for trigger in fts_triggers {
            db.execute(trigger, [])
                .expect("Unable to create FTS trigger(s).");
        }
    }

    // The migrations have finished successfully by this point
    if !urls_table_exists || current_user_version < USER_VERSION {
        db.pragma_update(None, "user_version", USER_VERSION)
            .expect("Unable to set pragma: user_version.");
        db.pragma_update(None, "application_id", APPLICATION_ID)
            .expect("Unable to set pragma: application_id.");
    }

    // Create index on expiry_time for faster lookups
    db.execute(
        "CREATE INDEX IF NOT EXISTS idx_expiry_time ON urls (expiry_time)",
        [],
    )
    .expect("Unable to create index on expiry_time.");

    // Set WAL mode if specified
    let (journal_mode, synchronous) = match (use_wal_mode, ensure_acid) {
        (true, false) => ("WAL", "NORMAL"),
        (true, true) => ("WAL", "FULL"),
        (false, false) => ("DELETE", "FULL"),
        (false, true) => ("DELETE", "EXTRA"),
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
