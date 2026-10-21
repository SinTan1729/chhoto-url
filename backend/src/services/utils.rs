// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_files::NamedFile;
use actix_web::{HttpResponse, Responder, dev::ConnectionInfo, http::StatusCode, web};
use argon2::{Argon2, PasswordVerifier};
use log::{debug, error};
use nanoid::nanoid;
use openidconnect::{AccessTokenHash, AuthorizationCode, OAuth2TokenResponse, TokenResponse};
use rand::{random_range, seq::IndexedRandom};
use rusqlite::Connection;
use serde::Deserialize;
use serde_json::Value;
use std::env;
use tokio::sync::mpsc;
use url::Url;

use crate::{
    AppState,
    auth::Auth,
    config::{Config, Credential, SlugStyle},
    database::{self, add_links},
    oidc::{OidcClient, OidcData},
    services::types::{
        ChhotoError::{self, ClientError, ServerError},
        GetReqParams, OidcCallbackParams, OneOrMany,
    },
};

// Struct for reading link pairs sent during API call for new link
#[derive(Deserialize, Clone)]
pub(crate) struct NewURLRequest {
    #[serde(default)]
    pub(crate) shortlink: String,
    pub(crate) longlink: String,
    pub(crate) expiry_delay: Option<i64>,
    pub(crate) notes: Option<String>,
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
fn is_longlink_valid(link: &str, allowed_protocols: &[String]) -> bool {
    let parts = Url::parse(link);
    parts.is_ok_and(|u| allowed_protocols.contains(&u.scheme().to_owned()))
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

// Only have printable ascii characters as valid characters in notes
#[inline]
fn is_note_valid(note: &Option<String>) -> bool {
    note.as_ref()
        .is_none_or(|n| n.chars().all(|c| c.is_ascii_graphic() || c == ' '))
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
    (s.len() > 2).then(|| s.to_owned())
}

// Simply get the version string
pub(crate) fn get_version() -> String {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    const GIT_COMMIT: Option<&str> = option_env!("CARGO_GIT_COMMIT");

    match GIT_COMMIT {
        Some(commit) if !commit.trim().is_empty() => {
            let mut out = String::with_capacity(VERSION.len() + commit.len() + 5);
            out.push_str(VERSION);
            out.push_str("-dev+");
            out.push_str(commit);
            out
        }
        _ => VERSION.to_owned(),
    }
}

// Request the DB for all URLs
pub(super) fn getall_helper(db: &Connection, params: GetReqParams) -> Result<String, ChhotoError> {
    let page_after = match params.page_after {
        Some(s) if s.is_empty() => {
            return Err(ChhotoError::ClientError {
                reason: "Invalid page_after was supplied!".to_owned(),
            });
        }
        other => other,
    };
    let page_no = match params.page_no {
        Some(n) if n <= 0 => {
            return Err(ChhotoError::ClientError {
                reason: "Invalid page_no was supplied!".to_owned(),
            });
        }
        other => other,
    };
    let page_size = params.page_size.filter(|&n| n > 0);
    let filter = params
        .filter
        .map(|s| {
            normalize_filter(&s).ok_or(ChhotoError::ClientError {
                reason: "Invalid filter was supplied!".to_owned(),
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
type AddLinksReturnType = Result<(Vec<Result<(String, i64), ChhotoError>>, bool), ChhotoError>;
pub(super) fn add_links_helper(
    req: &str,
    db: &mut Connection,
    config: &Config,
    using_public_mode: bool,
) -> AddLinksReturnType {
    // Ok : Vec<shortlink, expiry_time>, single_request
    let Ok((single_request, chunks)) =
        serde_json::from_str::<OneOrMany<NewURLRequest>>(req).map(|s| {
            let single = matches!(s, OneOrMany::One(_));
            (single, s.normalize())
        })
    else {
        return Err(ClientError {
            reason: "Invalid request!".to_owned(),
        });
    };
    if chunks.is_empty() {
        return Err(ClientError {
            reason: "An empty array of links was provided!".to_owned(),
        });
    }

    let allow_capital_letters = config.allow_capital_letters;
    let public_mode_expiry_delay = config.public_mode_expiry_delay;

    let mut output: Vec<_> = (0..chunks.len()).map(|_| Err(ServerError)).collect();
    let (mut with_shortlinks, mut without_shortlinks) = (Vec::new(), Vec::new());
    let clean_req = |mut req: NewURLRequest| {
        // Allow max delay of 5 years
        let exp = req
            .expiry_delay
            .map(|d| d.clamp(0, 157_784_760))
            .filter(|&d| d > 0);
        req.expiry_delay = match (using_public_mode, public_mode_expiry_delay) {
            (true, Some(delay)) => Some(exp.map_or(delay, |d| d.min(delay))),
            _ => exp,
        };
        req.notes = req.notes.filter(|s| !s.is_empty());
        req
    };
    for (i, req) in chunks.into_iter().enumerate() {
        let req = clean_req(req);
        if !is_shortlink_valid(&req.shortlink, allow_capital_letters) {
            output[i] = Err(ClientError {
                reason: "Invalid shortlink!".to_owned(),
            });
        } else if !is_longlink_valid(&req.longlink, &config.allowed_protocols) {
            output[i] = Err(ClientError {
                reason: "Invalid longlink!".to_owned(),
            });
        } else if !is_note_valid(&req.notes) {
            output[i] = Err(ClientError {
                reason: "Invalid notes!".to_owned(),
            });
        } else if req.shortlink.is_empty() {
            without_shortlinks.push((i, req));
        } else {
            with_shortlinks.push((i, req));
        }
    }

    for (i, res) in add_links(with_shortlinks, db, false).0 {
        output[i] = res
    }

    let with_link = |mut req: NewURLRequest, retry: bool| {
        req.shortlink = gen_link(
            &config.slug_style,
            config.slug_length,
            config.allow_capital_letters,
            retry,
        );
        req
    };
    let (successful, rejected) = add_links(
        without_shortlinks
            .into_iter()
            .map(|(i, r)| (i, with_link(r, false)))
            .collect(),
        db,
        true,
    );
    for (i, res) in successful {
        if res.is_ok() {
            output[i] = res
        }
    }
    if let Some(rej) = rejected {
        for (i, res) in add_links(
            rej.into_iter()
                .map(|(i, r)| (i, with_link(r, true)))
                .collect(),
            db,
            false,
        )
        .0
        {
            output[i] = res
        }
    }

    if !single_request {
        debug!("Processed a batch of {} requests.", output.len());
    }
    Ok((output, single_request))
}

// Make checks and then request the DB to edit an URL entry
pub(super) async fn edit_link_helper(
    req: &str,
    db: &Connection,
    hits_tx: &mpsc::Sender<(String, bool)>,
    config: &Config,
) -> Result<(), ChhotoError> {
    let chunks: EditURLRequest;
    if let Ok(json) = serde_json::from_str(req) {
        chunks = json;
    } else {
        return Err(ClientError {
            reason: "Malformed request!".to_owned(),
        });
    }
    if !is_shortlink_valid(&chunks.shortlink, config.allow_capital_letters) {
        return Err(ClientError {
            reason: "Invalid shortlink!".to_owned(),
        });
    } else if !is_longlink_valid(&chunks.longlink, &config.allowed_protocols) {
        return Err(ClientError {
            reason: "Invalid longlink!".to_owned(),
        });
    } else if !is_note_valid(&chunks.notes) {
        return Err(ClientError {
            reason: "Invalid notes!".to_owned(),
        });
    }
    let result = database::edit_link(
        &chunks.shortlink,
        &chunks.longlink,
        chunks.reset_hits,
        chunks.expiry_time.filter(|&t| t > 0),
        chunks.notes.filter(|s| !s.is_empty()).as_deref(),
        hits_tx,
        db,
    )
    .await;
    match result {
        // Zero rows returned means no updates
        Ok(0) => Err(ClientError {
            reason: "The shortlink was not found, and could not be edited.".to_owned(),
        }),
        Ok(_) => Ok(()),
        Err(()) => Err(ServerError),
    }
}

// Derive origin info for OIDC
pub(super) fn derive_origin_info(
    connection: &ConnectionInfo,
    using_custom_landing_dir: bool,
) -> (String, String) {
    let (scheme, host) = (connection.scheme(), connection.host());
    let mut origin = String::with_capacity(scheme.len() + host.len() + 16);
    origin.push_str(scheme);
    origin.push_str("://");
    origin.push_str(host);
    if using_custom_landing_dir {
        origin.push_str("/admin/manage");
    }
    let mut callback = origin.clone();
    callback.push_str("/oidc/callback");
    (origin, callback)
}

// Verify OIDC callback data
pub(super) async fn oidc_verify(
    client: OidcClient,
    params: OidcCallbackParams,
    oidc: &OidcData,
    custom_claim: &Option<(String, String)>,
) -> Result<(), String> {
    let Some(state) = &params.state else {
        return Err("State check failed.".to_string());
    };
    let Some((verifier, nonce)) = (*oidc.states.lock().await).remove(state) else {
        return Err("State mismatch.".to_string());
    };

    let code = params
        .code
        .expect("Authorization code should be available when state it valid.");

    let token_response = client
        .exchange_code(AuthorizationCode::new(code))
        .map_err(|_| "Auth request failed: code")?
        // Set the PKCE code verifier.
        .set_pkce_verifier(verifier)
        .request_async(&oidc.http_client)
        .await
        .map_err(|e| {
            eprintln!("OIDC token exchange failed: {e:?}");
            "Auth request failed: PKCE"
        })?;

    let id_token = token_response
        .id_token()
        .ok_or("Server did not return an ID token")?;
    let id_token_verifier = client.id_token_verifier();
    let claims = id_token
        .claims(&id_token_verifier, &nonce)
        .map_err(|_| "Server did not return any claims".to_string())?;
    if let Some(expected_access_token_hash) = claims.access_token_hash() {
        let actual_access_token_hash = (|| -> Result<_, String> {
            let alg = id_token.signing_alg().map_err(|e| e.to_string())?;
            let key = id_token
                .signing_key(&id_token_verifier)
                .map_err(|e| e.to_string())?;

            AccessTokenHash::from_token(token_response.access_token(), alg, key)
                .map_err(|e| e.to_string())
        })()
        .map_err(|_| "Mismatched token".to_string())?;

        if actual_access_token_hash != *expected_access_token_hash {
            return Err("Invalid access token".to_string());
        }
    }

    fn matches_claim(value: &Value, expected: &str) -> bool {
        match value {
            Value::String(s) => s == expected,
            Value::Bool(b) => b.to_string() == expected,
            Value::Number(n) => n.to_string() == expected,
            Value::Array(values) => values.iter().any(|v| matches_claim(v, expected)),
            _ => false,
        }
    }
    if let Some((k, v)) = custom_claim
        && !claims
            .additional_claims()
            .claims
            .get(k)
            .is_some_and(|val| matches_claim(val, v))
    {
        return Err("Custom claim mismatch.".to_string());
    }

    Ok(())
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
            reason: "The shortlink is invalid.".to_owned(),
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
            if allow_capital_letters {
                nanoid!(slug_len, &CHARS_CAPITAL)
            } else {
                nanoid!(slug_len, &CHARS_SMALL)
            }
        }
        SlugStyle::Pair => {
            let adj = ADJECTIVES
                .choose(&mut rand::rng())
                .expect("Error choosing random adjective.")
                .to_owned();
            let name = NAMES
                .choose(&mut rand::rng())
                .expect("Error choosing random name.");
            let mut out = String::with_capacity(adj.len() + name.len() + 4);
            out.push_str(name);
            out.push('-');
            out.push_str(adj);
            if try_longer_slug {
                let four_digits = |n| {
                    let mut out = String::with_capacity(4);
                    out.push((b'0' + (n / 1000) as u8) as char);
                    out.push((b'0' + ((n / 100) % 10) as u8) as char);
                    out.push((b'0' + ((n / 10) % 10) as u8) as char);
                    out.push((b'0' + (n % 10) as u8) as char);
                    out
                };
                out.push_str(&four_digits(random_range(0..=9999)));
            }
            out
        }
    }
}

// Redirect with no cache
pub(super) fn oidc_redirect_once(url: &str) -> HttpResponse {
    HttpResponse::Found()
        .insert_header(("Location", url))
        .insert_header(("Cache-Control", "no-store, no-cache, must-revalidate"))
        .finish()
}

// 404 error page
pub(crate) async fn error404(data: web::Data<AppState>) -> impl Responder {
    NamedFile::open([&data.frontend_dir, "/static/404.html"].concat())
        .customize()
        .with_status(StatusCode::NOT_FOUND)
}

// Verify login details
pub(super) async fn verify_login(
    password: String,
    config: Config,
    auth: Auth,
) -> Result<(bool, bool), actix_web::Error> {
    actix_web::web::block(move || {
        is_authorized(&password, &config, matches!(auth, Auth::NoAuthNeeded))
    })
    .await
    .map_err(actix_web::error::ErrorInternalServerError)
}
fn is_authorized(provided_pass: &str, config: &Config, no_auth_needed: bool) -> (bool, bool) {
    if let Some(password) = &config.password {
        let authorized = match password {
            Credential::Argon2(hash) => Argon2::default()
                .verify_password(provided_pass.as_bytes(), hash.as_ref())
                .is_ok(),
            Credential::Plain(pass) => {
                subtle::ConstantTimeEq::ct_eq(pass.as_bytes(), provided_pass.as_bytes()).into()
            }
        };

        (authorized, true)
    } else {
        (no_auth_needed, false)
    }
}
