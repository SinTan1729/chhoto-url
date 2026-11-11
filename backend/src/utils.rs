// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_web::cookie::Key;
use argon2::{Params, PasswordHash};
use log::{info, warn};
use regex::Regex;
use std::{
    convert::TryFrom, env::var, fs, io, path::Path, sync::Arc, thread::available_parallelism,
};
use tokio::sync::Semaphore;

use crate::{
    Semaphores,
    auth::{self, Credential::Argon2},
    config::Config,
};

fn copy_frontend(src: &str, dst: &str) -> io::Result<()> {
    fn copy_contents(src: &Path, dst: &Path) -> io::Result<()> {
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let from = entry.path();
            let to = dst.join(entry.file_name());

            if entry.file_type()?.is_dir() {
                fs::create_dir(&to)?;
                copy_contents(&from, &to)?;
            } else if entry.file_type()?.is_file() {
                match fs::hard_link(&from, &to) {
                    Ok(()) => {}
                    Err(e) if e.kind() == io::ErrorKind::CrossesDevices => {
                        fs::copy(&from, &to)?;
                    }
                    Err(e) => return Err(e),
                }
            }
        }

        Ok(())
    }

    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(dst)? {
        let path = entry?.path();

        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }

    copy_contents(Path::new(src), Path::new(dst))
}

