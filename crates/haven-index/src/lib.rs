//! SQLite-backed search and graph index for Haven.
//!
//! The index is disposable — all data is derivable from the OKF file bundle.
//! Schema changes bump `INDEX_SCHEMA_VERSION` and trigger a full rebuild via
//! [`IndexEngine::rebuild_from_bundle`]. There are no data migrations for
//! derived data; the index is dropped and reindexed from files alone.

use rusqlite::params;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Bumped on any schema change. A mismatch with the stored version causes
/// [`IndexEngine::open`] to reset to a clean, empty schema; callers then rebuild
/// from the bundle. v2: `docs_fts` is now a standalone FTS5 table (not external
/// content) so writes populate the search index directly, and a vector table
/// placeholder is initialized behind the `vector` feature.
pub const INDEX_SCHEMA_VERSION: i32 = 2;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("index schema version mismatch: db={db}, current={current}")]
    SchemaVersionMismatch { db: i32, current: i32 },
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("background task join error: {0}")]
    TaskJoin(#[from] tokio::task::JoinError),
    #[cfg(feature = "vector")]
    #[error("invalid vector dimension: got {got}, expected {expected}")]
    InvalidVectorDimension { got: usize, expected: usize },
}

/// Document metadata stored in the index.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DocEntry {
    pub path: String,
    pub title: String,
    pub doc_type: String,
    pub content_preview: String,
}

/// Full-text search result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchResult {
    pub path: String,
    pub title: String,
    pub snippet: String,
    pub score: f64,
}

/// An edge in the knowledge graph (document A links to document B).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Edge {
    pub source: String,
    pub target: String,
    pub context: Option<String>,
}

/// A node reached during graph traversal, with its hop distance from the start.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphNode {
    pub path: String,
    pub depth: i64,
}

/// The index engine.
///
/// Internally uses a single-writer SQLite connection in WAL mode behind a tokio
/// mutex. Public methods are async-safe; locks are never held across an `.await`.
pub struct IndexEngine {
    db_path: PathBuf,
    inner: Arc<Mutex<rusqlite::Connection>>,
}

impl Clone for IndexEngine {
    fn clone(&self) -> Self {
        Self {
            db_path: self.db_path.clone(),
            inner: self.inner.clone(),
        }
    }
}

impl IndexEngine {
    /// Open or create the index database.
    pub async fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let conn = rusqlite::Connection::open(&path)?;
        conn.execute_batch("PRAGMA journal_mode = WAL;")?;

        let engine = Self {
            db_path: path,
            inner: Arc::new(Mutex::new(conn)),
        };

