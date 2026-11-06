// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use r2d2::{ManageConnection, Pool};
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub(crate) struct ReaderDbManager {
    path: PathBuf,
}

impl ManageConnection for ReaderDbManager {
    type Connection = Connection;
    type Error = rusqlite::Error;

    fn connect(&self) -> Result<Connection, Self::Error> {
        let conn = super::utils::open_db(&self.path, true)?;
        Ok(conn)
    }

    fn is_valid(&self, _conn: &mut Connection) -> Result<(), Self::Error> {
        Ok(())
    }

    fn has_broken(&self, _conn: &mut Connection) -> bool {
        false
    }
}

pub(crate) type DbPool = Pool<ReaderDbManager>;

pub(crate) fn open_pool(path: &Path) -> Result<DbPool, r2d2::Error> {
    let manager = ReaderDbManager {
        path: path.to_path_buf(),
    };

    Pool::builder()
        .max_size(
            2 * std::thread::available_parallelism()
                .map(usize::from)
                .unwrap_or(2) as u32,
        )
        .min_idle(Some(2))
        .connection_timeout(Duration::from_secs(5))
        .build(manager)
}
