// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use chrono::Utc;
use log::{debug, info, warn};
use rusqlite::{Connection, OpenFlags, backup::NO_PROGRESS, named_params};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use time::{OffsetDateTime, macros::format_description};

use crate::database::{self, queries};

// Some constants
const APPLICATION_ID: i32 = i32::from_be_bytes(*b"chht"); // MUST NEVER BE CHANGED
const USER_VERSION: u32 = 5; // Should be incremented on change of schema

// Enum for backup types
pub(crate) enum BackupType {
    Daily,
    Init,
}

// Clean expired links
pub(crate) fn cleanup(
    db: &mut Connection,
    use_wal_mode: bool,
    no_time_budget: bool,
) -> Result<(), String> {
    let start = Instant::now();
    let now = Utc::now().timestamp();

    debug!("Starting database cleanup.");
    let deadline = Instant::now() + Duration::from_millis(500);
    let mut total_deleted = 0;
    let tx = db
        .transaction()
        .map_err(|_| "Error opening SQL transaction".to_string())?;
    {
        let mut statement = tx
            .prepare_cached(queries::CLEANUP_BATCH)
            .map_err(|_| "Error preparing SQL statement.".to_string())?;
        while Instant::now() < deadline || no_time_budget {
            let deleted = statement
                .execute(named_params! { ":now": now })
                .map_err(|_| "Error cleaning expired links.".to_string())?;

            total_deleted += deleted;

            if deleted < 500 {
                break;
            }
        }
    }
    tx.commit()
        .map_err(|_| "Error committinng transaction.".to_string())?;

    match total_deleted {
        0 => info!("No expired links were found."),
        1 => info!("1 expired link was deleted."),
        n => info!("{n} expired links were deleted."),
    }

    let leftovers = db
        .query_row(
            queries::CLEANUP_LEFTOVERS,
            named_params! { ":now": now },
            |row| row.get(0),
        )
        .map_err(|_| "Error checking for expired links.".to_string())?;

    if leftovers {
        info!("Expired links remain; they will be tried again during a later cleanup.");
    }

    if use_wal_mode
        && matches!(
            db.query_one("PRAGMA wal_checkpoint(RESTART)", (), |row| {
                row.get::<usize, isize>(1)
            }),
            Err(_) | Ok(-1)
        )
    {
        return Err("Unable to create WAL checkpoint.".to_string());
    }
    let freelist_count = db
        .query_one("PRAGMA freelist_count", (), |r| r.get::<usize, i64>(0))
        .map_err(|_| "Failed to get freelist_count.".to_string())?;

    // Roughly 20 MB with 4 KiB pages
    if freelist_count > 5000 && db.execute("VACUUM", ()).is_err() {
        return Err("failed to vacuum database".to_string());
    }
    let link_count: i64 = db
        .query_one("SELECT COUNT(id) FROM urls", (), |row| row.get(0))
        .unwrap_or_default();
    if link_count > 2000 && db.execute("PRAGMA optimize", ()).is_err() {
        Err("Unable to optimize database.".to_string())
    } else {
        debug!("Optimized database.");
        debug!(
            "Cleanup completed successfully in {:.3} ms.",
            start.elapsed().as_secs_f32() * 1000.0
        );
        Ok(())
    }
}

// Create backups
pub(crate) fn manage_backups(
    db_link: &str,
    disable_backups: bool,
    backup_type: BackupType,
) -> Result<(), String> {
    if disable_backups {
        return Ok(());
    }

    let start = Instant::now();
    let db_path = PathBuf::from(db_link);
    let parent = db_path
        .parent()
        .expect("Database should have a parent directory.");
    let db_name = db_path
        .file_name()
        .expect("Database should have a file name.")
        .to_string_lossy();

    let backup_dir = parent.join("backups");

    fs::create_dir_all(&backup_dir)
        .map_err(|_| "Failed to create backup directory.".to_string())?;

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

    let db = database::open_db(db_link, true);
    db.backup::<_, _, _>(
        "main",
        backup_dir
            .join(format!(
                "{db_name}.{suffix}.{}",
                OffsetDateTime::now_utc()
                    .format(format_description!(
                        "[year]-[month]-[day]T[hour]-[minute]-[second]Z"
                    ))
                    .expect("Failed to format backup timestamp.")
            ))
            .to_string_lossy()
            .as_ref(),
        NO_PROGRESS,
    )
    .map_err(|_| "There was an error while creating the backup.".to_string())?;

    fn matching_backups(dir: &Path, prefix: &str) -> std::io::Result<Vec<PathBuf>> {
        let mut paths = fs::read_dir(dir)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with(prefix))
            })
            .map(|path| Ok((path.metadata()?.modified()?, path)))
            .collect::<std::io::Result<Vec<_>>>()?;

        paths.sort_by_key(|(mtime, _)| *mtime);

        Ok(paths.into_iter().map(|(_, path)| path).collect())
    }
    let backups = matching_backups(&backup_dir, &format!("{db_name}.{suffix}"))
        .map_err(|_| "Error getting old backups.")?;
    for path in backups.iter().rev().skip(retain) {
        fs::remove_file(path).map_err(|_| "Error cleaning old backups.")?;
    }

    debug!(
        "Backup completed successfully in {:.3} ms.",
        start.elapsed().as_secs_f32() * 1000.0
    );
    Ok(())
}

