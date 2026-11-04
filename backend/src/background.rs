// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use chrono::{Local, Timelike};
use log::{info, warn};
use rusqlite::Connection;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::{
    spawn,
    sync::mpsc,
    time::{Duration, Instant, interval, sleep, sleep_until},
};

use crate::database;

// Run hit updates every 500ms or once 500 distinct links are pending.
pub(crate) fn spawn_hits_worker(
    writer: Arc<Mutex<Connection>>,
    mut hits_rx: mpsc::Receiver<(String, bool)>,
) -> tokio::task::JoinHandle<()> {
    spawn({
        async move {
            let mut pending = HashMap::new();
            fn update_count(link: String, reset: bool, pending: &mut HashMap<String, i64>) {
                if reset {
                    pending.remove(&link);
                } else {
                    *pending.entry(link).or_insert(0) += 1;
                }
            }
            loop {
                let Some((first, reset)) = hits_rx.recv().await else {
                    break;
                };
                update_count(first, reset, &mut pending);
                let deadline = Instant::now() + Duration::from_millis(500);

                while pending.len() < 500 {
                    tokio::select! {
                        Some((link, reset)) = hits_rx.recv() => update_count(link, reset, &mut pending),
                        _ = sleep_until(deadline) => break,
                        else => break,
                    }
                }
                if !pending.is_empty() {
                    database::add_hits(
                        std::mem::take(&mut pending),
                        &mut writer.lock().expect(crate::DB_MUTEX_ERROR),
                    );
                }
            }
        }
    })
}

// Do database cleanup once every hour
pub(crate) fn spawn_cleaner(
    writer: Arc<Mutex<Connection>>,
    db_path: String,
    use_wal_mode: bool,
    disable_backups: bool,
) -> tokio::task::JoinHandle<()> {
    spawn({
        let writer = Arc::clone(&writer);
        async move {
            info!("Starting database cleanup service, will run once every hour.");
            let mut interval = interval(Duration::from_hours(1));
            let mut first_run = true;
            loop {
                interval.tick().await;
                let hour = Local::now().hour();

                info!("Starting database cleanup task.");
                for attempt in 0..3 {
                    let mutex = writer.clone();
                    let result = tokio::task::spawn_blocking(move || {
                        let mut db = mutex.lock().expect(crate::DB_MUTEX_ERROR);
                        database::cleanup(&mut db, use_wal_mode, first_run, attempt)
                    })
                    .await;
                    match result {
                        Ok(Ok(true)) => {
                            if attempt < 2 {
                                info!("Expired links remain; will try again in 10 seconds.");
                            } else {
                                info!(
                                    "Expired links remain; they will be tried again during a later cleanup."
                                );
                                info!("Cleanup task complete.");
                            };
                        }
                        Ok(Ok(false)) => {
                            info!("No expired links remain.");
                            break;
                        }
                        Ok(Err(e)) => {
                            warn!("Cleanup issue: {e}");
                            break;
                        }
                        Err(e) => {
                            warn!("Cleanup task failed: {e}");
                            break;
                        }
                    };
                    if attempt < 2 {
                        sleep(Duration::from_secs(10)).await;
                    }
                }
                first_run = false;
                if hour == 3 {
                    info!("Doing a scheduled daily backup.");
                    let path = db_path.clone();
                    match tokio::task::spawn_blocking(move || {
                        database::manage_backups(
                            &path,
                            disable_backups,
                            database::BackupType::Daily,
                        )
                    })
                    .await
                    {
                        Ok(Ok(())) => {}
                        Ok(Err(e)) => warn!("Scheduled backup error: {e}"),
                        Err(e) => warn!("Scheduled backup task failed: {e}"),
                    }
                }
            }
        }
    })
}
