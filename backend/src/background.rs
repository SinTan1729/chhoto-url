// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use log::info;
use rusqlite::Connection;
use std::{collections::HashMap, sync::Arc};
use tokio::{
    spawn,
    sync::{Mutex, mpsc},
    time::{Duration, Instant, interval, sleep_until},
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
                    database::add_hits(std::mem::take(&mut pending), &mut *writer.lock().await);
                }
            }
        }
    })
}

// Do database cleanup once every hour
pub(crate) fn spawn_cleaner(
    writer: Arc<Mutex<Connection>>,
    use_wal_mode: bool,
) -> tokio::task::JoinHandle<()> {
    spawn({
        let writer = Arc::clone(&writer);
        async move {
            info!("Starting database cleanup service, will run once every hour.");
            let mut interval = interval(Duration::from_secs(3600));
            loop {
                interval.tick().await;
                database::cleanup(&*writer.lock().await, use_wal_mode);
            }
        }
    })
}
