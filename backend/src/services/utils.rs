// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_files::NamedFile;
use actix_web::{Responder, http::StatusCode};
use log::{debug, error};
use nanoid::nanoid;
use rand::{random_range, seq::IndexedRandom};
use rusqlite::Connection;
use serde::Deserialize;
use std::env;
use url::Url;

use crate::{
    config::{Config, SlugStyle},
    database,
    services::types::{
        ChhotoError::{self, ClientError, ServerError},
        GetReqParams,
    },
};

// Struct for reading link pairs sent during API call for new link
#[derive(Deserialize)]
struct NewURLRequest {
    #[serde(default)]
    shortlink: String,
    longlink: String,
    expiry_delay: Option<i64>,
    notes: Option<String>,
}

// Struct for reading link pairs sent during API call for editing link
#[derive(Deserialize)]
struct EditURLRequest {
    shortlink: String,
    longlink: String,
    reset_hits: bool,
    expiry_time: Option<i64>,
    notes: Option<String>,
}

// Only allow safe URI schemes
#[inline]
fn is_longurl_valid(link: &str) -> bool {
    let parts = Url::parse(link);
    parts.is_ok_and(|u| ["http", "https", "ftp", "magnet"].contains(&u.scheme()))
}

// Only have a-z, 0-9, - and _ as valid characters in a shortlink
#[inline]
fn is_shortlink_valid(link: &str, allow_capital_letters: bool) -> bool {
    if allow_capital_letters {
        link.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    } else {
        link.chars()
            .all(|c| c.is_ascii_digit() || c.is_ascii_lowercase() || c == '_' || c == '-')
    }
}

// Only have a-z, 0-9, - and _ as valid characters in a shortlink
#[inline]
fn normalize_filter(link: &str) -> Option<String> {
    if link.len() < 3 {
        return None;
    }

    let mut out = String::with_capacity(link.len());
    let mut last_was_sep = true;

    for c in link.chars() {
        // Allow printable ascii chars
        if c.is_ascii_alphanumeric() {
            out.push(c);
            last_was_sep = false;
        } else if c.is_ascii_punctuation() || c == ' ' {
            if !last_was_sep {
                out.push(' ');
                last_was_sep = true;
            }
        } else {
            return None;
        }
    }

    let s = out.trim_ascii_end();
    (s.len() > 2).then(|| s.to_string())
}

// Simply get the version string
pub(crate) fn get_version() -> String {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    const GIT_COMMIT: Option<&str> = option_env!("CARGO_GIT_COMMIT");

    match GIT_COMMIT {
        Some(commit) if !commit.trim().is_empty() => {
            format!("{VERSION}-dev+{commit}")
        }
        _ => VERSION.to_string(),
    }
}

// Request the DB for all URLs
pub(super) fn getall_helper(db: &Connection, params: GetReqParams) -> Result<String, ChhotoError> {
    let page_after = match params.page_after {
        Some(s) if s.is_empty() => {
            return Err(ChhotoError::ClientError {
                reason: "Invalid page_after was supplied!".to_string(),
            });
        }
        other => other,
    };
    let page_no = match params.page_no {
        Some(n) if n <= 0 => {
            return Err(ChhotoError::ClientError {
                reason: "Invalid page_no was supplied!".to_string(),
            });
        }
        other => other,
    };
    let page_size = params.page_size.filter(|&n| n > 0);
    let filter = params
        .filter
        .map(|s| {
            normalize_filter(&s).ok_or(ChhotoError::ClientError {
                reason: "Invalid filter was supplied!".to_string(),
            })
        })
        .transpose()?;
    let links = database::getall(db, page_after.as_deref(), page_no, page_size, filter);
    serde_json::to_string(&links).map_err(|err| {
        error!("Failure during creation of json from db columns.\n{err}");
        ChhotoError::ServerError
    })
}

