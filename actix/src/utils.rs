use crate::database;
use actix_web::web;
use regex::Regex;

pub fn get_longurl(shortlink: web::Path<String>) -> String {
    if validate_link(&shortlink) {
        database::find_url(shortlink.as_str())
    } else {
        "".to_string()
    }
}

fn validate_link(link: &str) -> bool {
    let re = Regex::new("[a-z0-9-_]+").unwrap();
    re.is_match(link)
}

pub fn getall() -> String {
    let links = database::getall();
    links.join("\n")
}
