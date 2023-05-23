use actix_session::Session;
use std::{env, time::SystemTime};

pub fn validate(session: Session) -> bool {
    // If there's no password provided, just return true
    if env::var("password").is_err() {
        return true;
    }

    let token = session.get::<String>("session-token");
    token.is_ok() && check(token.unwrap())
}

fn check(token: Option<String>) -> bool {
    if let Some(token_body) = token {
        let token_parts: Vec<&str> = token_body.split(';').collect();
        if token_parts.len() < 2 {
            false
        } else {
            let token_text = token_parts[0];
            let token_time = token_parts[1].parse::<u64>().unwrap_or(0);
            let time_now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Time went backwards!")
                .as_secs();
            token_text == "session-token" && time_now < token_time + 1209600 // There are 1209600 seconds in 14 days
        }
    } else {
        false
    }
}

pub fn gen_token() -> String {
    let token_text = String::from("session-token");
    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards!")
        .as_secs();
    format!("{token_text};{time}")
}
