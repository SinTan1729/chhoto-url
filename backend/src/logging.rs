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

use env_filter::Filter;
use jiff::ZonedDateTime;
use log::{Level, Log, Metadata, Record};

#[derive(Clone)]
struct AsyncLoggerParams {
    poll_interval: Duration,
    queue_capacity: usize,
    avg_log_line_bytes: usize,
    filter: Arc<Filter>,
    color: bool,
}

enum WriterMessage {
    Record(LogMessage),
    Flush(SyncSender<()>),
}
struct LogMessage {
    timestamp: ZonedDateTime,
    level: Level,
    target: String,
    message: String,
}
pub(crate) struct AsyncStderrLogger {
    params: AsyncLoggerParams,
    tx: SyncSender<WriterMessage>,
    dropped: Arc<AtomicU64>,
    disconnected_reported: AtomicBool,
}

impl AsyncStderrLogger {
    fn with_filter(params: AsyncLoggerParams) -> (Self, Receiver<WriterMessage>, Arc<AtomicU64>) {
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
            timestamp: ZonedDateTime::now(),
            level: record.level(),
            target: record.target().to_owned(),
            message: record.args().to_string(),
        };

        match self.tx.try_send(WriterMessage::Record(message)) {
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

    fn flush(&self) {
        let (ack_tx, ack_rx) = sync_channel(1);
        if self.tx.send(WriterMessage::Flush(ack_tx)).is_ok() {
            let _ = ack_rx.recv();
        }
    }
}

fn write_buffer(
    stderr: &std::io::Stderr,
    records: &mut Vec<LogMessage>,
    output: &mut Vec<u8>,
    line: &mut String,
    color: bool,
    flush: bool,
) -> std::io::Result<()> {
    output.clear();
    if records.is_empty() && !flush {
        return Ok(());
    }

    for record in records.iter() {
        line.clear();
        format_line(line, record, color);
        output.extend_from_slice(line.as_bytes());
        output.push(b'\n');
    }

    let mut stderr = stderr.lock();

    if !output.is_empty() {
        stderr.write_all(output)?;
    }

    if flush {
        stderr.flush()?;
    }

    records.clear();
    Ok(())
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
        record.timestamp.strftime("%Y-%m-%d %H:%M:%S%.3f%z")
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

fn run_writer(
    rx: Receiver<WriterMessage>,
    dropped: Arc<AtomicU64>,
    params: AsyncLoggerParams,
) -> std::io::Result<()> {
    let stderr = std::io::stderr();

    let mut reported = 0_u64;
    let max_batch_size: usize = (params.queue_capacity / 8).max(32);
    let drain_budget = Duration::from_millis(50);

    let mut buffer = Vec::with_capacity(max_batch_size);
    let mut output = Vec::with_capacity(
        params
            .avg_log_line_bytes
            .saturating_add(1)
            .saturating_mul(max_batch_size),
    );
    let mut line = String::with_capacity(params.avg_log_line_bytes);

    loop {
        let poll_deadline = Instant::now() + params.poll_interval;
        let mut batch_size = 0;
        let mut flush_ack = None;
        let mut disconnected = false;

        // Wait until the polling interval expires, the batch fills,
        // or a flush request arrives.
        while batch_size < max_batch_size {
            let remaining = poll_deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                break;
            }

            match rx.recv_timeout(remaining) {
                Ok(WriterMessage::Record(record)) => {
                    buffer.push(record);
                    batch_size += 1;
                }

                Ok(WriterMessage::Flush(ack)) => {
                    flush_ack = Some(ack);
                    break;
                }

                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => break,

                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    disconnected = true;
                    break;
                }
            }
        }

        // Drain remaining queued messages, respecting the batch
        // size and drain budget.
        let deadline = Instant::now() + drain_budget;
        while flush_ack.is_none() && !disconnected && batch_size < max_batch_size {
            match rx.try_recv() {
                Ok(WriterMessage::Record(record)) => {
                    buffer.push(record);
                    batch_size += 1;
                }

                Ok(WriterMessage::Flush(ack)) => {
                    flush_ack = Some(ack);
                    break;
                }

                Err(std::sync::mpsc::TryRecvError::Empty) => break,

                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    disconnected = true;
                    break;
                }
            }

            if Instant::now() >= deadline {
                break;
            }
        }

        // Append dropped-message warning.
        let current = dropped.load(Ordering::Relaxed);
        if current > reported {
            buffer.push(LogMessage {
                timestamp: ZonedDateTime::now(),
                level: Level::Warn,
                target: "chhoto_url::logging".to_owned(),
                message: format!(
                    "{} log lines dropped (stderr writer behind)",
                    current - reported,
                ),
            });

            reported = current;
        }

        let flush = flush_ack.is_some() || disconnected;
        write_buffer(
            &stderr,
            &mut buffer,
            &mut output,
            &mut line,
            params.color,
            flush,
        )?;

        if let Some(ack) = flush_ack {
            let _ = ack.send(());
        }
        if disconnected {
            return Ok(());
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

    spawn(move || {
        if let Err(err) = run_writer(rx, dropped, params) {
            eprintln!("Async logger writer failed: {err}");
        }
    })
}
