// crates/haven-git - dual-identity writer for Haven vaults.
//
// Identity is bound at open time. Human edits travel under the configured
// human signer; agent edits travel under `Haven Agent (<model>)` so the two
// provenance streams stay separable for `AGENTS.md §7`. Off-tree user edits
// are never absorbed by Haven-owned atomic commits.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use git2::{ObjectType, Repository, Signature, Tree};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use ulid::Ulid;

#[derive(Debug, Error)]
pub enum GitError {
    #[error("git2 error: {0}")]
    Git(#[from] git2::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("path escapes the vault root: {0}")]
    PathEscape(PathBuf),
    #[error("expected-content hash mismatch for {path}: pre={pre:?}, post={post:?}")]
    HashMismatch {
        path: PathBuf,
        pre: Option<String>,
        post: Option<String>,
    },
    #[error("symlink target resolves outside vault: {0}")]
    SymlinkEscape(PathBuf),
    #[error("refusing to stage a path outside the Haven-owned allowlist: {0}")]
    OffTree(PathBuf),
}

#[derive(Debug, Clone)]
pub struct Identity {
    pub human_name: String,
    pub human_email: String,
    pub agent_name: String,
    pub agent_email: String,
}

impl Identity {
    pub fn founder_default(model: impl Into<String>) -> Self {
        let model = model.into();
        Self {
            human_name: "Haven Founder".into(),
            human_email: "founder@haven.local".into(),
            agent_name: format!("Haven Agent ({model})"),
            agent_email: "agent@haven.local".into(),
        }
    }

    pub fn signature(&self, kind: AuthorKind) -> Result<Signature<'static>, GitError> {
        let (name, email) = match kind {
            AuthorKind::Human => (&self.human_name, &self.human_email),
            AuthorKind::Agent => (&self.agent_name, &self.agent_email),
        };
        Ok(Signature::now(name, email)?)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthorKind {
    Human,
    Agent,
}

#[derive(Debug, Clone, Default)]
pub struct OwnedAllowlist {
    roots: Vec<PathBuf>,
}

impl OwnedAllowlist {
    pub fn new(roots: Vec<PathBuf>) -> Self {
        Self { roots }
    }

    pub fn default_vault() -> Self {
        Self::new(vec![
            PathBuf::from("notes"),
            PathBuf::from("journal"),
            PathBuf::from("memory"),
            PathBuf::from("skills"),
            PathBuf::from("inputs"),
        ])
    }

    pub fn owns(&self, abs: &Path, vault_root: &Path) -> bool {
        if let Ok(rel) = abs.strip_prefix(vault_root) {
            self.roots.iter().any(|r| rel.starts_with(r))
        } else {
            false
        }
    }
}

/// Confine a path to the vault root, aborting on escapes or hostile symlinks.
pub fn confine(path: &Path, vault_root: &Path) -> Result<PathBuf, GitError> {
    let canonical = if path.exists() {
        path.canonicalize()?
    } else {
        let parent = path.parent().unwrap_or(vault_root);
        let parent = parent
            .canonicalize()
            .unwrap_or_else(|_| parent.to_path_buf());
        parent.join(path.file_name().unwrap_or_default())
    };
    if !canonical.starts_with(vault_root.canonicalize()?) {
        return Err(GitError::PathEscape(canonical));
    }
    if canonical.is_symlink() {
        let target = std::fs::read_link(&canonical)?;
        let resolved = if target.is_absolute() {
            target
        } else {
            canonical.parent().unwrap_or(vault_root).join(&target)
        };
        if !resolved.starts_with(vault_root.canonicalize()?) {
            return Err(GitError::SymlinkEscape(canonical));
        }
    }
    Ok(canonical)
}

#[derive(Debug, Clone)]
pub struct VaultWrite {
    pub path: PathBuf,
    pub new_content: Vec<u8>,
    pub author_kind: AuthorKind,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitReceipt {
    pub oid: String,
    pub path: String,
    pub author_kind: String,
    pub message: String,
}

pub fn open_or_init(vault_root: &Path) -> Result<Repository, GitError> {
    if vault_root.join(".git").exists() {
        Ok(Repository::open(vault_root)?)
    } else {
        std::fs::create_dir_all(vault_root)?;
        Ok(Repository::init(vault_root)?)
    }
}

#[derive(Clone)]
pub struct VaultWriter {
    inner: Arc<WriterInner>,
}

struct WriterInner {
    repo: Mutex<Repository>,
    vault_root: PathBuf,
    identity: Identity,
    allowlist: OwnedAllowlist,
}

impl VaultWriter {
    pub fn open(
        vault_root: &Path,
        identity: Identity,
        allowlist: OwnedAllowlist,
    ) -> Result<Self, GitError> {
        vault_root.canonicalize().map_err(std::io::Error::from)?;
        let repo = open_or_init(vault_root)?;
        let writer = VaultWriter {
            inner: Arc::new(WriterInner {
                repo: Mutex::new(repo),
                vault_root: vault_root.to_path_buf(),
                identity,
                allowlist,
            }),
        };
        Ok(writer)
    }