fn change_title(title: &str, src: &str, dst: &str) -> io::Result<()> {
    let mut html = fs::read_to_string(format!("{src}/index.html"))?;
    let re1 = Regex::new(r#"<span>(\s?)Chhoto URL<\/span>"#).unwrap();
    let re2 = Regex::new(r#"<title>Chhoto URL<\/title>"#).unwrap();
    let re3 = Regex::new(r#"<meta\s+property="og:title"\s+content="[^"]*"( />|>)"#).unwrap();

    html = re1
        .replace(&html, format!("<span>${{1}}{title}</span>"))
        .to_string();
    html = re2
        .replace(
            &html,
            format!("<title>{title} - Powered by Chhoto URL</title>"),
        )
        .to_string();
    html = re3
        .replace(
            &html,
            format!(r#"<meta property="og:title" content="{title} - Powered by Chhoto URL"$1"#),
        )
        .to_string();

    fs::write(format!("{dst}/index.html.tmp"), html)?;
    fs::rename(format!("{dst}/index.html.tmp"), format!("{dst}/index.html"))?;
    Ok(())
}

pub(crate) fn get_frontend_location() -> (String, String) {
    let (src, dst) = if Path::new("./frontend/base/index.html").is_file() {
        ("./frontend/base", "./frontend/final")
    } else {
        ("./frontend", "./frontend-final")
    };
    (src.to_string(), dst.to_string())
}

pub(crate) fn apply_custom_title(title: &Option<String>) -> io::Result<String> {
    let (src, dst) = get_frontend_location();
    let Some(title) = title else {
        let _ = fs::remove_dir(Path::new(&dst));
        return Ok(src.to_string());
    };

    copy_frontend(&src, &dst)?;
    change_title(title, &src, &dst)?;

    Ok(dst.to_string())
}

pub(crate) fn is_custom_landing_directory_valid(loc: &str) -> bool {
    let result: Option<bool> = (|| {
        fs::read_dir(loc).ok()?.try_fold(true, |_, entry| {
            let path = entry.ok()?.path();
            let name = path.file_name()?.to_str()?;

            if path.is_dir() && ["oidc", "api", "admin"].contains(&name) {
                warn!("Forbidden directory: {}", name);
                return Some(false);
            }
            if path.is_file() && name != "index.html" {
                warn!("Forbidden file: {}", name);
                return Some(false);
            }
            Some(true)
        })
    })();

    result.unwrap_or(false)
}

pub(crate) fn generate_key() -> Key {
    let key = match var("CHHOTO_SESSION_MASTER_KEY")
        .ok()
        .map(|value| value.trim().to_owned())
    {
        Some(value) => {
            let key = base64_url::decode(value.as_bytes())
                .ok()
                .filter(|b| b.len() >= 64);

            match key {
                Some(key) => {
                    info!("Using provided session master key.");
                    Some(Key::from(&key))
                }
                None => {
                    warn!(
                        "Provided session master key is invalid. \
                        Generated secure key which you may use: {}",
                        auth::gen_key()
                    );
                    None
                }
            }
        }
        None => None,
    };

    key.unwrap_or(Key::generate())
}

fn max_argon2_concurrency(hash: &PasswordHash) -> usize {
    use std::{fs, path::Path};

    const APP_HEADROOM: u64 = 50 * 1024 * 1024;
    const UNLIMITED_THRESHOLD: u64 = 1 << 60;

    let params = Params::try_from(hash).expect("Invalid Argon2 parameters");
    let required = (params.block_count() as u64) * 1024;

    let read = |path: &Path| -> Option<u64> { fs::read_to_string(path).ok()?.trim().parse().ok() };

    let mut available = None;
    let mut cgroup_v2 = false;

    // Cgroup v2: inspect the current cgroup and all its ancestors.
    if let Ok(content) = fs::read_to_string("/proc/self/cgroup")
        && let Some(cgroup) = content.lines().find_map(|line| line.strip_prefix("0::"))
    {
        cgroup_v2 = true;

        let root = Path::new("/sys/fs/cgroup");
        let leaf = root.join(cgroup.trim_start_matches('/'));

        for dir in leaf.ancestors() {
            if !dir.starts_with(root) {
                break;
            }
            let Ok(max) = fs::read_to_string(dir.join("memory.max")) else {
                continue;
            };
            let max = max.trim();
            if max == "max" {
                continue;
            }
            let Ok(max) = max.parse::<u64>() else {
                continue;
            };
            let Some(current) = read(&dir.join("memory.current")) else {
                continue;
            };
            let remaining = max.saturating_sub(current);
            available = Some(available.map_or(remaining, |prev: u64| prev.min(remaining)));
        }
    }

    // Cgroup v1: inspect the current cgroup and its ancestors.
    if !cgroup_v2
        && let Ok(content) = fs::read_to_string("/proc/self/cgroup")
        && let Some(cgroup) = content.lines().find_map(|line| {
            let (controllers, path) = line.split_once(':')?;
            controllers
                .split(',')
                .any(|controller| controller == "memory")
                .then_some(path)
        })
    {
        let root = Path::new("/sys/fs/cgroup/memory");
        let leaf = root.join(cgroup.trim_start_matches('/'));

        for dir in leaf.ancestors() {
            if !dir.starts_with(root) {
                break;
            }
            let Some(max) = read(&dir.join("memory.limit_in_bytes")) else {
                continue;
            };
            if max >= UNLIMITED_THRESHOLD {
                continue;
            }
            let Some(current) = read(&dir.join("memory.usage_in_bytes")) else {
                continue;
            };
            let remaining = max.saturating_sub(current);
            available = Some(available.map_or(remaining, |prev: u64| prev.min(remaining)));
        }
    }

    // Host memory fallback when no finite cgroup limit was found.
    let available = available.or_else(|| {
        fs::read_to_string("/proc/meminfo")
            .ok()?
            .lines()
            .find_map(|line| {
                let (key, value) = line.split_once(':')?;

                (key == "MemAvailable")
                    .then(|| value.split_whitespace().next()?.parse::<u64>().ok())
                    .flatten()
                    .map(|kb| kb * 1024)
            })
    });

    available
        .map(|available| available.saturating_sub(APP_HEADROOM))
        .map(|available| (available / required) as usize)
        .unwrap_or(1)
}

pub(crate) fn get_permit_values(config: &Config) -> Semaphores {
    let threads = available_parallelism().map(usize::from).unwrap_or(2);

    let password_argon2_concurrency = match config.password.as_ref() {
        Some(Argon2(p)) => max_argon2_concurrency(p),
        _ => threads,
    };
    let api_key_argon2_concurrency = match config.api_key.as_ref() {
        Some(Argon2(p)) => max_argon2_concurrency(p),
        _ => threads,
    };
    let argon2_concurrency = password_argon2_concurrency
        .min(api_key_argon2_concurrency)
        .min(threads);
    if argon2_concurrency == 0 {
        panic!("Not enough memory allocated for Argon2 hashing.");
    }
    println!("{argon2_concurrency}");

    Semaphores {
        auth: Arc::new(Semaphore::new(argon2_concurrency)),
        db: Arc::new(Semaphore::new(4 * threads)),
    }
}
