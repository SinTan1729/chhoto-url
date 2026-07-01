// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use log::{debug, error, warn};
use rusqlite::{Connection, fallible_iterator::FallibleIterator, named_params};
use serde::Serialize;
use std::{collections::HashMap, rc::Rc};
use tokio::sync::mpsc;

use crate::{
    database::queries,
    services::types::ChhotoError::{self, ClientError, ServerError},
    utils::NewURLRequest,
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

// Resolve site and add link to add_hit queue
pub(crate) async fn find_and_add_hit(
    shortlink: &str,
    db: &Connection,
    hits_tx: &mpsc::Sender<(String, bool)>,
) -> Result<String, ()> {
    let now = chrono::Utc::now().timestamp();
    let Ok(mut statement) = db.prepare_cached(queries::FIND_LINK) else {
        error!("Error preparing SQL statement for find link.");
        return Err(());
    };
    let Ok(long_url) = statement
        .query_one(named_params! {":short": shortlink, ":now": now}, |row| {
            row.get("long_url")
        })
    else {
        return Err(());
    };

    debug!("Accessed link: {shortlink}.");
    if let Err(err) = hits_tx.send((shortlink.to_string(), false)).await {
        error!("Failed to enqueue hit update after access: {err}");
    }
    Ok(long_url)
}
// Add hits
pub(crate) fn add_hits(shortlinks: HashMap<String, i64>, db: &mut Connection) {
    let Ok(tx) = db.transaction() else {
        warn!("Unable to start a transaction for add hit.");
        return;
    };
    {
        let Ok(mut statement) = tx.prepare_cached(queries::ADD_HIT) else {
            warn!("Error preparing SQL statement for add hit.");
            return;
        };
        for (link, count) in shortlinks.iter() {
            let _ = statement
                .execute(named_params! {":short": link, ":count": count})
                .inspect_err(|e| {
                    warn!("Unable to update hit for {link}: {e}");
                });
        }
    }
    if let Err(e) = tx.commit() {
        warn!("Add hit commit failed: {e}");
        warn!(
            "Dropped a total of {} hit increments, with {} distinct links.",
            shortlinks.values().sum::<i64>(),
            shortlinks.len()
        );
    }
}

// Insert a new link
type AddLinksReturnType = Vec<(usize, Result<(String, i64), ChhotoError>)>;
pub(crate) fn add_links(
    requests: Vec<(usize, NewURLRequest)>,
    db: &mut Connection,
    return_rejected: bool,
) -> (AddLinksReturnType, Option<Vec<(usize, NewURLRequest)>>) {
    if requests.is_empty() {
        return (Vec::new(), None);
    }
    let now = chrono::Utc::now().timestamp();
    let in_use_error = ClientError {
        reason: "Short URL is already in use!".to_string(),
    };
    let mut output = Vec::with_capacity(requests.len());
    let mut rejected = Vec::with_capacity(requests.len());

    for chunk in requests.chunks(500) {
        let chunk_error = || chunk.iter().map(|(i, _)| (*i, Err(ServerError)));
        let start = output.len();
        let Ok(tx) = db.transaction() else {
            error!("Unable to start a transaction for add link.");
            output.extend(chunk_error());
            continue;
        };
        {
            let Ok(mut statement) = tx.prepare_cached(queries::ADD_LINK) else {
                error!("Error preparing SQL statement for add link.");
                output.extend(chunk_error());
                continue;
            };

            for (i, req) in chunk {
                let expiry_time = req.expiry_delay.map(|delay| now + delay);
                output.push(match statement.execute(
                named_params! {
                    ":long": req.longlink,
                    ":short": req.shortlink,
                    ":expiry": expiry_time,
                    ":now": now,
                    ":notes" : req.notes
                },
            ) {
                Ok(1) => {
                    debug!(
                        "Added link with shortlink: {}, longlink: {}, expiry_delay: {:?}, notes: {:?}",
                        req.shortlink, req.longlink, req.expiry_delay, req.notes
                    );
                    (*i, Ok((req.shortlink.to_owned(), expiry_time.unwrap_or_default())))
                }
                Ok(0) => {
                    debug!("Duplicate insertion attempted for {}.", req.shortlink);
                        if return_rejected {
                            rejected.push((*i, req.to_owned()));
                            (0, Err(ServerError)) // Placeholder; add_links_helper ignores errors in this mode.
                        } else {
                            (*i, Err(in_use_error.to_owned()))
                        }
                }
                Ok(n) => {
                    error!("Unexpected row count while adding link {}: {}", req.shortlink, n);
                    (*i, Err(ServerError))
                }
                Err(e) => {
                    error!(
                        "There was some error while adding the link ({}, {}, {:?}): {}",
                        req.shortlink, req.longlink, req.expiry_delay, e
                    );
                    (*i, Err(ServerError))
                }
            });
            }
        }
        if let Err(e) = tx.commit() {
            error!("Add link commit failed: {e}");
            output.truncate(start);
            output.extend(chunk_error());
        }
    }

    (output, Some(rejected).filter(|_| return_rejected))
}

// Edit an existing link
pub(crate) async fn edit_link(
    shortlink: &str,
    longlink: &str,
    reset_hits: bool,
    expiry_time: Option<i64>,
    notes: Option<&str>,
    hits_tx: &mpsc::Sender<(String, bool)>,
    db: &Connection,
) -> Result<usize, ()> {
    let now = chrono::Utc::now().timestamp();
    let Ok(mut statement) = db.prepare_cached(queries::EDIT_LINK) else {
        error!("Error preparing SQL statement for edit_link.");
        return Err(());
    };
    if reset_hits && let Err(err) = hits_tx.send((shortlink.to_string(), true)).await {
        error!("Failed to enqueue hit update after edit: {err}");
    }
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