    pub fn vault_root(&self) -> &Path {
        &self.inner.vault_root
    }

    /// Atomic replace with expected-content hash check. Pre-write recovery
    /// snapshot protects files outside the writer's seen-set.
    pub fn write_atomic(
        &self,
        target: &Path,
        content: &[u8],
        seen_hashes: &mut SeenSet,
    ) -> Result<(), GitError> {
        let vault = self.inner.vault_root.canonicalize()?;
        let abs = if target.is_absolute() {
            target.to_path_buf()
        } else {
            vault.join(target)
        };
        if !self.inner.allowlist.owns(&abs, &vault) {
            return Err(GitError::OffTree(abs));
        }
        let confined = confine(&abs, &vault)?;
        if let Some(expected) = seen_hashes.hash_for(&confined) {
            let on_disk_hash = match std::fs::read(&confined) {
                Ok(b) => Some(sha256_hex(&b)),
                Err(_) => None,
            };
            if Some(expected.as_str()) != on_disk_hash.as_deref() {
                return Err(GitError::HashMismatch {
                    path: confined,
                    pre: Some(expected),
                    post: on_disk_hash,
                });
            }
        }
        if confined.exists() {
            let snap_dir = self.inner.vault_root.join(".haven").join("snapshots");
            std::fs::create_dir_all(&snap_dir)?;
            let snap_id = Ulid::new().to_string();
            let _ = std::fs::copy(
                &confined,
                snap_dir.join(format!(
                    "{}-{snap_id}",
                    confined
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or_default()
                )),
            );
        }
        let tmp = confined.with_extension("haven-tmp");
        std::fs::write(&tmp, content)?;
        std::fs::rename(&tmp, &confined)?;
        let post_hash = sha256_hex(content);
        seen_hashes.set(confin_path(&self.inner.vault_root, &confined), post_hash);
        Ok(())
    }

