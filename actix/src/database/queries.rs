// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

pub(super) const FIND_URL: &str = "
SELECT long_url, hits, expiry_time, notes FROM urls
  WHERE short_url = :short
    AND (
      expiry_time IS NULL 
      OR expiry_time > :now
    )";

pub(super) const FIND_AND_ADD_HIT: &str = "
UPDATE urls 
  SET hits = hits + 1 
  WHERE short_url = :short 
    AND (
      expiry_time IS NULL 
      OR expiry_time > :now
    )
RETURNING long_url";

pub(super) const ADD_LINK: &str = "
INSERT INTO urls
  (long_url, short_url, hits, expiry_time, notes)
  VALUES (:long, :short, 0, :expiry, :notes)
ON CONFLICT(short_url) DO UPDATE 
  SET long_url = :long, hits = 0, expiry_time = :expiry, notes = :notes
  WHERE short_url = :short 
    AND expiry_time <= :now
    AND expiry_time IS NOT NULL";

pub(super) const DELETE_LINK: &str = "DELETE FROM urls WHERE short_url = :short";

pub(super) const URLS_TABLE_SCHEMA: &str = "
CREATE TABLE urls (
  id INTEGER PRIMARY KEY,
  short_url TEXT NOT NULL,
  long_url TEXT NOT NULL,
  hits INTEGER NOT NULL,
  expiry_time INTEGER,
  notes TEXT
)";

pub(super) const CLEANUP: &str =
    "DELETE FROM urls WHERE :now >= expiry_time AND expiry_time IS NOT NULL";

pub(super) const TABLE_LIST: &str = "
SELECT type, name FROM sqlite_master
  WHERE type IN ('table', 'index') 
    AND name NOT LIKE 'sqlite_%'";

pub(super) const FTS_TABLE_SCHEMA: &str = "
CREATE VIRTUAL TABLE urls_fts USING fts5(
  long_url, short_url, notes,
  content='urls',
  content_rowid='id',
  tokenize='trigram remove_diacritics 2'
)";

pub(super) const FTS_TRIGGERS: [&str; 3] = [
    "
CREATE TRIGGER urls_insert
AFTER INSERT ON urls BEGIN
  INSERT INTO urls_fts(rowid, long_url, short_url, notes)
  VALUES (new.id, new.long_url, new.short_url, new.notes);
END",
    "
CREATE TRIGGER urls_delete
AFTER DELETE ON urls BEGIN
  INSERT INTO urls_fts(urls_fts, rowid, long_url, short_url, notes)
  VALUES('delete', old.id, old.long_url, old.short_url, old.notes);
END",
    "
CREATE TRIGGER urls_update
AFTER UPDATE ON urls BEGIN
  INSERT INTO urls_fts(urls_fts, rowid, long_url, short_url, notes)
  VALUES('delete', old.id, old.long_url, old.short_url, old.notes);
  INSERT INTO urls_fts(rowid, long_url, short_url, notes)
  VALUES (new.id, new.long_url, new.short_url, new.notes);
END",
];
pub(super) const URLS_MIGRATION_3: &str = "
INSERT INTO urls (long_url, short_url, hits, expiry_time, notes)
  SELECT long_url, short_url, hits, NULLIF(expiry_time,0), NULLIF(notes,'')
  FROM urls_old       
  ORDER BY id";

pub(super) const EDIT_LINK: &str = "
UPDATE urls
  SET
    long_url = :long,
    hits = COALESCE(:hits, hits),
    notes = COALESCE(:notes, notes),
    expiry_time = COALESCE(:expiry, expiry_time)
  WHERE short_url = :short
    AND (expiry_time IS NULL OR expiry_time > :now)
";

pub(super) const GETALL_QUERIES: [&str; 4] = [
    // 0 => standard
    "
SELECT short_url, long_url, hits, expiry_time, notes FROM (
  SELECT t.id, t.short_url, t.long_url, t.hits, t.expiry_time, t.notes
  FROM urls AS t
  WHERE (
    t.expiry_time IS NULL
    OR t.expiry_time > :now
  ) 
  ORDER BY t.id DESC
  LIMIT :size OFFSET :offset
) 
ORDER BY id ASC",
    // 1 => cursor
    "
SELECT short_url, long_url, hits, expiry_time, notes FROM (
  SELECT t.id, t.short_url, t.long_url, t.hits, t.expiry_time, t.notes
  FROM urls AS t
  JOIN urls AS u
    ON u.short_url = :pos
  WHERE
    t.id < u.id
  AND (
    t.expiry_time IS NULL
    OR t.expiry_time > :now
  ) 
  ORDER BY t.id DESC
  LIMIT :size
) ORDER BY id ASC",
    // 2 => standard + fts
    "
SELECT short_url, long_url, hits, expiry_time, notes FROM (
  SELECT t.id, t.short_url, t.long_url, t.hits, t.expiry_time, t.notes
  FROM urls AS t
  JOIN urls_fts AS f
    ON t.id = f.rowid
  WHERE (
    t.expiry_time IS NULL
    OR t.expiry_time > :now
    )
  AND urls_fts MATCH :filter
  ORDER BY t.id DESC
  LIMIT :size OFFSET :offset
)
ORDER BY id ASC",
    // 3 => cursor + fts
    "
SELECT short_url, long_url, hits, expiry_time, notes FROM (
  SELECT t.id, t.short_url, t.long_url, t.hits, t.expiry_time, t.notes
  FROM urls AS t
  JOIN urls AS u
    ON u.short_url = :pos
  JOIN urls_fts AS f
    ON t.id = f.rowid
  WHERE
    t.id < u.id
    AND (
      t.expiry_time IS NULL
      OR t.expiry_time > :now
    )
    AND urls_fts MATCH :filter
  ORDER BY t.id DESC
  LIMIT :size
)
ORDER BY id ASC",
];
