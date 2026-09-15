// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use std::{fs, io, path::Path};

fn copy_frontend() -> io::Result<()> {
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

    let (src, dst) = ("/frontend/", "/frontend-final/");
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

fn change_title(title: &str) -> io::Result<()> {
    let mut html = fs::read_to_string("/frontend/index.html")?;
    let re1 = regex::Regex::new("<title>Chhoto URL</title>").unwrap();
    let re2 = regex::Regex::new(r#"<meta\s+property="og:title"\s+content="[^"]*"( />|>)"#).unwrap();
    html = re1
        .replace(
            &html,
            format!("<title>{} - Powered by Chhoto URL</title>", title),
        )
        .to_string();
    html = re2
        .replace(
            &html,
            format!(
                r#"<meta property="og:title" content="{} - Powered by Chhoto URL"$1"#,
                title
            ),
        )
        .to_string();
    fs::write("/frontend-final/index.html", html)?;
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

pub(crate) fn apply_custom_title(title: &Option<String>) -> io::Result<String> {
    let Some(title) = title else {
        return Ok("/frontend/".to_string());
    };

    copy_frontend()?;
    change_title(title)?;

    Ok("/frontend-final/".to_string())
}