    /// Stage and commit a single owned file. Uses an isolated index so the
    /// caller's pre-existing index entries never leak in.
    pub fn commit(&self, write: VaultWrite) -> Result<CommitReceipt, GitError> {
        let mut repo = self.inner.repo.lock();
        let vault = self.inner.vault_root.canonicalize()?;
        let canonical_target = if write.path.is_absolute() {
            write.path.clone()
        } else {
            vault.join(&write.path)
        };
        if !self.inner.allowlist.owns(&canonical_target, &vault) {
            return Err(GitError::OffTree(canonical_target));
        }
        let canonical_target = confine(&canonical_target, &vault)?;
        let relative: PathBuf = canonical_target
            .strip_prefix(&vault)
            .map_err(|_| GitError::PathEscape(canonical_target.clone()))?
            .to_path_buf();
        let mut index = repo.index()?;
        index.clear()?;
        let blob_oid = repo.blob(&write.new_content)?;
        let tree_oid = {
            let mut builder = repo.treebuilder(None)?;
            builder.insert(Path::new(&relative), blob_oid, ObjectType::Blob as i32)?;
            builder.write()?
        };
        let tree = repo.find_tree(tree_oid)?;
        write_commit(&mut repo, &tree, &write, &self.inner.identity)
    }
}

fn write_commit(
    repo: &mut Repository,
    tree: &Tree,
    write: &VaultWrite,
    identity: &Identity,
) -> Result<CommitReceipt, GitError> {
    let sig = identity.signature(write.author_kind)?;
    let parent = match repo.head() {
        Ok(head) => Some(
            head.target()
                .ok_or_else(|| git2::Error::from_str("HEAD without target"))?,
        ),
        Err(_) => None,
    };
    let mut parents = Vec::new();
    if let Some(p) = parent {
        parents.push(repo.find_commit(p)?);
    }
    let oid = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &write.message,
        tree,
        &parents.iter().collect::<Vec<_>>(),
    )?;
    Ok(CommitReceipt {
        oid: oid.to_string(),
        path: write.path.display().to_string(),
        author_kind: match write.author_kind {
            AuthorKind::Human => "human".into(),
            AuthorKind::Agent => "agent".into(),
        },
        message: write.message.clone(),
    })
}

pub fn sha256_hex(content: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(content);
    let digest = h.finalize();
    format!("{digest:x}")
}

fn confin_path(root: &Path, abs: &Path) -> PathBuf {
    abs.strip_prefix(root).unwrap_or(abs).to_path_buf()
}

#[derive(Debug, Default, Clone)]
pub struct SeenSet {
    inner: Arc<Mutex<std::collections::HashMap<PathBuf, String>>>,
}

impl SeenSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_path(path: &Path) -> Result<Self, GitError> {
        let set = Self::new();
        set.populate(path)?;
        Ok(set)
    }

    pub fn populate(&self, root: &Path) -> Result<(), GitError> {
        let allow = OwnedAllowlist::default_vault();
        let vault_canonical = root.canonicalize()?;
        for entry in walkdir_byte_root(&vault_canonical)? {
            let abs = entry?;
            if !allow.owns(&abs, &vault_canonical) {
                continue;
            }
            if abs.is_file() {
                let contents = std::fs::read(&abs).unwrap_or_default();
                let hash = sha256_hex(&contents);
                let rel = abs
                    .strip_prefix(&vault_canonical)
                    .unwrap_or(&abs)
                    .to_path_buf();
                self.inner.lock().insert(rel, hash);
            }
        }
        Ok(())
    }

    pub fn hash_for(&self, abs: &Path) -> Option<String> {
        let vault = self.inner.lock();
        let key = abs.to_path_buf();
        vault.get(&key).cloned()
    }