// Make checks and then request the DB to add a new URL entry
pub(super) fn add_link_helper(
    req: &str,
    db: &Connection,
    config: &Config,
    using_public_mode: bool,
) -> Result<(String, i64), ChhotoError> {
    // Ok : shortlink, expiry_time
    let mut chunks: NewURLRequest;
    if let Ok(json) = serde_json::from_str(req) {
        chunks = json;
    } else {
        return Err(ClientError {
            reason: "Invalid request!".to_string(),
        });
    }
    if !is_longurl_valid(&chunks.longlink) {
        return Err(ClientError {
            reason: "Unsupported URL scheme.".to_string(),
        });
    }

    let style = &config.slug_style;
    let len = config.slug_length;
    let allow_capital_letters = config.allow_capital_letters;
    let shortlink_provided = if chunks.shortlink.is_empty() {
        chunks.shortlink = gen_link(style, len, allow_capital_letters, false);
        false
    } else {
        true
    };

    // In public mode, set automatic expiry delay
    if using_public_mode && let Some(delay) = config.public_mode_expiry_delay {
        chunks.expiry_delay = Some(chunks.expiry_delay.map_or(delay, |d| d.min(delay)))
    };

    // Allow max delay of 5 years
    chunks.expiry_delay = chunks
        .expiry_delay
        .map(|d| d.clamp(0, 157784760))
        .filter(|&d| d > 0);
    chunks.notes = chunks.notes.filter(|s| !s.is_empty());

    if !shortlink_provided || is_shortlink_valid(chunks.shortlink.as_str(), allow_capital_letters) {
        match database::add_link(
            &chunks.shortlink,
            &chunks.longlink,
            chunks.expiry_delay,
            chunks.notes.as_deref(),
            db,
        ) {
            Ok(expiry_time) => Ok((chunks.shortlink, expiry_time)),
            Err(ClientError { reason }) => {
                if shortlink_provided {
                    Err(ClientError { reason })
                } else {
                    // Optionally, retry with a longer slug length
                    chunks.shortlink =
                        gen_link(style, len, allow_capital_letters, config.try_longer_slug);
                    match database::add_link(
                        &chunks.shortlink,
                        &chunks.longlink,
                        chunks.expiry_delay,
                        chunks.notes.as_deref(),
                        db,
                    ) {
                        Ok(expiry_time) => Ok((chunks.shortlink, expiry_time)),
                        Err(_) => {
                            error!("Something went wrong while adding a generated link.");
                            Err(ServerError)
                        }
                    }
                }
            }
            Err(ServerError) => Err(ServerError),
        }
    } else {
        Err(ClientError {
            reason: "Short URL is not valid!".to_string(),
        })
    }
}

// Make checks and then request the DB to edit an URL entry
pub(super) fn edit_link_helper(
    req: &str,
    db: &Connection,
    config: &Config,
) -> Result<(), ChhotoError> {
    let chunks: EditURLRequest;
    if let Ok(json) = serde_json::from_str(req) {
        chunks = json;
    } else {
        return Err(ClientError {
            reason: "Malformed request!".to_string(),
        });
    }
    if !is_shortlink_valid(&chunks.shortlink, config.allow_capital_letters) {
        return Err(ClientError {
            reason: "Invalid shortlink!".to_string(),
        });
    }
    if !is_longurl_valid(&chunks.longlink) {
        return Err(ClientError {
            reason: "Unsupported URL scheme.".to_string(),
        });
    }
    let result = database::edit_link(
        &chunks.shortlink,
        &chunks.longlink,
        chunks.reset_hits,
        chunks.expiry_time.filter(|&t| t > 0),
        chunks.notes.filter(|s| !s.is_empty()).as_deref(),
        db,
    );
    match result {
        // Zero rows returned means no updates
        Ok(0) => Err(ClientError {
            reason: "The shortlink was not found, and could not be edited.".to_string(),
        }),
        Ok(_) => Ok(()),
        Err(()) => Err(ServerError),
    }
}

// Check if link, and request DB to delete it if exists
pub(super) fn delete_link_helper(
    shortlink: &str,
    db: &Connection,
    allow_capital_letters: bool,
) -> Result<(), ChhotoError> {
    if is_shortlink_valid(shortlink, allow_capital_letters) {
        database::delete_link(shortlink, db)
    } else {
        Err(ClientError {
            reason: "The shortlink is invalid.".to_string(),
        })
    }
}

