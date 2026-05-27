// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use chrono::{Local, Timelike, Utc};
use log::{debug, error, info};
use rusqlite::{Connection, named_params};
use std::{collections::HashSet, fs, path::PathBuf};

use crate::database::queries;

// Some constants
const APPLICATION_ID: i32 = i32::from_be_bytes(*b"chht"); // MUST NEVER BE CHANGED
const USER_VERSION: u32 = 4; // Should be incremented on change of schema

// Enum for backup types
enum BackupType {
    Daily,
    Init,
}

// Clean expired links
pub(crate) fn cleanup(db: &Connection, use_wal_mode: bool) {
    let now = Utc::now().timestamp();
    debug!("Starting database cleanup.");

    if Local::now().hour() == 3 {
        info!("Doing a scheduled daily backup.");
        manage_backups(db, BackupType::Daily);
    }

    db.prepare_cached(queries::CLEANUP)
        .expect("Error preparing SQL statement for cleanup.")
        .execute(named_params! {":now" : now})
        .inspect(|&u| match u {
            0 => (),
            1 => info!("1 expired link was deleted."),
            _ => info!("{u} expired links were deleted."),
        })
        .expect("Error cleaning expired links.");

    if use_wal_mode {
        let mut statement = db
            .prepare_cached("PRAGMA wal_checkpoint(PASSIVE)")
            .expect("Error preparing SQL statement for pragma: wal_checkpoint.");
        statement
            .query_one((), |row| row.get::<usize, isize>(1))
            .ok()
            .filter(|&v| v != -1)
            .expect("Unable to create WAL checkpoint.");
    }
    let freelist_count: i64 = db
        .prepare_cached("PRAGMA freelist_count")
        .expect("Error preparing SQL statement for pragma: freelist_count")
        .query_row((), |r| r.get(0))
        .expect("failed to get freelist_count");

    // Roughly 20 MB with 4 KiB pages
    if freelist_count > 5000 {
        db.prepare_cached("VACUUM")
            .expect("Error preparing SQL statement for vacuum.")
            .execute(())
            .expect("failed to vacuum database");
    }
    db.prepare_cached("PRAGMA optimize")
        .expect("Error preparing SQL statement for pragma: optimize.")
        .execute(())
        .expect("Unable to optimize database.");
    debug!("Optimized database.")
}

// Create backups
fn manage_backups(db: &Connection, backup_type: BackupType) {
    let path = db.path().expect("The database path should exist.");

    let db_path = PathBuf::from(path);

    let parent = db_path
        .parent()
        .expect("Database should have a parent directory.");

    let db_name = db_path
        .file_name()
        .expect("Database should have a file name.")
        .to_string_lossy();

    let backup_dir = parent.join("backups");

    if let Err(e) = fs::create_dir_all(&backup_dir) {
        error!("Failed to create backup directory: {e}");
        return;
    }

    let (suffix, retain) = match backup_type {
        BackupType::Init => {
            info!("Creating an init backup of the database.");
            ("init", 3)
        }
        BackupType::Daily => {
            info!("Creating a daily backup of the database.");
            ("daily", 7)
        }
    };

    let backup_path = |idx: usize| backup_dir.join(format!("{db_name}.{suffix}{idx}"));

    if let Err(e) = db.backup("main", backup_path(0).to_string_lossy().as_ref(), None) {
        error!("There was an error while creating the backup: {e}");
        return;
    }

    // Migrate legacy backups
    if matches!(backup_type, BackupType::Daily) {
        for path in fs::read_dir(parent).into_iter().flatten().flatten() {
            let path = path.path();
            if path
                .file_name()
                .and_then(|f| f.to_str())
                .is_some_and(|f| f.starts_with(&format!("{db_name}.bak")))
            {
                let filename = path.file_name().unwrap().to_string_lossy();
                let daily_name = filename.replacen(".bak", ".daily", 1);
                if let Err(e) = fs::rename(&path, backup_dir.join(daily_name)) {
                    error!("There was an error migrating legacy backup: {e}");
                }
            }
        }
    }

    for i in (0..retain).rev() {
        if let Err(e) = fs::rename(backup_path(i), backup_path(i + 1))
            && e.kind() != std::io::ErrorKind::NotFound
        {
            error!("Failed to rotate backup {i}: {e}");
        }
    }
}

