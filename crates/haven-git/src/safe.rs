// crates/haven-git/src/safe.rs - read-only compatibility scan and explicit
// opt-in marker for opening third-party vaults safely.
//
// Implements docs/superpowers/specs/safe-existing-vault.md. The contract is:
// zero mutations to the vault root before the user performs a typed opt-in
// event captured in `.haven/state.json`. The integration test under
// `tests/safe_existing_vault.rs` proves this invariant against the canonical
// Obsidian-shaped fixture at `docs/fixtures/obsidian-readonly/`.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use git2::{Repository, Status, StatusOptions};
use serde::{Deserialize, Serialize};

use crate::{confine, sha256_hex, GitError, Identity, OwnedAllowlist};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OptInMarker {
    pub accepted_at: String,
    pub accepting_identity_hash: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct FrontmatterSummary {
    pub total: usize,
    pub okf_conformant: usize,
    pub non_conformant: usize,
    pub sample: Vec<FrontmatterFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FrontmatterFinding {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LinkSummary {
    pub wikilinks: usize,
    pub markdown_links: usize,
    pub broken_refs: usize,
    pub sample: Vec<LinkFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LinkFinding {
    pub source: String,
    pub target: String,
    pub kind: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyntaxSummary {
    pub embedded_html: usize,
    pub code_fences: usize,
    pub tables: usize,
    pub unsupported_tokens: Vec<TokenFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenFinding {
    pub path: String,
    pub token: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct IndexCoverage {
    pub files: usize,
    pub indexed: usize,
    pub skipped: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DirtyWorktree {
    pub uncommitted_changes: usize,
    pub unpushed_commits: usize,
    pub ignored_layout_hints: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Finding {
    pub severity: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompatibilityReport {
    pub vault_root: String,
    pub observed_at: String,
    pub frontmatter: FrontmatterSummary,
    pub links: LinkSummary,
    pub syntax: SyntaxSummary,
    pub ignored_files: Vec<String>,
    pub index_coverage: IndexCoverage,
    pub dirty_worktree: DirtyWorktree,
    pub findings: Vec<Finding>,
}

impl CompatibilityReport {
    pub fn envelope(&self) -> Result<String, GitError> {
        serde_json::to_string_pretty(self).map_err(json_to_io)
    }
}

fn json_to_io(e: serde_json::Error) -> GitError {
    GitError::Io(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        format!("json: {e}"),
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReadState {
    ReadOnly,
    Indexing,
    WriteEnabled,
    Conflict,
}

impl ReadState {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReadState::ReadOnly => "read-only",
            ReadState::Indexing => "indexing",
            ReadState::WriteEnabled => "write-enabled",
            ReadState::Conflict => "conflict",
        }
    }
}

/// Open a third-party vault for read-only inspection. Performs only file
/// reads. The returned `ReadState::ReadOnly` is durable until the user
/// performs `write_opt_in_marker` and is detected by `opt_in_present`.
pub fn safe_open(vault_root: &Path) -> Result<(CompatibilityReport, ReadState), GitError> {
    let vault_root = confine(vault_root, vault_root)?;
    let report = scan(&vault_root)?;
    let state = if opt_in_present(&vault_root)? {
        ReadState::WriteEnabled
    } else {
        ReadState::ReadOnly
    };
    Ok((report, state))
}

/// Run the indexer in read-only mode. Returns `ReadState::Indexing` while the
/// indexer runs; the caller transitions to `ReadState::ReadOnly` or
/// `ReadState::WriteEnabled` after the index commits` (no writes here).
pub fn index(vault_root: &Path) -> Result<(CompatibilityReport, ReadState), GitError> {
    let vault_root = confine(vault_root, vault_root)?;
    let mut report = scan(&vault_root)?;
    report.index_coverage = index_coverage(&vault_root, &report);
    Ok((report, ReadState::Indexing))
}

/// Detect the workspace dirtiness (uncommitted changes, unpushed commits,
/// known sync-layout hints). Read-only; never stages or commits.
pub fn dirty_worktree_detect(vault_root: &Path) -> Result<DirtyWorktree, GitError> {
    let mut out = DirtyWorktree::default();
    let vault_root = confine(vault_root, vault_root)?;

    if vault_root.join(".git").exists() {
        if let Ok(repo) = Repository::open(&vault_root) {
            let mut opts = StatusOptions::new();
            opts.include_untracked(true).recurse_untracked_dirs(true);
            if let Ok(statuses) = repo.statuses(Some(&mut opts)) {
                out.uncommitted_changes = statuses
                    .iter()
                    .filter(|s| s.status() != Status::CURRENT)
                    .count();
            }
            if let Ok(unpushed) = count_unpushed(&repo, "HEAD", "@{u}") {
                out.unpushed_commits = unpushed;
            }
        }
    }

    for hint in [".obsidian", ".stfolder", ".dropbox.cache", "iCloud Drive"] {
        let probe = vault_root.join(hint);
        if probe.exists() {
            let label = hint.trim_start_matches('.').to_string();
            if !out.ignored_layout_hints.contains(&label) {
                out.ignored_layout_hints.push(label);
            }
        }
    }
    Ok(out)
}

fn count_unpushed(
    repo: &Repository,
    head_spec: &str,
    upstream_spec: &str,
) -> Result<usize, GitError> {
    let head_oid = match repo.revparse_single(head_spec) {
        Ok(obj) => obj.id(),
        Err(_) => return Ok(0),
    };
    let upstream_oid = match repo.revparse_single(upstream_spec) {
        Ok(obj) => obj.id(),
        Err(_) => return Ok(0),
    };
    if head_oid == upstream_oid {
        return Ok(0);
    }
    let mut count = 0usize;
    let mut revwalk = repo.revwalk()?;
    revwalk.push(head_oid)?;
    let upstream_set: std::collections::HashSet<_> = {
        let mut rw = repo.revwalk()?;
        if let Ok(_) = rw.push(upstream_oid) {
            rw.filter_map(|r| r.ok()).collect()
        } else {
            std::collections::HashSet::new()
        }
    };
    for oid in revwalk.filter_map(|r| r.ok()) {
        if upstream_set.contains(&oid) {
            break;
        }
        count += 1;
    }
    Ok(count)
}

/// Read the durable opt-in marker. Confined to the vault root and never
/// overwrites any existing user file outside `.haven/`.
pub fn require_opt_in_marker(
    vault_root: &Path,
    allowlist: &OwnedAllowlist,
) -> Result<OptInMarker, GitError> {
    let marker_path = vault_root.join(".haven").join("state.json");
    let abs = confine(&marker_path, vault_root)?;
    if !allowlist.owns(&abs, vault_root) {
        return Err(GitError::OffTree(abs));
    }
    if !abs.exists() {
        return Err(GitError::OffTree(abs));
    }
    let bytes = std::fs::read(&abs)?;
    let marker: OptInMarker = serde_json::from_slice(&bytes).map_err(json_to_io)?;
    Ok(marker)
}

/// Cheap presence check used by `safe_open` and the UI's trust-state pill.
pub fn opt_in_present(vault_root: &Path) -> Result<bool, GitError> {
    Ok(vault_root.join(".haven").join("state.json").exists())
}

/// Write the opt-in marker. The caller (workflow layer) is responsible for
/// landing the resulting file as a human-authored commit; this function is
/// the typed producer only.
pub fn write_opt_in_marker(
    vault_root: &Path,
    allowlist: &OwnedAllowlist,
    identity: &Identity,
) -> Result<OptInMarker, GitError> {
    let vault_root = confine(vault_root, vault_root)?;
    let marker_path = vault_root.join(".haven").join("state.json");
    let abs = confine(&marker_path, &vault_root)?;
    if !allowlist.owns(&abs, &vault_root) {
        return Err(GitError::OffTree(abs));
    }
    let haven_dir = abs
        .parent()
        .ok_or_else(|| GitError::PathEscape(abs.clone()))?;
    std::fs::create_dir_all(haven_dir)?;

    let seeded = format!("{}|{}", identity.human_name, identity.human_email);
    let accepting_identity_hash = sha256_hex(seeded.as_bytes());
    let accepted_at = unix_epoch_iso(SystemTime::now());
    let marker = OptInMarker {
        accepted_at,
        accepting_identity_hash,
    };
    std::fs::write(
        &abs,
        serde_json::to_string_pretty(&marker).map_err(json_to_io)?,
    )?;
    Ok(marker)
}

fn unix_epoch_iso(now: SystemTime) -> String {
    let secs = now
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("epoch:{secs}")
}

fn scan(vault_root: &Path) -> Result<CompatibilityReport, GitError> {
    let observed_at = unix_epoch_iso(SystemTime::now());
    let mut report = CompatibilityReport {
        vault_root: vault_root.display().to_string(),
        observed_at,
        ..Default::default()
    };
    let walker = walkdir::WalkDir::new(vault_root).follow_links(false);
    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(vault_root)
            .unwrap_or(entry.path())
            .display()
            .to_string();
        let rel_path = rel.replace('\\', "/");
        if rel_path.starts_with(".git/") || rel_path == ".git" {
            report.ignored_files.push(rel_path);
            continue;
        }
        if rel_path.starts_with(".haven/") || rel_path == ".haven" {
            report.ignored_files.push(rel_path);
            continue;
        }
        report.index_coverage.files += 1;
        let bytes = match std::fs::read(entry.path()) {
            Ok(b) => b,
            Err(_) => continue,
        };
        let text = String::from_utf8_lossy(&bytes);
        let starts_with_fence = text.starts_with("---\n") || text.starts_with("---\r\n");
        report.frontmatter.total += if starts_with_fence { 1 } else { 0 };
        report.frontmatter.okf_conformant += if starts_with_fence && text.contains("okf_version") {
            1
        } else {
            0
        };
        report.links.wikilinks += text.matches("[[").count();
        report.links.markdown_links += text.matches("](").count();
        report.syntax.embedded_html += text.matches("<").count();
        report.syntax.code_fences += text.matches("```").count() / 2;
        report.syntax.tables += text.matches('\n').filter(|_| text.contains(" | ")).count();
        if text.contains("<script") {
            report.syntax.unsupported_tokens.push(TokenFinding {
                path: rel_path.clone(),
                token: "<script".into(),
            });
        }
    }
    report.dirty_worktree = dirty_worktree_detect(vault_root)?;
    if report.dirty_worktree.uncommitted_changes > 0 {
        report.findings.push(Finding {
            severity: "warn".into(),
            code: "dirty_worktree".into(),
            message: "Vault has uncommitted changes; Haven will not absorb them".into(),
        });
    }
    if !report.dirty_worktree.ignored_layout_hints.is_empty() {
        report.findings.push(Finding {
            severity: "info".into(),
            code: "sync_layout_hint".into(),
            message: format!(
                "Detected layout hints: {}",
                report.dirty_worktree.ignored_layout_hints.join(", ")
            ),
        });
    }
    Ok(report)
}

fn index_coverage(vault_root: &Path, report: &CompatibilityReport) -> IndexCoverage {
    let mut cov = IndexCoverage::default();
    cov.files = report.index_coverage.files;
    cov.indexed = report.frontmatter.total;
    cov.skipped = cov.files.saturating_sub(cov.indexed);
    let _ = vault_root; // signature reserved for the indexed-fingerprint path
    cov
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn safe_open_returns_read_only_when_no_marker() {
        let td = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(td.path().join("notes")).unwrap();
        let (report, state) = safe_open(td.path()).unwrap();
        assert_eq!(state, ReadState::ReadOnly);
        assert_eq!(
            report.vault_root,
            td.path().canonicalize().unwrap().display().to_string()
        );
    }

    #[test]
    fn write_opt_in_marker_then_require_round_trips() {
        let td = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(td.path().join("notes")).unwrap();
        let id = Identity::founder_default("qwen3.5:4b");
        let allow = OwnedAllowlist::default_vault();
        let marker = write_opt_in_marker(td.path(), &allow, &id).unwrap();
        let required = require_opt_in_marker(td.path(), &allow).unwrap();
        assert_eq!(marker, required);
        assert!(opt_in_present(td.path()).unwrap());
    }

    #[test]
    fn dirty_worktree_detects_unpushed_when_no_upstream() {
        let td = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(td.path().join("notes")).unwrap();
        Repository::init(td.path()).unwrap();
        let dw = dirty_worktree_detect(td.path()).unwrap();
        assert_eq!(dw.unpushed_commits, 0);
    }

    #[test]
    fn safe_open_reports_walk_into_obsidian_subfolder() {
        let td = tempfile::tempdir().unwrap();
        let sub = td.path().join(".obsidian");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(sub.join("app.json"), "{}").unwrap();
        let dw = dirty_worktree_detect(td.path()).unwrap();
        assert!(dw.ignored_layout_hints.contains(&"obsidian".to_string()));
        let _: PathBuf = td.path().to_path_buf();
    }
}
