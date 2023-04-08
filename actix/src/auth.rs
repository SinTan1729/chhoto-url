use actix_session::Session;

pub fn validate(session: Session) -> bool {
    let token = session.get::<i32>("session-token");
    if token.is_err() {
        false
    } else if !check(token.unwrap()) {
        false
    } else {
        true
    }
}

fn check(token: Option<i32>) -> bool {
    if token.is_none() {
        false
    } else if token.unwrap() != 123 {
        false
    } else {
        true
    }
}
