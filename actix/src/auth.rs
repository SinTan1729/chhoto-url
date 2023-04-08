use actix_session::Session;
use std::time::SystemTime;

pub fn validate(session: Session) -> bool {
    let token = session.get::<String>("session-token");
    if token.is_err() {
        false
    } else if !check(token.unwrap()) {
        false
    } else {
        true
    }
}

fn check(token: Option<String>) -> bool {
    if token.is_none() {
        false
    } else {
        let token_body = token.unwrap();
        let token_parts: Vec<&str> = token_body.split(";").collect();
        if token_parts.len() < 2 {
            false
        } else {
            let token_text = token_parts[0];
            let token_time = token_parts[1].parse::<u64>().unwrap_or(0);
            let time_now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Time went backwards!")
                .as_secs();
            if token_text == "valid-session-token" && time_now < token_time + 1209600 {
                // There are 1209600 seconds in 14 days
                true
            } else {
                false
            }
        }
    }
}

pub fn gen_token() -> String {
    let token_text = "valid-session-token".to_string();
    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards!")
        .as_secs();
    format!("{token_text};{time}")
}
