// SPDX-FileCopyrightText: 2023 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_web::HttpRequest;
use nanoid::nanoid;
use rand::seq::IndexedRandom;
use regex::Regex;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::{auth, config::Config, database, services::GetReqParams};

// Struct for reading link pairs sent during API call for new link
#[derive(Deserialize)]
struct NewURLRequest {
    #[serde(default)]
    shortlink: String,
    longlink: String,
    #[serde(default)]
    expiry_delay: i64,
}

// Struct for reading link pairs sent during API call for editing link
#[derive(Deserialize)]
struct EditURLRequest {
    shortlink: String,
    longlink: String,
    reset_hits: bool,
}

// Define JSON struct for error response
#[derive(Serialize)]
pub struct Response {
    pub(crate) success: bool,
    pub(crate) error: bool,
    reason: String,
    pass: bool,
}

// If the api_key environment variable exists
pub fn is_api_ok(http: HttpRequest, config: &Config) -> Response {
    // If the api_key environment variable exists
    if config.api_key.is_some() {
        // If the header exists
        if let Some(header) = auth::api_header(&http) {
            // If the header is correct
            if auth::validate_key(header.to_string(), config) {
                Response {
                    success: true,
                    error: false,
                    reason: "Correct API key".to_string(),
                    pass: false,
                }
            } else {
                Response {
                    success: false,
                    error: true,
                    reason: "Incorrect API key".to_string(),
                    pass: false,
                }
            }
        // The header may not exist when the user logs in through the web interface, so allow a request with no header.
        // Further authentication checks will be conducted in services.rs
        } else {
            // Due to the implementation of this result in services.rs, this JSON object will not be outputted.
            Response {
                success: false,
                error: false,
                reason: "No valid authentication was found".to_string(),
                pass: true,
            }
        }
    } else {
        // If the API key isn't set, but an API Key header is provided
        if auth::api_header(&http).is_some() {
            Response {
                success: false,
                error: true,
                reason: "An API key was provided, but the 'api_key' environment variable is not configured in the Chhoto URL instance".to_string(), 
                pass: false
            }
        } else {
            Response {
                success: false,
                error: false,
                reason: "".to_string(),
                pass: true,
            }
        }
    }
}

// Only have a-z, 0-9, - and _ as valid characters in a shortlink
fn validate_link(link: &str, allow_capital_letters: bool) -> bool {
    let re = if allow_capital_letters {
        Regex::new("^[A-Za-z0-9-_]+$").expect("Regex generation failed.")
    } else {
        Regex::new("^[a-z0-9-_]+$").expect("Regex generation failed.")
    };
    re.is_match(link)
}

// Request the DB for all URLs
pub fn getall(db: &Connection, params: GetReqParams) -> String {
    let page_after = params.page_after.filter(|s| !s.is_empty());
    let page_no = params.page_no.filter(|&n| n > 0);
    let page_size = params.page_size.filter(|&n| n > 0);
    let links = database::getall(db, page_after, page_no, page_size);
    serde_json::to_string(&links).expect("Failure during creation of json from db.")
}

// Make checks and then request the DB to add a new URL entry
pub fn add_link(
    req: String,
    db: &Connection,
    config: &Config,
    using_public_mode: bool,
) -> (bool, String, i64) {
    // Success status, response string, expiry time
    let mut chunks: NewURLRequest;
    if let Ok(json) = serde_json::from_str(&req) {
        chunks = json;
    } else {
        return (false, String::from("Invalid request!"), 0);
    }

    let style = &config.slug_style;
    let len = config.slug_length;
    let allow_capital_letters = config.allow_capital_letters;
    let shortlink_provided = if chunks.shortlink.is_empty() {
        chunks.shortlink = gen_link(style, len, allow_capital_letters);
        false
    } else {
        true
    };

    // In public mode, set automatic expiry delay
    if using_public_mode && config.public_mode_expiry_delay > 0 {
        if chunks.expiry_delay == 0 {
            chunks.expiry_delay = config.public_mode_expiry_delay;
        } else {
            chunks.expiry_delay = chunks.expiry_delay.min(config.public_mode_expiry_delay);
        }
    }

    // Allow max delay of 5 years
    chunks.expiry_delay = chunks.expiry_delay.min(157784760);
    chunks.expiry_delay = chunks.expiry_delay.max(0);

    if validate_link(chunks.shortlink.as_str(), allow_capital_letters) {
        if let Some(expiry_time) =
            database::add_link(&chunks.shortlink, &chunks.longlink, chunks.expiry_delay, db)
        {
            (true, chunks.shortlink, expiry_time)
        } else if shortlink_provided {
            (false, String::from("Short URL is already in use!"), 0)
        } else if config.slug_style == "UID" && config.try_longer_slug {
            // Optionally, retry with a longer slug length
            chunks.shortlink = gen_link(style, len + 4, allow_capital_letters);
            if let Some(expiry_time) =
                database::add_link(&chunks.shortlink, &chunks.longlink, chunks.expiry_delay, db)
            {
                (true, chunks.shortlink, expiry_time)
            } else {
                (false, String::from("Something went very wrong!"), 0)
            }
        } else {
            (false, String::from("Something went wrong!"), 0)
        }
    } else {
        (false, String::from("Short URL is not valid!"), 0)
    }
}

// Make checks and then request the DB to edit an URL entry
pub fn edit_link(req: String, db: &Connection, config: &Config) -> Option<(bool, String)> {
    // None means success
    // The boolean is true when it's a server error and false when it's a client error
    // The string is the error message

    let chunks: EditURLRequest;
    if let Ok(json) = serde_json::from_str(&req) {
        chunks = json;
    } else {
        return Some((false, String::from("Malformed request!")));
    }
    if !validate_link(&chunks.shortlink, config.allow_capital_letters) {
        return Some((false, String::from("Invalid shortlink!")));
    }
    let result = database::edit_link(&chunks.shortlink, &chunks.longlink, chunks.reset_hits, db);
    if Ok(0) == result {
        // Zero rows returned means no updates
        Some((
            false,
            "The short link was not found, and could not be edited.".to_string(),
        ))
    } else if result.is_ok() {
        None
    } else {
        Some((true, String::from("Something went wrong!"))) // Should not really happen
    }
}
// Check if link, and request DB to delete it if exists
pub fn delete_link(shortlink: String, db: &Connection, allow_capital_letters: bool) -> bool {
    if validate_link(shortlink.as_str(), allow_capital_letters) {
        database::delete_link(shortlink, db)
    } else {
        false
    }
}

// Generate a random link using either adjective-name pair (default) of a slug or a-z, 0-9
fn gen_link(style: &String, len: usize, allow_capital_letters: bool) -> String {
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

    if style == "UID" {
        if allow_capital_letters {
            nanoid!(len, &CHARS_CAPITAL)
        } else {
            nanoid!(len, &CHARS_SMALL)
        }
    } else {
        format!(
            "{0}-{1}",
            ADJECTIVES
                .choose(&mut rand::rng())
                .expect("Error choosing random adjective."),
            NAMES
                .choose(&mut rand::rng())
                .expect("Error choosing random name.")
        )
    }
}