// Initialize the database
pub(crate) fn init_db(
    db: &mut Connection,
    use_wal_mode: bool,
    ensure_acid: bool,
    disable_backups: bool,
) {
    if let Some(db_path) = db.path() {
        if let Err(e) = manage_backups(db_path, disable_backups, BackupType::Init) {
            warn!("Init backup error: {e}")
        };
    } else {
        warn!("Failed to do init database backup.");
    }

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
                match &*obj_type {
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
        (tables, indices) = (HashSet::from(["urls".to_owned()]), HashSet::new());
        tx.pragma_update(None, "user_version", 3)
            .expect("Unable to set pragma: user_version.");
        tx.commit()
            .expect("Unable to commit transaction for migration 3.");
        db.execute("VACUUM", ())
            .expect("failed to vacuum database after migration 3.");
    }
    // Migration 4: Change expiry_time column type and add enabled column,
    // introduced in version 7.10.0
    if current_user_version < 5 {
        info!("Applying migration 4: Change expiry column type and add enabled column.");
        let tx = db
            .transaction()
            .expect("Unable to create transaction for migration 4.");
        tx.execute(
            "ALTER TABLE urls ADD COLUMN enabled BOOL NOT NULL DEFAULT True",
            (),
        )
        .expect("Unable to apply migration 4.");
        tx.execute_batch(
            "ALTER TABLE urls ADD COLUMN expiry_temp DATETIME;
             UPDATE urls SET expiry_temp = expiry_time;
             DROP INDEX idx_expiry_time;
             ALTER TABLE urls DROP COLUMN expiry_time;
             ALTER TABLE urls RENAME COLUMN expiry_temp TO expiry_time;
             CREATE INDEX idx_expiry_time ON urls (expiry_time)",
        )
        .expect("Unable to apply migration 4.");
        tx.pragma_update(None, "user_version", 5)
            .expect("Unable to set pragma: user_version.");
        tx.commit()
            .expect("Unable to commit transaction for migration 4.");
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
    // Create partial index on enabled for faster lookups
    if !indices.contains("idx_enabled") {
        info!("Creating index idx_enabled on urls(enabled).");
        db.execute(
            "CREATE INDEX idx_enabled ON urls (enabled) WHERE enabled == TRUE",
            (),
        )
        .expect("Unable to create index on enabled.");
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
    let link_count: i64 = db
        .query_one("SELECT COUNT(id) FROM urls", (), |row| row.get(0))
        .unwrap_or_default();
    if link_count > 2000 {
        db.execute("PRAGMA optimize=0x10002", ())
            .expect("Unable to optimize database.");
        debug!("Optimized database.");
    } else {
        debug!("Using bootstrap stat1 table");
        db.execute_batch(queries::Z_STAT1_INIT)
            .expect("Error inserting bootstrap table.");
    }

    info!("Database initialization was successful.");
}

// Open and return a rusqlite connection
pub(crate) fn open_db(path: &str, read_only: bool) -> Connection {
    if read_only {
        let flags = OpenFlags::SQLITE_OPEN_READ_ONLY
            | OpenFlags::SQLITE_OPEN_URI
            | OpenFlags::SQLITE_OPEN_NO_MUTEX;
        Connection::open_with_flags(path, flags).expect("Unable to open reader database.")
    } else {
        Connection::open(path).expect("Unable to open writer database.")
    }
}

// Perform a healthcheck
pub(crate) fn is_database_healthy(db: &Connection) -> bool {
    db.query_row_and_then(
        "SELECT application_id FROM pragma_application_id",
        (),
        |row| row.get::<usize, i32>(0),
    )
    .unwrap_or_default()
        == APPLICATION_ID
}