// Generate a random link using either adjective-name pair (default) of a slug or a-z, 0-9
fn gen_link(
    style: &SlugStyle,
    len: usize,
    allow_capital_letters: bool,
    try_longer_slug: bool,
) -> String {
    #[rustfmt::skip]
    static ADJECTIVES: [&str; 108] = ["admiring", "adoring", "affectionate", "agitated", "amazing", "angry", "awesome", "beautiful", 
		"blissful", "bold", "boring", "brave", "busy", "charming", "clever", "compassionate", "competent", "condescending", "confident", "cool", 
		"cranky", "crazy", "dazzling", "determined", "distracted", "dreamy", "eager", "ecstatic", "elastic", "elated", "elegant", "eloquent", "epic", 
		"exciting", "fervent", "festive", "flamboyant", "focused", "friendly", "frosty", "funny", "gallant", "gifted", "goofy", "gracious", 
		"great", "happy", "hardcore", "heuristic", "hopeful", "hungry", "infallible", "inspiring", "intelligent", "interesting", "jolly", 
		"jovial", "keen", "kind", "laughing", "loving", "lucid", "magical", "modest", "musing", "mystifying", "naughty", "nervous", "nice", 
		"nifty", "nostalgic", "objective", "optimistic", "peaceful", "pedantic", "pensive", "practical", "priceless", "quirky", "quizzical", 
		"recursing", "relaxed", "reverent", "romantic", "sad", "serene", "sharp", "silly", "sleepy", "stoic", "strange", "stupefied", "suspicious", 
		"sweet", "tender", "thirsty", "trusting", "unruffled", "upbeat", "vibrant", "vigilant", "vigorous", "wizardly", "wonderful", "xenodochial", 
		"youthful", "zealous", "zen"];
    #[rustfmt::skip]
    static NAMES: [&str; 241] = ["agnesi", "albattani", "allen", "almeida", "antonelli", "archimedes", "ardinghelli", "aryabhata", "austin", 
		"babbage", "banach", "banzai", "bardeen", "bartik", "bassi", "beaver", "bell", "benz", "bhabha", "bhaskara", "black", "blackburn", "blackwell", 
		"bohr", "booth", "borg", "bose", "bouman", "boyd", "brahmagupta", "brattain", "brown", "buck", "burnell", "cannon", "carson", "cartwright", 
		"carver", "cauchy", "cerf", "chandrasekhar", "chaplygin", "chatelet", "chatterjee", "chaum", "chebyshev", "clarke", "cohen", "colden", "cori", 
		"cray", "curie", "curran", "darwin", "davinci", "dewdney", "dhawan", "diffie", "dijkstra", "dirac", "driscoll", "dubinsky", "easley", "edison", 
		"einstein", "elbakyan", "elgamal", "elion", "ellis", "engelbart", "euclid", "euler", "faraday", "feistel", "fermat", "fermi", "feynman", "franklin", 
		"gagarin", "galileo", "galois", "ganguly", "gates", "gauss", "germain", "goldberg", "goldstine", "goldwasser", "golick", "goodall", "gould", "greider", 
		"grothendieck", "haibt", "hamilton", "hardy", "haslett", "hawking", "heisenberg", "hellman", "hermann", "herschel", "hertz", "heyrovsky", "hodgkin", 
		"hofstadter", "hoover", "hopper", "hugle", "hypatia", "ishizaka", "jackson", "jang", "jemison", "jennings", "jepsen", "johnson", "joliot", "jones", 
		"kalam", "kapitsa", "kare", "keldysh", "keller", "kepler", "khayyam", "khorana", "kilby", "kirch", "knuth", "kowalevski", "lalande", "lamarr", 
		"lamport", "leakey", "leavitt", "lederberg", "lehmann", "lewin", "lichterman", "liskov", "lovelace", "lumiere", "mahavira", "margulis", "matsumoto", 
		"maxwell", "mayer", "mccarthy", "mcclintock", "mclaren", "mclean", "mcnulty", "meitner", "mendel", "mendeleev", "meninsky", "merkle", "mestorf", 
		"mirzakhani", "montalcini", "moore", "morse", "moser", "murdock", "napier", "nash", "neumann", "newton", "nightingale", "nobel", "noether", "northcutt", 
		"noyce", "panini", "pare", "pascal", "pasteur", "payne", "perlman", "pike", "poincare", "poitras", "proskuriakova", "ptolemy", "raman", "ramanujan", 
		"rhodes", "ride", "riemann", "ritchie", "robinson", "roentgen", "rosalind", "rubin", "saha", "sammet", "sanderson", "satoshi", "shamir", "shannon", 
		"shaw", "shirley", "shockley", "shtern", "sinoussi", "snyder", "solomon", "spence", "stonebraker", "sutherland", "swanson", "swartz", "swirles", 
		"taussig", "tesla", "tharp", "thompson", "torvalds", "tu", "turing", "varahamihira", "vaughan", "vaughn", "villani", "visvesvaraya", "volhard", 
		"wescoff", "weierstrass", "wilbur", "wiles", "williams", "williamson", "wilson", "wing", "wozniak", "wright", "wu", "yalow", "yonath", "zhukovsky"];

    static CHARS_SMALL: [char; 36] = [
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
        's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
    ];
    // uppercase and lowercase characters; exclude ambiguous characters
    static CHARS_CAPITAL: [char; 58] = [
        'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T',
        'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'm',
        'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '1', '2', '3', '4', '5',
        '6', '7', '8', '9',
    ];

    match style {
        SlugStyle::Uid => {
            let slug_len = if try_longer_slug { len + 4 } else { len };
            debug!(
                "Generating a link with style: UID, length: {slug_len}, allow_capital_letters: {allow_capital_letters}"
            );
            if allow_capital_letters {
                nanoid!(slug_len, &CHARS_CAPITAL)
            } else {
                nanoid!(slug_len, &CHARS_SMALL)
            }
        }
        SlugStyle::Pair => {
            debug!("Generating a link with style: Pair.");
            let adj = ADJECTIVES
                .choose(&mut rand::rng())
                .expect("Error choosing random adjective.")
                .to_string();
            let name = NAMES
                .choose(&mut rand::rng())
                .expect("Error choosing random name.");
            if try_longer_slug {
                format!("{adj}-{name}-{:04}", random_range(0..=9999))
            } else {
                format!("{adj}-{name}")
            }
        }
    }
}

// 404 error page
pub(crate) async fn error404() -> impl Responder {
    NamedFile::open_async("./frontend/static/404.html")
        .await
        .customize()
        .with_status(StatusCode::NOT_FOUND)
}
