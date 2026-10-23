// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use std::{
    fmt::Write as _,
    io::{BufWriter, IsTerminal, Write},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
        mpsc::{Receiver, SyncSender, TrySendError, sync_channel},
    },
    thread::spawn,
    time::Duration,
};

use chrono::Local;
use env_filter::Filter;
use log::{Level, Log, Metadata, Record};

#[derive(Clone)]
struct AsyncLoggerParams {
    poll_interval: Duration,
    queue_capacity: usize,
    avg_log_line_bytes: usize,
    filter: Filter,
    color: bool,
}

pub(crate) struct AsyncStderrLogger {
    params: AsyncLoggerParams,
    tx: Mutex<SyncSender<String>>,
    dropped: Arc<AtomicU64>,
}

impl AsyncStderrLogger {
    fn with_filter(params: AsyncLoggerParams) -> (Self, Receiver<String>, Arc<AtomicU64>) {
        let (tx, rx) = sync_channel(params.queue_capacity);
        let dropped = Arc::new(AtomicU64::new(0));
        (
            Self {
                params,
                tx: Mutex::new(tx),
                dropped: Arc::clone(&dropped),
            },
            rx,
            dropped,
        )
    }
}

impl Log for AsyncStderrLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.params.filter.enabled(metadata)
    }

    fn log(&self, record: &Record) {
        if !self.params.filter.matches(record) {
            return;
        }
        let mut line = String::with_capacity(self.params.avg_log_line_bytes);
        format_line(&mut line, record, self.params.color);
        let tx = self
            .tx
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Err(TrySendError::Full(_)) = tx.try_send(line) {
            // Writer thread is behind: drop instead of blocking the worker.
            self.dropped.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn flush(&self) {}
}

fn format_line(out: &mut String, record: &Record, color: bool) {
    if color {
        out.push_str("\x1b[2m");
    }
    out.push('[');
    if color {
        out.push_str("\x1b[0m");
    }
    let _ = write!(out, "{}", Local::now().format("%Y-%m-%d %H:%M:%S%Z"));
    out.push(' ');
    let level = record.level().as_str();
    if color {
        out.push_str(level_ansi(record.level()));
    }
    out.push_str(level);
    for _ in level.len()..5 {
        out.push(' ');
    }
    if color {
        out.push_str("\x1b[0m");
    }
    out.push(' ');
    out.push_str(record.target());
    if color {
        out.push_str("\x1b[2m");
    }
    out.push_str("] ");
    if color {
        out.push_str("\x1b[0m");
    }
    let _ = write!(out, "{}", record.args());
}

fn level_ansi(level: Level) -> &'static str {
    match level {
        Level::Error => "\x1b[31m",
        Level::Warn => "\x1b[33m",
        Level::Info => "\x1b[32m",
        Level::Debug => "\x1b[34m",
        Level::Trace => "\x1b[36m",
    }
}

fn run_writer(rx: Receiver<String>, dropped: Arc<AtomicU64>, params: AsyncLoggerParams) {
    let stderr = std::io::stderr();
    let mut out = BufWriter::with_capacity(
        params.avg_log_line_bytes * params.queue_capacity,
        stderr.lock(),
    );
    let mut reported = 0_u64;
    let max_batch_size: usize = (params.queue_capacity / 8).max(32);

    loop {
        let mut wrote = false;
        match rx.recv_timeout(params.poll_interval) {
            Ok(line) => {
                if out.write_all(line.as_bytes()).is_err() || out.write_all(b"\n").is_err() {
                    break;
                }
                wrote = true;

                for _ in 1..max_batch_size {
                    match rx.try_recv() {
                        Ok(line) => {
                            if out.write_all(line.as_bytes()).is_err()
                                || out.write_all(b"\n").is_err()
                            {
                                return;
                            }
                            wrote = true;
                        }
                        Err(_) => break,
                    }
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }

        let current = dropped.load(Ordering::Relaxed);
        if current > reported {
            let message = format!(
                "{} log lines dropped (stderr writer behind)",
                current - reported
            );
            let args = format_args!("{}", message);
            let record = Record::builder()
                .args(args)
                .level(Level::Warn)
                .target("chhoto_url::logging")
                .build();

            let mut line = String::with_capacity(params.avg_log_line_bytes);
            format_line(&mut line, &record, params.color);
            if out.write_all(line.as_bytes()).is_err() || out.write_all(b"\n").is_err() {
                break;
            }

            reported = current;
            wrote = true;
        }
        if wrote && out.flush().is_err() {
            break;
        }
    }
    let _ = out.flush();
}

pub fn init_async(
    poll_interval: Duration,
    queue_capacity: usize,
    avg_log_line_bytes: usize,
) -> std::thread::JoinHandle<()> {
    let mut builder = env_filter::Builder::new();
    builder.parse(
        std::env::var("RUST_LOG")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or("warn,chhoto_url=info,actix_session::middleware=error".to_owned())
            .as_str(),
    );

    let filter = builder.build();
    let max_level = filter.filter();
    let color = std::io::stderr().is_terminal();
    let params = AsyncLoggerParams {
        poll_interval,
        queue_capacity,
        avg_log_line_bytes,
        filter,
        color,
    };
    let (logger, rx, dropped) = AsyncStderrLogger::with_filter(params.clone());

    log::set_boxed_logger(Box::new(logger)).expect("Install logger once");
    log::set_max_level(max_level);

    spawn(move || run_writer(rx, dropped, params))
}
