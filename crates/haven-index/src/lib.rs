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
        // The `meta` table must exist BEFORE we query it; otherwise the
        // first-time open fails with `no such table: meta`.
        conn.execute(
            "CREATE TABLE IF NOT EXISTS meta(
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;
        let stored: Option<i32> = conn
            .query_row(
                "SELECT value FROM meta WHERE key = 'schema_version'",
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
        let key = file.path.to_string_lossy().into_owned();
        let conn = self.inner.lock();
        let existing: Option<String> = conn
            .query_row(
                "SELECT content_hash FROM files WHERE path = ?1",
                params![&key],
                |row| row.get(0),
            )
            .optional()?;
        if existing.as_deref() == Some(file.content_hash.as_str()) {
            return Ok(());
        }
        // Wrap the files + fts writes in a transaction so a partial failure
        // cannot leave the index looking like the new file but matching no
        // body, or vice versa.
        conn.execute_batch("BEGIN")?;
        if let Err(e) = (|| -> Result<(), rusqlite::Error> {
            conn.execute(
                "INSERT OR REPLACE INTO files(path, content_hash, mtime) VALUES(?1, ?2, ?3)",
                params![&key, &file.content_hash, chrono::Utc::now().timestamp()],
            )?;
            conn.execute("DELETE FROM fts WHERE path = ?1", params![&key])?;
            conn.execute(
                "INSERT INTO fts(path, body) VALUES(?1, ?2)",
                params![&key, &file.body],
            )?;
            Ok(())
        })() {
            let _ = conn.execute_batch("ROLLBACK");
            return Err(e.into());
        }
        conn.execute_batch("COMMIT")?;
        Ok(())
    }

    pub fn delete(&self, path: &Path) -> Result<(), IndexError> {
        let key = path.to_string_lossy().into_owned();
        let conn = self.inner.lock();
        // Drop any edges that reference the deleted path so
        // `edges_from()` never returns dangling references.
        conn.execute(
            "DELETE FROM edges WHERE src = ?1 OR dst = ?1",
            params![&key],
        )?;
        conn.execute("DELETE FROM fts WHERE path = ?1", params![&key])?;
        conn.execute("DELETE FROM files WHERE path = ?1", params![&key])?;
        Ok(())
    }

    pub fn rename(&self, old: &Path, new: &Path) -> Result<(), IndexError> {
        let old_key = old.to_string_lossy().into_owned();
        let new_key = new.to_string_lossy().into_owned();
        let conn = self.inner.lock();
        // Wrap the fts + files rewrite in a transaction so a primary-key
        // conflict cannot leave the tables pointing at different paths.
        conn.execute_batch("BEGIN")?;
        if let Err(e) = (|| -> Result<(), rusqlite::Error> {
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
            // Refresh edges that referenced the old path so the link graph
            // does not point at the pre-rename location.
            conn.execute(
                "UPDATE edges SET src = ?1 WHERE src = ?2",
                params![&new_key, &old_key],
            )?;
            conn.execute(
                "UPDATE edges SET dst = ?1 WHERE dst = ?2",
                params![&new_key, &old_key],
            )?;
            Ok(())
        })() {
            let _ = conn.execute_batch("ROLLBACK");
            return Err(e.into());
        }
        conn.execute_batch("COMMIT")?;
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
            params![
                src.to_string_lossy().into_owned(),
                dst.to_string_lossy().into_owned(),
                kind,
            ],
        )?;
        Ok(())
    }

    pub fn edges_from(&self, src: &Path) -> Result<Vec<(PathBuf, String)>, IndexError> {
        let conn = self.inner.lock();
        let mut stmt = conn.prepare("SELECT dst, kind FROM edges WHERE src = ?1")?;
        let rows = stmt.query_map(params![src.to_string_lossy().into_owned()], |row| {
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

    /// After `rm -rf .haven/`, `open()` re-runs migration + rebuild. Edges
    /// are intentionally NOT rebuilt: callers must re-derive and re-insert
    /// edges after a rebuild, otherwise the link graph decays silently.
    pub fn rebuild_from(root: &Path, files: &[IndexedFile]) -> Result<Self, IndexError> {
        let index = Self::open(root)?;
        for f in files {
            index.upsert(f)?;
        }
        Ok(index)
    }

    /// Rebuilder variant that also restores edges (for callers that have
    /// the link graph available after a `rm -rf .haven/`).
    pub fn rebuild_from_with_edges(
        root: &Path,
        files: &[IndexedFile],
        edges: &[(PathBuf, PathBuf, String)],
    ) -> Result<Self, IndexError> {
        let index = Self::open(root)?;
        for f in files {
            index.upsert(f)?;
        }
        for (src, dst, kind) in edges {
            index.add_edge(src, dst, kind)?;
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
    fn delete_drops_edges_referencing_path() {
        let tmp = TempDir::new().unwrap();
        let a = write(&tmp, "notes/a.md", "alpha");
        let b = write(&tmp, "notes/b.md", "beta");
        let index = Index::open(tmp.path()).unwrap();
        index.add_edge(&a, &b, "wikilink").unwrap();
        index.delete(&a).unwrap();
        let edges = index.edges_from(&a).unwrap();
        assert!(
            edges.is_empty(),
            "edges pointing at deleted path must be purged"
        );
        let edges = index.edges_from(&b).unwrap();
        assert!(
            edges.is_empty(),
            "edges whose `dst` was deleted must also be purged"
        );
    }

    #[test]
    fn rename_refreshes_edges_and_index() {
        let tmp = TempDir::new().unwrap();
        let old = write(&tmp, "notes/a.md", "alpha");
        let b = write(&tmp, "notes/b.md", "beta");
        let index = Index::open(tmp.path()).unwrap();
        index
            .upsert(&IndexedFile {
                path: old.clone(),
                body: "alpha".into(),
                content_hash: "h1".into(),
            })
            .unwrap();
        index.add_edge(&old, &b, "wikilink").unwrap();
        let new = write(&tmp, "notes/c.md", "alpha");
        index.rename(&old, &new).unwrap();
        let res = index.search("alpha", 10).unwrap();
        assert_eq!(res, vec![new.clone()]);
        let edges = index.edges_from(&new).unwrap();
        assert_eq!(edges.len(), 1, "edges must point at the new path");
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

    #[test]
    fn rebuild_with_edges_restores_link_graph() {
        let tmp = TempDir::new().unwrap();
        let a = write(&tmp, "notes/a.md", "alpha");
        let b = write(&tmp, "notes/b.md", "beta");
        let _index = Index::open(tmp.path()).unwrap();
        let haven = tmp.path().join(".haven");
        std::fs::remove_dir_all(&haven).unwrap();
        let index = Index::rebuild_from_with_edges(
            tmp.path(),
            &[IndexedFile {
                path: a.clone(),
                body: "alpha".into(),
                content_hash: "h1".into(),
            }],
            &[(a.clone(), b.clone(), "wikilink".to_string())],
        )
        .unwrap();
        let edges = index.edges_from(&a).unwrap();
        assert_eq!(edges, vec![(b, "wikilink".into())]);
    }
}