        engine.ensure_schema().await?;
        Ok(engine)
    }

    /// Create tables if they don't exist, and reset the schema if the stored
    /// version is stale. Never re-entrant: a mismatch drops and recreates the
    /// schema in place (data is disposable — callers repopulate via
    /// [`Self::rebuild_from_bundle`]).
    async fn ensure_schema(&self) -> Result<(), Error> {
        let conn = self.inner.lock().await;
        Self::create_tables(&conn)?;

        let version: i64 = conn
            .query_row(
                "SELECT CAST(value AS INTEGER) FROM _meta WHERE key = 'schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        if version != INDEX_SCHEMA_VERSION as i64 {
            Self::drop_tables(&conn)?;
            Self::create_tables(&conn)?;
            Self::stamp_version(&conn)?;
        }

        Ok(())
    }

    /// Full-text search across indexed documents.
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>, Error> {
        let conn = self.inner.lock().await;
        let mut stmt = conn.prepare(
            "SELECT docs.path, docs.title,
                    snippet(docs_fts, 2, '<mark>', '</mark>', '...', 32) AS snippet,
                    rank
             FROM docs_fts
             JOIN docs ON docs_fts.path = docs.path
             WHERE docs_fts MATCH ?1
             ORDER BY rank
             LIMIT 20",
        )?;

        let results = stmt
            .query_map([query], |row| {
                Ok(SearchResult {
                    path: row.get(0)?,
                    title: row.get(1)?,
                    snippet: row.get(2)?,
                    score: row.get(3)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(results)
    }

    /// Index a single document (writes metadata + FTS rows).
    pub async fn index_document(
        &self,
        path: &str,
        title: &str,
        doc_type: &str,
        content: &str,
    ) -> Result<(), Error> {
        let conn = self.inner.lock().await;
        upsert_doc(&conn, path, title, doc_type, content)?;
        Ok(())
    }

    /// Add an edge between two documents.
    pub async fn add_edge(
        &self,
        source: &str,
        target: &str,
        context: Option<&str>,
    ) -> Result<(), Error> {
        let conn = self.inner.lock().await;
        conn.execute(
            "INSERT OR IGNORE INTO edges (source, target, context) VALUES (?1, ?2, ?3)",
            params![source, target, context],
        )?;
        Ok(())
    }

    /// Outgoing edges from `path` (documents that `path` links to).
    pub async fn outgoing_links(&self, path: &str) -> Result<Vec<Edge>, Error> {
        let conn = self.inner.lock().await;
        let mut stmt = conn.prepare(
            "SELECT source, target, context FROM edges WHERE source = ?1 ORDER BY target",
        )?;
        let rows = stmt
            .query_map([path], |row| {
                Ok(Edge {
                    source: row.get(0)?,
                    target: row.get(1)?,
                    context: row.get(2)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(rows)
    }

    /// Backlinks: incoming edges to `path` (documents that link to `path`).
    pub async fn backlinks(&self, path: &str) -> Result<Vec<Edge>, Error> {
        let conn = self.inner.lock().await;
        let mut stmt = conn.prepare(
            "SELECT source, target, context FROM edges WHERE target = ?1 ORDER BY source",
        )?;
        let rows = stmt
            .query_map([path], |row| {
                Ok(Edge {
                    source: row.get(0)?,
                    target: row.get(1)?,
                    context: row.get(2)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(rows)
    }

    /// Recursively traverse the graph from `start`, returning every reachable
    /// document with the minimum hop distance. Uses a recursive CTE bounded by
    /// `max_depth` so cycles cannot run away; results are deduplicated by path.
    pub async fn graph_reachable(
        &self,
        start: &str,
        max_depth: u32,
    ) -> Result<Vec<GraphNode>, Error> {
        let depth: i64 = max_depth as i64;
        let conn = self.inner.lock().await;
        let mut stmt = conn.prepare(
            "WITH RECURSIVE reach(path, depth) AS (
                 SELECT ?1, 0
                 UNION ALL
                 SELECT e.target, r.depth + 1
                 FROM reach r
                 JOIN edges e ON e.source = r.path
                 WHERE r.depth < ?2
             )
             SELECT path, MIN(depth) AS depth
             FROM reach
             GROUP BY path
             ORDER BY depth, path",
        )?;
        let rows = stmt
            .query_map(params![start, depth], |row| {
                Ok(GraphNode {
                    path: row.get(0)?,
                    depth: row.get(1)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(rows)
    }

    /// Rebuild the entire index from scratch by re-reading the OKF bundle.
    ///
    /// This is the "provably disposable" recovery path: drop all tables,
    /// recreate them, then walk `bundle_root` parsing every conformant
    /// Markdown file, indexing its content, and rebuilding edges from inline
    /// Markdown links. `.haven` and `.git` subtrees are skipped. Non-conformant
    /// or unreadable files are skipped (permissive read), never fatal.
    pub async fn rebuild_from_bundle(&self, bundle_root: &Path) -> Result<(), Error> {
        // 1. Reset schema: drop, recreate, stamp version. No data migrations.
        {
            let conn = self.inner.lock().await;
            Self::drop_tables(&conn)?;
            Self::create_tables(&conn)?;
            Self::stamp_version(&conn)?;
        }

        // 2. Walk the bundle off the async runtime (heavy, synchronous FS scan).
        let root = bundle_root.to_path_buf();
        let entries = tokio::task::spawn_blocking(move || collect_markdown(&root)).await??;

        // 3. Parse + index in a single transaction (no lock held across awaits).
        {
            let mut conn = self.inner.lock().await;
            let tx = conn.transaction()?;
            for (rel_path, raw) in &entries {
                // Permissive read: a non-conformant file must not abort a rebuild.
                let doc = match okf::parse(raw) {
                    Ok(d) => d,
                    Err(_) => continue,
                };
                let title = doc
                    .frontmatter
                    .title
                    .as_deref()
                    .unwrap_or(rel_path.as_str());
                let doc_type = doc.frontmatter.doc_type.as_str();
                upsert_doc(&tx, rel_path, title, doc_type, &doc.body)?;

                // Rebuild edges from inline Markdown links. External/anchor
                // targets are filtered; remaining targets are normalized to
                // bundle-relative POSIX paths so they match indexed doc paths.
                for target in okf::extract_links(&doc.body)
                    .into_iter()
                    .filter_map(|l| normalize_link(&l))
                {
                    let _ = tx.execute(
                        "INSERT OR IGNORE INTO edges (source, target, context) VALUES (?1, ?2, NULL)",
                        params![rel_path.as_str(), target.as_str()],
                    );
                }
            }
            tx.commit()?;
        }

        Ok(())
    }

    /// Delete the index database files. Best-effort: on platforms that hold an
    /// exclusive handle, removal may require dropping the engine first.
    pub async fn delete(&self) -> Result<(), Error> {
        let path = self.db_path.clone();
        drop(self.inner.clone());
        // Give SQLite a moment to release handles.
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(format!("{}-wal", path.display()));
        let _ = std::fs::remove_file(format!("{}-shm", path.display()));
        Ok(())
    }

    /// Create all index tables (idempotent).
    fn create_tables(conn: &rusqlite::Connection) -> Result<(), Error> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS _meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS docs (
                path TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                doc_type TEXT NOT NULL,
                content TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            -- Standalone FTS5 (not external-content) so writes populate the
            -- search index directly; the index is disposable so the duplicated
            -- content storage is an acceptable trade-off for Phase 1.
            CREATE VIRTUAL TABLE IF NOT EXISTS docs_fts USING fts5(
                path, title, content
            );

            CREATE TABLE IF NOT EXISTS edges (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source TEXT NOT NULL,
                target TEXT NOT NULL,
                context TEXT,
                UNIQUE(source, target)
            );

            CREATE INDEX IF NOT EXISTS idx_edges_source ON edges(source);
            CREATE INDEX IF NOT EXISTS idx_edges_target ON edges(target);",
        )?;

        #[cfg(feature = "vector")]
        {
            init_vector(conn)?;
        }

        Ok(())
    }

    /// Drop all index tables (idempotent).
    fn drop_tables(conn: &rusqlite::Connection) -> Result<(), Error> {
        #[cfg(feature = "vector")]
        {
            let _ = conn.execute_batch("DROP TABLE IF EXISTS vec_items;");
        }
        conn.execute_batch(
            "DROP TABLE IF EXISTS edges;
             DROP TABLE IF EXISTS docs_fts;
             DROP TABLE IF EXISTS docs;
             DROP TABLE IF EXISTS _meta;",
        )?;
        Ok(())
    }

    /// Stamp the current schema version into `_meta`.
    fn stamp_version(conn: &rusqlite::Connection) -> Result<(), Error> {
        conn.execute(
            "INSERT OR REPLACE INTO _meta (key, value) VALUES ('schema_version', ?1)",
            [INDEX_SCHEMA_VERSION.to_string()],
        )?;
        Ok(())
    }
}

/// Insert or replace a document row and keep the FTS index in sync.
fn upsert_doc(
    conn: &rusqlite::Connection,
    path: &str,
    title: &str,
    doc_type: &str,
    content: &str,
) -> Result<(), Error> {
    conn.execute(
        "INSERT OR REPLACE INTO docs (path, title, doc_type, content, updated_at)
         VALUES (?1, ?2, ?3, ?4, datetime('now'))",
        params![path, title, doc_type, content],
    )?;
    conn.execute("DELETE FROM docs_fts WHERE path = ?1", params![path])?;
    conn.execute(
        "INSERT INTO docs_fts (path, title, content) VALUES (?1, ?2, ?3)",
        params![path, title, content],
    )?;
    Ok(())
}

/// Normalize an inline Markdown link target to a bundle-relative POSIX path.
///
/// Returns `None` for external URLs, `mailto:`, and anchor-only targets so they
/// never become graph edges. Fragments are stripped and `./` / leading `/`
/// prefixes are removed so the result matches indexed document paths.
fn normalize_link(target: &str) -> Option<String> {
    let t = target.trim();
    if t.is_empty() || t.starts_with('#') || t.starts_with("mailto:") || t.contains("://") {
        return None;
    }
    // Strip a trailing/inline fragment (e.g. `notes/b.md#section` -> `notes/b.md`).
    let t = t.split('#').next().unwrap_or(t).trim();
    if t.is_empty() {
        return None;
    }
    let t = t.replace('\\', "/");
    let t = t.trim_start_matches("./");
    let t = t.trim_start_matches('/');
    if t.is_empty() {
        return None;
    }
    Some(t.to_string())
}

/// Recursively walk `root`, returning `(bundle_relative_path, raw_content)` for
/// every `.md` file. Skips `.haven` and `.git` subtrees entirely. Unreadable
/// files and directories are skipped (permissive), never fatal.
fn collect_markdown(root: &Path) -> Result<Vec<(String, String)>, Error> {
    let mut out = Vec::new();
    let mut stack: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if path.is_dir() {
                if name_str == ".haven" || name_str == ".git" {
                    continue;
                }
                stack.push(path);
            } else if path.is_file() && name_str.ends_with(".md") {
                let rel = match path.strip_prefix(root) {
                    Ok(r) => r.to_string_lossy().replace('\\', "/"),
                    Err(_) => continue,
                };
                match fs::read_to_string(&path) {
                    Ok(content) => out.push((rel, content)),
                    Err(_) => continue,
                }
            }
        }
    }
    Ok(out)
}

#[cfg(feature = "vector")]
impl IndexEngine {
    /// Store (or replace) a 768-dim embedding for `path`. Placeholder storage;
    /// vectors are re-derivable from the bundle so this is disposable.
    pub async fn store_embedding(&self, path: &str, embedding: &[f32]) -> Result<(), Error> {
        const EMBEDDING_DIM: usize = 768;
        if embedding.len() != EMBEDDING_DIM {
            return Err(Error::InvalidVectorDimension {
                got: embedding.len(),
                expected: EMBEDDING_DIM,
            });
        }
        // vec0 accepts a JSON array literal for float columns.
        let json = format!(
            "[{}]",
            embedding
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );
        let conn = self.inner.lock().await;
        conn.execute("DELETE FROM vec_items WHERE doc_path = ?1", params![path])?;
        conn.execute(
            "INSERT INTO vec_items (doc_path, embedding) VALUES (?1, ?2)",
            params![path, json],
        )?;
        Ok(())
    }
}

/// Register the `vec0` module and create the vector table. Only linked under
/// the `vector` feature to keep default builds non-blocking.
#[cfg(feature = "vector")]
fn init_vector(conn: &rusqlite::Connection) -> Result<(), Error> {
    sqlite_vec::init(conn)?;
    conn.execute_batch(
        "CREATE VIRTUAL TABLE IF NOT EXISTS vec_items USING vec0(
            doc_path TEXT,
            embedding float[768]
        );",
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_bundle(root: &Path) {
        std::fs::create_dir_all(root).unwrap();
        std::fs::write(
            root.join("index.md"),
            "---\ntype: index\ntitle: Home\n---\n\nStart at [note a](notes/a.md) and [note b](notes/b.md).\n",
        )
        .unwrap();
        std::fs::create_dir_all(root.join("notes")).unwrap();
        std::fs::write(
            root.join("notes/a.md"),
            "---\ntype: note\ntitle: Note A\n---\n\nAlpha content. Links to [b](notes/b.md). External [ex](https://example.com).\n",
        )
        .unwrap();
        std::fs::write(
            root.join("notes/b.md"),
            "---\ntype: note\ntitle: Note B\n---\n\nBeta content. See [c](notes/c.md).\n",
        )
        .unwrap();
        std::fs::write(
            root.join("notes/c.md"),
            "---\ntype: note\ntitle: Note C\n---\n\nGamma content. No links here.\n",
        )
        .unwrap();
        std::fs::create_dir_all(root.join(".haven")).unwrap();
        std::fs::write(
            root.join(".haven/junk.md"),
            "---\ntype: note\ntitle: Junk\n---\n\njunk content must not be indexed.\n",
        )
        .unwrap();
        std::fs::create_dir_all(root.join(".git")).unwrap();
        std::fs::write(
            root.join(".git/junk.md"),
            "---\ntype: note\ntitle: Git Junk\n---\n\ngit junk content must not be indexed.\n",
        )
        .unwrap();
    }

    #[tokio::test]
    async fn open_and_index_document() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join(".haven").join("index.db");

        let engine = IndexEngine::open(&db_path).await.unwrap();
        engine
            .index_document(
                "notes/test.md",
                "Test Note",
                "note",
                "This is the content of my test note.",
            )
            .await
            .unwrap();

        let results = engine.search("test note").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Test Note");
    }

    #[tokio::test]
    async fn delete_and_rebuild_is_idempotent() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join(".haven").join("index.db");

        {
            let engine = IndexEngine::open(&db_path).await.unwrap();
            engine
                .index_document("notes/a.md", "Doc A", "note", "Content A")
                .await
                .unwrap();
            assert_eq!(engine.search("Content A").await.unwrap().len(), 1);
        } // engine dropped -> connection closed

        // Physically remove the disposable index, then reopen -> empty.
        let _ = std::fs::remove_file(&db_path);
        let _ = std::fs::remove_file(format!("{}-wal", db_path.display()));
        let _ = std::fs::remove_file(format!("{}-shm", db_path.display()));

        let engine2 = IndexEngine::open(&db_path).await.unwrap();
        assert!(engine2.search("Content A").await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn rebuild_indexes_docs_and_skips_reserved_dirs() {
        let dir = TempDir::new().unwrap();
        let bundle = dir.path().join("bundle");
        write_bundle(&bundle);
        let db_path = dir.path().join(".haven").join("index.db");

        let engine = IndexEngine::open(&db_path).await.unwrap();
        engine.rebuild_from_bundle(&bundle).await.unwrap();

        let alpha = engine.search("alpha").await.unwrap();
        assert_eq!(alpha.len(), 1);
        assert_eq!(alpha[0].path, "notes/a.md");

        // Reserved dirs (.haven, .git) must be ignored.
        let junk = engine.search("junk").await.unwrap();
        assert!(junk.is_empty(), "reserved dir content must not be indexed");
    }

    #[tokio::test]
    async fn backlinks_and_outgoing_edges() {
        let dir = TempDir::new().unwrap();
        let bundle = dir.path().join("bundle");
        write_bundle(&bundle);
        let db_path = dir.path().join(".haven").join("index.db");

        let engine = IndexEngine::open(&db_path).await.unwrap();
        engine.rebuild_from_bundle(&bundle).await.unwrap();

        // index.md -> notes/a.md, notes/b.md
        let out = engine.outgoing_links("index.md").await.unwrap();
        let targets: Vec<String> = out.iter().map(|e| e.target.clone()).collect();
        assert!(targets.contains(&"notes/a.md".to_string()));
        assert!(targets.contains(&"notes/b.md".to_string()));

        // notes/a.md -> notes/b.md, and external URL must be filtered out.
        let out_a = engine.outgoing_links("notes/a.md").await.unwrap();
        assert!(out_a.iter().any(|e| e.target == "notes/b.md"));
        assert!(!out_a.iter().any(|e| e.target.contains("example.com")));

        // backlinks to notes/b.md <- index.md and notes/a.md
        let bl = engine.backlinks("notes/b.md").await.unwrap();
        let sources: Vec<String> = bl.iter().map(|e| e.source.clone()).collect();
        assert!(sources.contains(&"index.md".to_string()));
        assert!(sources.contains(&"notes/a.md".to_string()));
    }

    #[tokio::test]
    async fn graph_reachable_traverses_edges() {
        let dir = TempDir::new().unwrap();
        let bundle = dir.path().join("bundle");
        write_bundle(&bundle);
        let db_path = dir.path().join(".haven").join("index.db");

        let engine = IndexEngine::open(&db_path).await.unwrap();
        engine.rebuild_from_bundle(&bundle).await.unwrap();

        let reach = engine.graph_reachable("notes/a.md", 5).await.unwrap();
        let by_path: std::collections::HashMap<String, i64> =
            reach.iter().map(|n| (n.path.clone(), n.depth)).collect();
        assert_eq!(by_path.get("notes/a.md"), Some(&0));
        assert_eq!(by_path.get("notes/b.md"), Some(&1));
        assert_eq!(by_path.get("notes/c.md"), Some(&2));
        // Edges are directional; index.md is not reachable from notes/a.md.
        assert!(!by_path.contains_key("index.md"));
    }

    #[tokio::test]
    async fn rebuild_is_disposable_and_reconstructs_state() {
        let dir = TempDir::new().unwrap();
        let bundle = dir.path().join("bundle");
        write_bundle(&bundle);
        let db_path = dir.path().join(".haven").join("index.db");

        // First build.
        let engine = IndexEngine::open(&db_path).await.unwrap();
        engine.rebuild_from_bundle(&bundle).await.unwrap();
        let search1 = engine.search("beta").await.unwrap();
        let bl1 = engine.backlinks("notes/b.md").await.unwrap().len();
        let reach1 = engine.graph_reachable("notes/a.md", 5).await.unwrap().len();
        drop(engine);

        // Simulate "delete .haven" once the connection is closed.
        let _ = std::fs::remove_file(&db_path);
        let _ = std::fs::remove_file(format!("{}-wal", db_path.display()));
        let _ = std::fs::remove_file(format!("{}-shm", db_path.display()));

        // Reindex from the bundle alone — state must be fully reconstructed.
        let engine2 = IndexEngine::open(&db_path).await.unwrap();
        engine2.rebuild_from_bundle(&bundle).await.unwrap();
        let search2 = engine2.search("beta").await.unwrap();
        let bl2 = engine2.backlinks("notes/b.md").await.unwrap().len();
        let reach2 = engine2
            .graph_reachable("notes/a.md", 5)
            .await
            .unwrap()
            .len();

        assert_eq!(search1.len(), search2.len());
        assert_eq!(bl1, bl2);
        assert_eq!(reach1, reach2);
        assert!(search2.iter().any(|r| r.path == "notes/b.md"));
    }
}
