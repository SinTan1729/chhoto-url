// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use regex::Regex;
use std::{fs, io, path::Path};

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
    let mut html = fs::read_to_string([src, "/index.html"].concat())?;
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

    fs::write([dst, "/index.html.tmp"].concat(), html)?;
    fs::rename(
        [dst, "/index.html.tmp"].concat(),
        [dst, "/index.html"].concat(),
    )?;
    Ok(())
}

pub(crate) fn init_logger() {
    env_logger::builder()
        .parse_filters(
            std::env::var("RUST_LOG")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or("warn,chhoto_url=info,actix_session::middleware=error".to_owned())
                .as_str(),
        )
        .format(|buf, record| {
            use chrono::Local;
            use env_logger::fmt::style::{AnsiColor, Style};
            use std::io::Write;

            let subtle = Style::new().fg_color(Some(AnsiColor::BrightBlack.into()));
            let level_style = buf.default_level_style(record.level());

            writeln!(
                buf,
                "{subtle}[{subtle:#}{} {level_style}{:<6}{level_style:#}{}{subtle}]{subtle:#} {}",
                Local::now().format("%Y-%m-%d %H:%M:%S%Z"),
                record.level(),
                record.module_path().unwrap_or_default(),
                record.args()
            )
        })
        .init();
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