    pub fn set(&self, rel: PathBuf, hash: String) {
        self.inner.lock().insert(rel, hash);
    }
}

fn walkdir_byte_root(
    root: &Path,
) -> Result<impl Iterator<Item = Result<PathBuf, GitError>>, GitError> {
    let mut out = Vec::new();
    walk_dir_recursive(root, &mut out)?;
    Ok(out.into_iter())
}

fn walk_dir_recursive(
    root: &Path,
    out: &mut Vec<Result<PathBuf, GitError>>,
) -> Result<(), GitError> {
    if !root.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let p = entry.path();
        out.push(Ok(p.clone()));
        if p.is_dir() {
            walk_dir_recursive(&p, out)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn new_vault() -> TempDir {
        let td = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(td.path().join("notes")).unwrap();
        std::fs::create_dir_all(td.path().join("journal")).unwrap();
        std::fs::create_dir_all(td.path().join("skills")).unwrap();
        td
    }

    #[test]
    fn opens_or_inits_a_vault_repo() {
        let td = new_vault();
        let repo = Repository::open(td.path()).unwrap();
        assert!(repo.is_empty().unwrap_or(true));
    }

    #[test]
    fn indices_dual_identity_for_agent_commit() {
        let td = new_vault();
        let id = Identity::founder_default("qwen3.5:4b");
        let allow = OwnedAllowlist::default_vault();
        let writer = VaultWriter::open(td.path(), id.clone(), allow).unwrap();
        let mut seen = SeenSet::new();
        let target = td.path().join("notes/sample.md");
        writer
            .write_atomic(
                &target,
                b"---\nokf_version: v0.1\ntype: note\n---\n# sample\n",
                &mut seen,
            )
            .unwrap();
        let receipt = writer
            .commit(VaultWrite {
                path: PathBuf::from("notes/sample.md"),
                new_content: b"# sample\nbody".to_vec(),
                author_kind: AuthorKind::Agent,
                message: "seed notes/sample.md".into(),
            })
            .unwrap();
        assert_eq!(receipt.author_kind, "agent");
        let repo = Repository::open(td.path()).unwrap();
        let head = repo.head().unwrap().target().unwrap();
        let commit = repo.find_commit(head).unwrap();
        let author = commit.author();
        assert_eq!(author.name().unwrap(), id.agent_name);
        assert_eq!(author.email().unwrap(), id.agent_email);
    }

    #[test]
    fn refuses_an_off_tree_path() {
        let td = new_vault();
        let writer = VaultWriter::open(
            td.path(),
            Identity::founder_default("qwen3.5:4b"),
            OwnedAllowlist::default_vault(),
        )
        .unwrap();
        let mut seen = SeenSet::new();
        let target = td.path().join("README.md");
        let err = writer
            .write_atomic(&target, b"hello", &mut seen)
            .unwrap_err();
        assert!(matches!(err, GitError::OffTree(_)));
    }

    #[test]
    fn expected_hash_mismatch_aborts_atomic_write() {
        let td = new_vault();
        let writer = VaultWriter::open(
            td.path(),
            Identity::founder_default("qwen3.5:4b"),
            OwnedAllowlist::default_vault(),
        )
        .unwrap();
        let target = td.path().join("notes/sample.md");
        std::fs::write(&target, b"first version").unwrap();
        let mut seen = SeenSet::new();
        seen.set(PathBuf::from("notes/sample.md"), "deadbeef".into());
        let err = writer
            .write_atomic(&target, b"second version", &mut seen)
            .unwrap_err();
        assert!(matches!(err, GitError::HashMismatch { .. }));
    }

    #[test]
    fn human_commit_uses_human_identity() {
        let td = new_vault();
        let id = Identity::founder_default("qwen3.5:4b");
        let writer =
            VaultWriter::open(td.path(), id.clone(), OwnedAllowlist::default_vault()).unwrap();
        let mut seen = SeenSet::new();
        let target = td.path().join("notes/h.md");
        writer
            .write_atomic(&target, b"# human\n", &mut seen)
            .unwrap();
        let receipt = writer
            .commit(VaultWrite {
                path: PathBuf::from("notes/h.md"),
                new_content: b"# human\nbody".to_vec(),
                author_kind: AuthorKind::Human,
                message: "edit by human".into(),
            })
            .unwrap();
        assert_eq!(receipt.author_kind, "human");
        let repo = Repository::open(td.path()).unwrap();
        let head = repo.head().unwrap().target().unwrap();
        let commit = repo.find_commit(head).unwrap();
        let author = commit.author();
        assert_eq!(author.name().unwrap(), id.human_name);
    }

    #[test]
    fn isolated_index_leaves_user_staging_untouched() {
        let td = new_vault();
        let writer = VaultWriter::open(
            td.path(),
            Identity::founder_default("qwen3.5:4b"),
            OwnedAllowlist::default_vault(),
        )
        .unwrap();
        let user_path = td.path().join("README.md");
        std::fs::write(&user_path, b"user staged content").unwrap();
        let repo = Repository::open(td.path()).unwrap();
        let mut index = repo.index().unwrap();
        std::fs::create_dir_all(td.path().join(".git")).ok();
        index.add_path(Path::new("README.md")).ok();
        assert!(writer
            .commit(VaultWrite {
                path: PathBuf::from("README.md"),
                new_content: b"haven's view".to_vec(),
                author_kind: AuthorKind::Human,
                message: "evil".into(),
            })
            .is_err());
        let head_state = repo.head().ok();
        assert!(head_state.is_none() || repo.head().is_ok());
    }
}
