// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use std::{
    fmt::Write as _,
    io::{IsTerminal, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{Receiver, SyncSender, TrySendError, sync_channel},
    },
    thread::spawn,
    time::{Duration, Instant},
};

use chrono::Local;
use env_filter::Filter;
use log::{Level, Log, Metadata, Record};

#[derive(Clone)]
struct AsyncLoggerParams {
    poll_interval: Duration,
    queue_capacity: usize,
    avg_log_line_bytes: usize,
    filter: Arc<Filter>,
    color: bool,
}

struct LogMessage {
    timestamp: chrono::DateTime<Local>,
    level: Level,
    target: String,
    message: String,
}
pub(crate) struct AsyncStderrLogger {
    params: AsyncLoggerParams,
    tx: SyncSender<LogMessage>,
    dropped: Arc<AtomicU64>,
    disconnected_reported: AtomicBool,
}

impl AsyncStderrLogger {
    fn with_filter(params: AsyncLoggerParams) -> (Self, Receiver<LogMessage>, Arc<AtomicU64>) {
        let (tx, rx) = sync_channel(params.queue_capacity);
        let dropped = Arc::new(AtomicU64::new(0));
        (
            Self {
                params,
                tx,
                dropped: Arc::clone(&dropped),
                disconnected_reported: false.into(),
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

        let message = LogMessage {
            timestamp: Local::now(),
            level: record.level(),
            target: record.target().to_owned(),
            message: record.args().to_string(),
        };

        match self.tx.try_send(message) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
                self.dropped.fetch_add(1, Ordering::Relaxed);
            }
            Err(TrySendError::Disconnected(_)) => {
                if !self.disconnected_reported.swap(true, Ordering::Relaxed) {
                    eprintln!("Async logger writer disconnected; log line discarded");
                }
            }
        }
    }

    fn flush(&self) {}
}

fn format_line(out: &mut String, record: &LogMessage, color: bool) {
    if color {
        out.push_str("\x1b[2m");
    }
    out.push('[');
    if color {
        out.push_str("\x1b[0m");
    }

    let _ = write!(
        out,
        "{}",
        record.timestamp.format("%Y-%m-%d %H:%M:%S%.3f%Z")
    );
    out.push(' ');

    let level = record.level.as_str();
    if color {
        out.push_str(level_ansi(record.level));
    }
    out.push_str(level);

    for _ in level.len()..5 {
        out.push(' ');
    }
    if color {
        out.push_str("\x1b[0m");
    }
    out.push(' ');
    out.push_str(&record.target);
    if color {
        out.push_str("\x1b[2m");
    }
    out.push_str("] ");
    if color {
        out.push_str("\x1b[0m");
    }
    out.push_str(&record.message);
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

fn run_writer(rx: Receiver<LogMessage>, dropped: Arc<AtomicU64>, params: AsyncLoggerParams) {
    let stderr = std::io::stderr();

    let mut reported = 0_u64;
    let max_batch_size: usize = (params.queue_capacity / 8).max(32);
    let drain_budget = Duration::from_millis(50);
    let mut buffer = Vec::with_capacity(
        params
            .avg_log_line_bytes
            .saturating_add(1)
            .saturating_mul(max_batch_size),
    );
    let mut line = String::with_capacity(params.avg_log_line_bytes);

    loop {
        std::thread::sleep(params.poll_interval);

        let deadline = Instant::now() + drain_budget;
        let mut disconnected = false;
        while Instant::now() < deadline {
            buffer.clear();
            let mut batch_size = 0;
            while batch_size < max_batch_size && Instant::now() < deadline {
                match rx.try_recv() {
                    Ok(message) => {
                        line.clear();
                        format_line(&mut line, &message, params.color);
                        buffer.extend_from_slice(line.as_bytes());
                        buffer.push(b'\n');
                        batch_size += 1;
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => break,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        disconnected = true;
                        break;
                    }
                }
            }

            if !buffer.is_empty() {
                if Instant::now() >= deadline {
                    dropped.fetch_add(batch_size as u64, Ordering::Relaxed);
                    break;
                }
                let mut stderr = stderr.lock();
                if stderr.write_all(&buffer).is_err() {
                    return;
                }
            }

            if disconnected || batch_size == 0 {
                break;
            }
        }

        let mut discarded = 0_u64;
        for _ in 0..params.queue_capacity {
            match rx.try_recv() {
                Ok(_) => discarded += 1,
                Err(_) => break,
            }
        }

        if discarded > 0 {
            dropped.fetch_add(discarded, Ordering::Relaxed);
        }
        let current = dropped.load(Ordering::Relaxed);
        if current > reported {
            let message = LogMessage {
                timestamp: Local::now(),
                level: Level::Warn,
                target: "chhoto_url::logging".to_owned(),
                message: format!(
                    "{} log lines dropped (stderr writer behind)",
                    current - reported
                ),
            };
            buffer.clear();
            line.clear();
            format_line(&mut line, &message, params.color);
            buffer.extend_from_slice(line.as_bytes());
            buffer.push(b'\n');

            let mut stderr = stderr.lock();
            if stderr.write_all(&buffer).is_err() {
                return;
            }

            reported = current;
        }

        if disconnected {
            break;
        }
    }
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
    let filter = Arc::new(filter);
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
