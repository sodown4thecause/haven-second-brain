//! crates/haven-index - SQLite (WAL, single-writer) full-text index for Haven.
//!
//! Constraint: the index is derived state. Files are the only source of
//! truth (`AGENTS.md §1`). Delete `.haven/` and the index rebuilds.
//! Schema-versioning via `INDEX_SCHEMA_VERSION`.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const INDEX_SCHEMA_VERSION: i32 = 1;

#[derive(Debug, Error)]
pub enum IndexError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("schema version mismatch: stored={stored}, build={build}")]
    SchemaMismatch { stored: i32, build: i32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedFile {
    pub path: PathBuf,
    pub body: String,
    pub content_hash: String,
}

#[derive(Clone)]
pub struct Index {
    inner: Arc<Mutex<Connection>>,
    #[allow(dead_code)]
    root: PathBuf,
}

impl Index {
    pub fn open(root: &Path) -> Result<Self, IndexError> {
        let haven_dir = root.join(".haven");
        std::fs::create_dir_all(&haven_dir)?;
        let db_path = haven_dir.join("index.sqlite");
        let conn = Connection::open(&db_path)?;
        // Single-writer, recoverable on partial scans.
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        Self::migrate(&conn)?;
        Ok(Index {
            inner: Arc::new(Mutex::new(conn)),
            root: root.to_path_buf(),
        })
    }

    fn migrate(conn: &Connection) -> Result<(), IndexError> {
        let stored: Option<i32> = conn
            .query_row(
                "SELECT version FROM meta WHERE key = 'schema_version'",
                [],
                |row| row.get(0),
            )
            .optional()?;
        if let Some(ver) = stored {
            if ver != INDEX_SCHEMA_VERSION {
                return Err(IndexError::SchemaMismatch {
                    stored: ver,
                    build: INDEX_SCHEMA_VERSION,
                });
            }
        } else {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS meta(
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL
                )",
                [],
            )?;
            conn.execute(
                "INSERT OR REPLACE INTO meta(key, value) VALUES('schema_version', ?1)",
                params![INDEX_SCHEMA_VERSION.to_string()],
            )?;
        }
        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS fts USING fts5(path, body)",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS files(
                path TEXT PRIMARY KEY,
                content_hash TEXT NOT NULL,
                mtime INTEGER NOT NULL
            )",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS edges(
                src TEXT NOT NULL,
                dst TEXT NOT NULL,
                kind TEXT NOT NULL,
                PRIMARY KEY (src, dst, kind)
            ) WITHOUT ROWID",
            [],
        )?;
        Ok(())
    }

    /// Idempotent upsert. Calls with the same content hash short-circuit to
    /// avoid rewriting the FTS row.
    pub fn upsert(&self, file: &IndexedFile) -> Result<(), IndexError> {
        let conn = self.inner.lock();
        let existing: Option<String> = conn
            .query_row(
                "SELECT content_hash FROM files WHERE path = ?1",
                params![file.path.display().to_string()],
                |row| row.get(0),
            )
            .optional()?;
        if existing.as_deref() == Some(file.content_hash.as_str()) {
            return Ok(());
        }
        conn.execute(
            "INSERT OR REPLACE INTO files(path, content_hash, mtime) VALUES(?1, ?2, ?3)",
            params![
                file.path.display().to_string(),
                file.content_hash,
                chrono::Utc::now().timestamp(),
            ],
        )?;
        conn.execute(
            "DELETE FROM fts WHERE path = ?1",
            params![file.path.display().to_string()],
        )?;
        conn.execute(
            "INSERT INTO fts(path, body) VALUES(?1, ?2)",
            params![file.path.display().to_string(), file.body],
        )?;
        Ok(())
    }

    pub fn delete(&self, path: &Path) -> Result<(), IndexError> {
        let key = path.display().to_string();
        let conn = self.inner.lock();
        conn.execute("DELETE FROM fts WHERE path = ?1", params![&key])?;
        conn.execute("DELETE FROM files WHERE path = ?1", params![&key])?;
        Ok(())
    }

    pub fn rename(&self, old: &Path, new: &Path) -> Result<(), IndexError> {
        let old_key = old.display().to_string();
        let new_key = new.display().to_string();
        let conn = self.inner.lock();
        // FTS5 UPDATE statement is rowid-keyed; do delete + insert.
        let body: Option<String> = conn
            .query_row(
                "SELECT body FROM fts WHERE path = ?1",
                params![&old_key],
                |row| row.get(0),
            )
            .optional()?;
        conn.execute("DELETE FROM fts WHERE path = ?1", params![&old_key])?;
        if let Some(body) = body {
            conn.execute(
                "INSERT INTO fts(path, body) VALUES(?1, ?2)",
                params![&new_key, body],
            )?;
        }
        conn.execute(
            "UPDATE files SET path = ?1 WHERE path = ?2",
            params![&new_key, &old_key],
        )?;
        Ok(())
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<PathBuf>, IndexError> {
        let conn = self.inner.lock();
        let mut stmt = conn.prepare("SELECT path FROM fts WHERE fts MATCH ?1 LIMIT ?2")?;
        let mut rows = stmt.query(params![query, limit as i64])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            let path: String = row.get(0)?;
            out.push(PathBuf::from(path));
        }
        Ok(out)
    }

    pub fn add_edge(&self, src: &Path, dst: &Path, kind: &str) -> Result<(), IndexError> {
        let conn = self.inner.lock();
        conn.execute(
            "INSERT OR REPLACE INTO edges(src, dst, kind) VALUES(?1, ?2, ?3)",
            params![src.display().to_string(), dst.display().to_string(), kind,],
        )?;
        Ok(())
    }

    pub fn edges_from(&self, src: &Path) -> Result<Vec<(PathBuf, String)>, IndexError> {
        let conn = self.inner.lock();
        let mut stmt = conn.prepare("SELECT dst, kind FROM edges WHERE src = ?1")?;
        let rows = stmt.query_map(params![src.display().to_string()], |row| {
            let dst: String = row.get(0)?;
            let kind: String = row.get(1)?;
            Ok((dst, kind))
        })?;
        let mut out = Vec::new();
        for r in rows {
            let (dst, kind) = r?;
            out.push((PathBuf::from(dst), kind));
        }
        Ok(out)
    }

    /// After `rm -rf .haven/`, `open()` re-runs migration + rebuild.
    pub fn rebuild_from(root: &Path, files: &[IndexedFile]) -> Result<Self, IndexError> {
        let index = Self::open(root)?;
        for f in files {
            index.upsert(f)?;
        }
        Ok(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write(tmp: &TempDir, name: &str, body: &str) -> PathBuf {
        let p = tmp.path().join(name);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, body).unwrap();
        p
    }

    #[test]
    fn upsert_then_search() {
        let tmp = TempDir::new().unwrap();
        let p = write(&tmp, "notes/a.md", "alpha beta gamma");
        let index = Index::open(tmp.path()).unwrap();
        index
            .upsert(&IndexedFile {
                path: p.clone(),
                body: "alpha beta gamma".into(),
                content_hash: "h1".into(),
            })
            .unwrap();
        let res = index.search("beta", 10).unwrap();
        assert_eq!(res, vec![p.clone()]);
    }

    #[test]
    fn idempotent_upsert_does_not_corrupt_fts() {
        let tmp = TempDir::new().unwrap();
        let p = write(&tmp, "notes/a.md", "alpha");
        let index = Index::open(tmp.path()).unwrap();
        let file = IndexedFile {
            path: p.clone(),
            body: "alpha".into(),
            content_hash: "h1".into(),
        };
        index.upsert(&file).unwrap();
        index.upsert(&file).unwrap();
        let res = index.search("alpha", 10).unwrap();
        assert_eq!(res, vec![p]);
    }

    #[test]
    fn delete_clears_search() {
        let tmp = TempDir::new().unwrap();
        let p = write(&tmp, "notes/a.md", "alpha");
        let index = Index::open(tmp.path()).unwrap();
        index
            .upsert(&IndexedFile {
                path: p.clone(),
                body: "alpha".into(),
                content_hash: "h1".into(),
            })
            .unwrap();
        index.delete(&p).unwrap();
        let res = index.search("alpha", 10).unwrap();
        assert!(res.is_empty());
    }

    #[test]
    fn rename_keeps_indexed_body() {
        let tmp = TempDir::new().unwrap();
        let old = write(&tmp, "notes/a.md", "alpha");
        let index = Index::open(tmp.path()).unwrap();
        index
            .upsert(&IndexedFile {
                path: old.clone(),
                body: "alpha".into(),
                content_hash: "h1".into(),
            })
            .unwrap();
        let new = write(&tmp, "notes/b.md", "alpha");
        index.rename(&old, &new).unwrap();
        let res = index.search("alpha", 10).unwrap();
        assert_eq!(res, vec![new]);
    }

    #[test]
    fn rebuild_after_haven_dir_vanish() {
        let tmp = TempDir::new().unwrap();
        let p = write(&tmp, "notes/a.md", "alpha");
        let _index = Index::open(tmp.path()).unwrap();
        let haven = tmp.path().join(".haven");
        std::fs::remove_dir_all(&haven).unwrap();
        let files = vec![IndexedFile {
            path: p.clone(),
            body: "alpha".into(),
            content_hash: "h1".into(),
        }];
        let index = Index::rebuild_from(tmp.path(), &files).unwrap();
        let res = index.search("alpha", 10).unwrap();
        assert_eq!(res, vec![p]);
    }
}
