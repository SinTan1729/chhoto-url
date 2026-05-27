// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use log::{debug, error};
use rusqlite::{Connection, fallible_iterator::FallibleIterator, named_params};
use serde::Serialize;
use std::rc::Rc;

use crate::{
    database::queries,
    services::types::ChhotoError::{self, ClientError, ServerError},
};

// Struct for encoding a DB row
#[derive(Serialize)]
pub(crate) struct DBRow {
    shortlink: String,
    pub(crate) longlink: String,
    pub(crate) hits: i64,
    pub(crate) expiry_time: i64,
    pub(crate) notes: String,
}

// Find a single URL for /api/expand
pub(crate) fn find_url(shortlink: &str, db: &Connection) -> Result<DBRow, ChhotoError> {
    // Long link, hits, expiry time
    let now = chrono::Utc::now().timestamp();
    let Ok(mut statement) = db.prepare_cached(queries::FIND_URL) else {
        error!("Error preparing SQL statement for find_url.");
        return Err(ServerError);
    };
    statement
        .query_row(named_params! {":short": shortlink, ":now": now}, |row| {
            Ok(DBRow {
                shortlink: String::new(),
                longlink: row.get("long_url")?,
                hits: row.get("hits")?,
                expiry_time: row.get("expiry_time").unwrap_or_default(),
                notes: row.get("notes").unwrap_or_default(),
            })
        })
        .inspect(|_| {
            debug!("Expanded link: {shortlink}.");
        })
        .map_err(|_| ChhotoError::ClientError {
            reason: "The shortlink does not exist on the server!".to_string(),
        })
}

// Get all URLs in DB
pub(crate) fn getall(
    db: &Connection,
    page_after: Option<&str>,
    page_no: Option<i64>,
    page_size: Option<i64>,
    filter: Option<String>,
) -> Rc<[DBRow]> {
    let now = chrono::Utc::now().timestamp();

    let has_cursor = page_after.is_some();
    let has_filter = filter.is_some();
    let paginated = has_cursor || page_no.is_some();

    let size = page_size.unwrap_or(if paginated { 10 } else { -1 });
    let offset = page_no.map(|n| (n - 1) * size).unwrap_or(0);

    let (query, params) = match (has_cursor, has_filter) {
        (false, false) => (
            queries::GETALL_QUERIES[0],
            named_params! {
                ":now": now,
                ":size": size,
                ":offset": offset,
            },
        ),
        (true, false) => (
            queries::GETALL_QUERIES[1],
            named_params! {
                ":now": now,
                ":size": size,
                ":pos": page_after,
            },
        ),
        (false, true) => (
            queries::GETALL_QUERIES[2],
            named_params! {
                ":now": now,
                ":size": size,
                ":offset": offset,
                ":filter": filter,
            },
        ),
        (true, true) => (
            queries::GETALL_QUERIES[3],
            named_params! {
                ":now": now,
                ":size": size,
                ":pos": page_after,
                ":filter": filter,
            },
        ),
    };

    let Ok(mut statement) = db.prepare_cached(query) else {
        error!("Error preparing SQL statement for getall.");
        return [].into();
    };

    let raw_data = statement.query(params);

    let Ok(data) = raw_data else {
        error!("Error running SQL statement for getall.");
        return [].into();
    };

    let links: Rc<[DBRow]> = data
        .map(|row| {
            Ok(DBRow {
                shortlink: row.get("short_url")?,
                longlink: row.get("long_url")?,
                hits: row.get("hits")?,
                expiry_time: row.get("expiry_time").unwrap_or_default(),
                notes: row.get("notes").unwrap_or_default(),
            })
        })
        .collect()
        .unwrap_or_else(|err| {
            error!("Error processing fetched rows: {err}");
            [].into()
        });

    debug!(
        "Path getall was accessed with page_no: {:?}, page_after: {:?}, page_size: {:?}, filter: {:?}",
        page_no, page_after, page_size, filter
    );
    links
}

// Add a hit when site is visited during link resolution
pub(crate) fn find_and_add_hit(shortlink: &str, db: &Connection) -> Result<String, ()> {
    let now = chrono::Utc::now().timestamp();
    let Ok(mut statement) = db.prepare_cached(queries::FIND_AND_ADD_HIT) else {
        error!("Error preparing SQL statement for add_hit.");
        return Err(());
    };
    statement
        .query_one(named_params! {":short": shortlink, ":now": now}, |row| {
            row.get("long_url")
        })
        .inspect(|_| {
            debug!("Accessed link: {shortlink}.");
        })
        .map_err(drop)
}

// Insert a new link
pub(crate) fn add_link(
    shortlink: &str,
    longlink: &str,
    expiry_delay: Option<i64>,
    notes: Option<&str>,
    db: &Connection,
) -> Result<i64, ChhotoError> {
    let now = chrono::Utc::now().timestamp();
    let expiry_time = expiry_delay.map(|delay| now + delay);

    let Ok(mut statement) = db.prepare_cached(queries::ADD_LINK) else {
        error!("Error preparing SQL statement for add_link.");
        return Err(ServerError);
    };
    match statement.execute(named_params! {":long": longlink, ":short": shortlink,
    ":expiry": expiry_time, ":now": now, ":notes" : notes})
    {
        Ok(1) => {
            debug!(
                "Added link with shortlink: {}, longlink: {}, expiry_delay: {:?}, notes: {:?}",
                shortlink, longlink, expiry_delay, notes
            );
            Ok(expiry_time.unwrap_or_default())
        }
        Ok(_) => {
            debug!("Duplicate insertion attempted for {shortlink}.");
            Err(ClientError {
                reason: "Short URL is already in use!".to_string(),
            })
        }
        Err(e) => {
            error!(
                "There was some error while adding the link ({shortlink}, {longlink}, {:?}): {e}",
                expiry_delay
            );
            Err(ServerError)
        }
    }
}

// Edit an existing link
pub(crate) fn edit_link(
    shortlink: &str,
    longlink: &str,
    reset_hits: bool,
    expiry_time: Option<i64>,
    notes: Option<&str>,
    db: &Connection,
) -> Result<usize, ()> {
    let now = chrono::Utc::now().timestamp();
    let Ok(mut statement) = db.prepare_cached(queries::EDIT_LINK) else {
        error!("Error preparing SQL statement for edit_link.");
        return Err(());
    };
    statement
        .execute(named_params! {
            ":long": longlink,
            ":short": shortlink,
            ":now": now,
            ":hits": reset_hits.then_some(0),
            ":notes": notes,
            ":expiry": expiry_time,
        })
        .inspect_err(|err| {
            error!(
                "Got an error while editing link ({shortlink}, {longlink}, {reset_hits}): {err}"
            );
        })
        .inspect(|_| {
            debug!(
                "Link {} was edited using longlink: {}, reset_hits: {}, expiry_time: {:?}, notes: {:?}.",
                shortlink, longlink, reset_hits, expiry_time, notes
            );
        })
        .map_err(drop)
}

// Delete an existing link
pub(crate) fn delete_link(shortlink: &str, db: &Connection) -> Result<(), ChhotoError> {
    let Ok(mut statement) = db.prepare_cached(queries::DELETE_LINK) else {
        error!("Error preparing SQL statement for delete_link.");
        return Err(ServerError);
    };
    match statement.execute(named_params! {":short" : shortlink}) {
        Ok(delta) if delta > 0 => {
            debug!("Deleted link {shortlink}.");
            Ok(())
        }
        _ => Err(ClientError {
            reason: "The shortlink was not found, and could not be deleted.".to_string(),
        }),
    }
}
