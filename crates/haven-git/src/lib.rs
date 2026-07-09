//! Git repository operations for Haven.
//!
//! Handles repo initialization, dual-identity commits (human vs agent),
//! staged writes, and debounced filesystem watching.

pub mod watch;

use git2::{Commit, Oid, Repository, Signature, StatusOptions, Tree};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("git error: {0}")]
    Git(#[from] git2::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("repository not initialized")]
    NotInitialized,
    #[error("path is not a git repository: {0}")]
    NotARepo(PathBuf),
}

#[derive(Debug, Clone)]
pub struct Author {
    pub name: String,
    pub email: String,
}

impl Author {
    pub fn human(name: &str, email: &str) -> Self {
        Self {
            name: name.to_string(),
            email: email.to_string(),
        }
    }

    pub fn agent(model: &str) -> Self {
        Self {
            name: format!("Haven Agent ({model})"),
            email: "agent@haven.local".to_string(),
        }
    }
}

/// Core git handle wrapping a `git2::Repository`.
///
/// All writes go through this handle. The `repo_path` is the root of the
/// knowledge bundle (the git working directory).
pub struct GitHandle {
    repo: Repository,
    repo_path: PathBuf,
    index_lock: Arc<Mutex<()>>,
}

impl GitHandle {
    /// Open an existing git repository at the given path.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref().to_path_buf();
        let repo = Repository::open(&path).map_err(|_| Error::NotARepo(path.clone()))?;
        Ok(Self {
            repo,
            repo_path: path,
            index_lock: Arc::new(Mutex::new(())),
        })
    }

    /// Initialize a new git repository at the given path.
    pub fn init(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref().to_path_buf();
        std::fs::create_dir_all(&path)?;
        let repo = Repository::init(&path)?;
        Ok(Self {
            repo,
            repo_path: path,
            index_lock: Arc::new(Mutex::new(())),
        })
    }

    /// Return the path to the repo root.
    pub fn repo_path(&self) -> &Path {
        &self.repo_path
    }

    /// Write a file to the working directory and commit it with the given author.
    ///
    /// The `relative_path` is relative to the repo root. The `content` is written
    /// to the file, staged, and committed in a single operation.
    pub async fn commit_file(
        &self,
        relative_path: &str,
        content: &str,
        author: &Author,
        message: &str,
    ) -> Result<Oid, Error> {
        let _lock = self.index_lock.lock().await;

        let abs_path = self.repo_path.join(relative_path);
        if let Some(parent) = abs_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&abs_path, content)?;

        let mut index = self.repo.index()?;
        index.add_path(Path::new(relative_path))?;
        index.write()?;

        let tree_oid = index.write_tree()?;
        let tree = self.repo.find_tree(tree_oid)?;

        let sig = Signature::now(&author.name, &author.email)?;

        let oid = match self.repo.head() {
            Ok(head) => {
                let target = head
                    .target()
                    .ok_or_else(|| Error::Git(git2::Error::from_str("HEAD has no target")))?;
                let parent = self.repo.find_commit(target)?;
                self.repo
                    .commit(Some("HEAD"), &sig, &sig, message, &tree, &[&parent])
            }
            Err(_) => self
                .repo
                .commit(Some("HEAD"), &sig, &sig, message, &tree, &[]),
        }?;

        Ok(oid)
    }

    /// Scan the working directory for changed files (staged, unstaged, and
    /// untracked). Returns bundle-relative paths.
    pub fn status_porcelain(&self) -> Result<Vec<String>, Error> {
        let mut files = Vec::new();
        let mut opts = StatusOptions::new();
        opts.include_untracked(true).recurse_untracked_dirs(true);
        let statuses = self.repo.statuses(Some(&mut opts))?;
        for entry in statuses.iter() {
            if let Some(path) = entry.path() {
                files.push(path.to_string());
            }
        }
        Ok(files)
    }

    /// Get the latest commit hash on HEAD.
    pub fn head_commit(&self) -> Result<Option<Oid>, Error> {
        match self.repo.head() {
            Ok(head) => Ok(Some(head.target().ok_or_else(|| {
                Error::Git(git2::Error::from_str("HEAD has no target"))
            })?)),
            Err(_) => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn init_creates_repo() {
        let dir = TempDir::new().unwrap();
        let handle = GitHandle::init(dir.path()).unwrap();
        assert!(handle.repo_path().exists());
        assert!(dir.path().join(".git").exists());
    }

    #[test]
    fn open_existing_repo() {
        let dir = TempDir::new().unwrap();
        GitHandle::init(dir.path()).unwrap();
        let handle = GitHandle::open(dir.path()).unwrap();
        assert_eq!(handle.repo_path(), dir.path());
    }

    #[test]
    fn open_nonexistent_repo_fails() {
        let dir = TempDir::new().unwrap();
        let result = GitHandle::open(dir.path());
        assert!(matches!(result, Err(Error::NotARepo(_))));
    }

    #[tokio::test]
    async fn commit_file_creates_initial_commit() {
        let dir = TempDir::new().unwrap();
        let handle = GitHandle::init(dir.path()).unwrap();
        let author = Author::human("Alice", "alice@example.com");
        let oid = handle
            .commit_file("notes/test.md", "# Test", &author, "Initial commit")
            .await
            .unwrap();
        assert!(!oid.is_zero());
        assert!(dir.path().join("notes/test.md").exists());
        assert_eq!(handle.head_commit().unwrap(), Some(oid));
    }

    #[tokio::test]
    async fn commit_file_appends_to_history() {
        let dir = TempDir::new().unwrap();
        let handle = GitHandle::init(dir.path()).unwrap();
        let author = Author::human("Alice", "alice@example.com");
        let oid1 = handle
            .commit_file("a.md", "first", &author, "First")
            .await
            .unwrap();
        let oid2 = handle
            .commit_file("a.md", "second", &author, "Second")
            .await
            .unwrap();
        assert_ne!(oid1, oid2);
        assert_eq!(handle.head_commit().unwrap(), Some(oid2));
    }

    #[test]
    fn author_human_identity() {
        let human = Author::human("Bob", "bob@example.com");
        assert_eq!(human.name, "Bob");
        assert_eq!(human.email, "bob@example.com");
    }

    #[test]
    fn author_agent_identity_format() {
        let agent = Author::agent("gemma-4-e4b");
        assert_eq!(agent.name, "Haven Agent (gemma-4-e4b)");
        assert_eq!(agent.email, "agent@haven.local");
    }

    #[tokio::test]
    async fn commit_records_correct_author_identity() {
        let dir = TempDir::new().unwrap();
        let handle = GitHandle::init(dir.path()).unwrap();

        let human = Author::human("Bob", "bob@example.com");
        let agent = Author::agent("gemma-4-e4b");

        let oid_h = handle
            .commit_file("human.md", "x", &human, "human commit")
            .await
            .unwrap();
        let oid_a = handle
            .commit_file("agent.md", "y", &agent, "agent commit")
            .await
            .unwrap();

        let repo = git2::Repository::open(dir.path()).unwrap();
        let commit_h = repo.find_commit(oid_h).unwrap();
        assert_eq!(commit_h.author().name(), Some("Bob"));
        assert_eq!(commit_h.author().email(), Some("bob@example.com"));

        let commit_a = repo.find_commit(oid_a).unwrap();
        assert_eq!(commit_a.author().name(), Some("Haven Agent (gemma-4-e4b)"));
        assert_eq!(commit_a.author().email(), Some("agent@haven.local"));
    }

    #[tokio::test]
    async fn status_porcelain_detects_untracked_files() {
        let dir = TempDir::new().unwrap();
        let handle = GitHandle::init(dir.path()).unwrap();
        let author = Author::human("Alice", "alice@example.com");
        handle
            .commit_file("committed.md", "content", &author, "commit")
            .await
            .unwrap();

        // Create an untracked file.
        std::fs::write(dir.path().join("untracked.md"), "new").unwrap();

        let status = handle.status_porcelain().unwrap();
        assert!(
            status.iter().any(|p| p == "untracked.md"),
            "expected untracked.md in status: {status:?}"
        );
        // The committed file should be clean and absent from status.
        assert!(
            !status.iter().any(|p| p == "committed.md"),
            "committed file should not appear in status: {status:?}"
        );
    }
}