// Initialize the database
pub(crate) fn initialize_db(path: &str, use_wal_mode: bool, ensure_acid: bool) {
    let mut db = Connection::open(path).expect("Unable to open database!");
    manage_backups(&db, BackupType::Init);

    info!("Initializing database.");
    let (mut tables, mut indices) = db
        .prepare(queries::TABLE_LIST)
        .expect("Error preparing statement for database objects query.")
        .query_map((), |row| {
            Ok((row.get::<_, String>("type")?, row.get::<_, String>("name")?))
        })
        .expect("Error executing database objects query.")
        .filter_map(Result::ok)
        .fold(
            (HashSet::new(), HashSet::new()),
            |(mut tables, mut indices), (obj_type, name)| {
                match obj_type.as_str() {
                    "table" => {
                        tables.insert(name);
                    }
                    "index" => {
                        indices.insert(name);
                    }
                    _ => {}
                }

                (tables, indices)
            },
        );

    let urls_table_exists = tables.contains("urls");
    let urls_fts_table_exists = tables.contains("urls_fts");

    let current_user_version: u32 = if urls_table_exists {
        db.query_row_and_then("SELECT user_version FROM pragma_user_version", (), |row| {
            row.get(0)
        })
        .unwrap_or_default()
    } else {
        USER_VERSION
    };

    let current_application_id: i32 = db
        .query_row_and_then(
            "SELECT application_id FROM pragma_application_id",
            (),
            |row| row.get(0),
        )
        .unwrap_or_default();
    if current_application_id != 0
        || (urls_table_exists && urls_fts_table_exists && current_user_version > 1)
    {
        assert_eq!(
            current_application_id, APPLICATION_ID,
            "Incorrect application_id: The database file seems to belong to some other application."
        )
    } else {
        db.pragma_update(None, "application_id", APPLICATION_ID)
            .expect("Unable to set pragma: application_id.");
    }
    // Create table if it doesn't exist
    if !urls_table_exists {
        info!("Creating an empty urls table.");
        db.execute(
            queries::URLS_TABLE_SCHEMA,
            // expiry_time is added later during migration 1
            (),
        )
        .expect("Unable to initialize empty database.");
    }

    // Migration 1: Add expiry_time, introduced in 6.0.0
    if current_user_version < 1 {
        info!("Applying migration 1: Add expiry_time column to urls.");
        let tx = db
            .transaction()
            .expect("Unable to create transaction for migration 1.");
        tx.execute(
            "ALTER TABLE urls ADD COLUMN expiry_time INTEGER NOT NULL DEFAULT 0",
            (),
        )
        .expect("Unable to apply migration 1.");
        tx.pragma_update(None, "user_version", 1)
            .expect("Unable to set pragma: user_version.");
        tx.commit()
            .expect("Unable to commit transaction for migration 1.");
    }
    // Migration 2: Add notes, introduced in 7.0.0
    if current_user_version < 3 {
        info!("Applying migration 2: Add notes column to urls.");
        let tx = db
            .transaction()
            .expect("Unable to create transaction for migration 2.");
        tx.execute("ALTER TABLE urls ADD COLUMN notes TEXT", ())
            .expect("Unable to apply migration 2.");
        tx.pragma_update(None, "user_version", 2)
            .expect("Unable to set pragma: user_version.");
        tx.commit()
            .expect("Unable to commit transaction for migration 2.");
    }
    // Migration 3: Remove AUTOINCREMENT from the id row
    if current_user_version < 4 {
        info!("Applying migration 3: Remove AUTOINCREMENT from id row.");
        let tx = db
            .transaction()
            .expect("Unable to create transaction for migration 2.");
        tx.execute("ALTER TABLE urls RENAME TO urls_old", ())
            .expect("Unable to temporarily rename urls to urls_old.");
        tx.execute(queries::URLS_TABLE_SCHEMA, ())
            .expect("Unable to create new urls table.");
        tx.execute(queries::URLS_MIGRATION_3, ())
            .expect("Unable to clone data to the new table.");
        tx.execute("DROP TABLE urls_old", ())
            .expect("Unable to delete old urls table.");
        if urls_fts_table_exists {
            tx.execute("DROP TABLE urls_fts", ())
                .expect("Unable to delete old urls_fts table.");
        }
        (tables, indices) = (HashSet::from(["urls".to_string()]), HashSet::new());
        tx.pragma_update(None, "user_version", 3)
            .expect("Unable to set pragma: user_version.");
        tx.commit()
            .expect("Unable to commit transaction for migration 3.");
        db.execute("VACUUM", ())
            .expect("failed to vacuum database after migration 3.");
    }

    // Create index on short_url for faster lookups
    if !indices.contains("idx_short_url") {
        info!("Creating index idx_short_url on urls(short_url).");
        db.execute("CREATE UNIQUE INDEX idx_short_url ON urls (short_url)", ())
            .expect("Unable to create index on short_url.");
    }

    // Create index on expiry_time for faster lookups
    if !indices.contains("idx_expiry_time") {
        info!("Creating index idx_expiry_time on urls(expiry_time).");
        db.execute("CREATE INDEX idx_expiry_time ON urls (expiry_time)", ())
            .expect("Unable to create index on expiry_time.");
    }

    // Create FTS5 table if it doesn't exist, and also create triggers
    if !tables.contains("urls_fts") {
        info!("Creating FTS table urls_fts, and adding triggers.");
        let tx = db
            .transaction()
            .expect("Unable to create transaction for FTS table creation.");
        tx.execute(queries::FTS_TABLE_SCHEMA, ())
            .expect("Unable to create FTS table.");

        tx.execute("INSERT INTO urls_fts(urls_fts) VALUES ('rebuild')", ())
            .expect("Unable to populate FTS table.");
        for trigger in queries::FTS_TRIGGERS {
            tx.execute(trigger, ())
                .expect("Unable to create FTS trigger(s).");
        }

        tx.commit().expect("Unable to create FTS table.");
    }

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
    db.pragma_update(None, "temp_store", "memory")
        .expect("Unable to set pragma: temp_store.");
    let tx = db
        .transaction()
        .expect("Unable to create transaction for pragma updates.");
    // Set some further optimizations and run vacuum if necessary
    tx.pragma_update(None, "journal_size_limit", "8388608")
        .expect("Unable to set pragma: journal_size_limit.");
    tx.pragma_update(None, "mmap_size", "16777216")
        .expect("Unable to set pragma: mmap_size.");
    // The schema should be up-to-date by this point
    tx.pragma_update(None, "user_version", USER_VERSION)
        .expect("Unable to set pragma: user_version.");
    tx.commit().expect("Unable to set correct pragma values.");
    db.execute("PRAGMA optimize=0x10002", ())
        .expect("Unable to optimize database.");

    info!("Database initialization was successful.");
}

// Open and return a rusqlite connection
pub(crate) fn open_db(path: &str) -> Connection {
    Connection::open(path).expect("Unable to open database.")
}
