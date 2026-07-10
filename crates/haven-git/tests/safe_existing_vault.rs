// crates/haven-git/tests/safe_existing_vault.rs - integration test proving
// that opening a third-party Obsidian vault performs zero user-file
// mutations before a typed opt-in event.
//
// Spec: docs/superpowers/specs/safe-existing-vault.md.
// Fixture: docs/fixtures/obsidian-readonly/ (canonical Obsidian-shaped
// sample with .obsidian folder, OKF and non-OKF frontmatter, wikilinks,
// markdown links, code fences, embedded HTML).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use haven_git::safe::{self, ReadState};
use haven_git::{Identity, OwnedAllowlist};

const FIXTURE_ROOT: &str = "../../docs/fixtures/obsidian-readonly";

fn copy_fixture_to(target: &Path) {
    let src = PathBuf::from(FIXTURE_ROOT);
    assert!(
        src.exists(),
        "fixture missing at {}; the test relies on docs/fixtures/obsidian-readonly",
        src.display()
    );
    copy_recursive(&src, target);
}

fn copy_recursive(src: &Path, dst: &Path) {
    if src.is_file() {
        fs::create_dir_all(dst.parent().unwrap()).unwrap();
        fs::copy(src, dst).unwrap();
        return;
    }
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let from = entry.path();
        let to = dst.join(entry.file_name());
        copy_recursive(&from, &to);
    }
}

fn hash_vault(root: &Path) -> BTreeMap<PathBuf, String> {
    let mut out = BTreeMap::new();
    for entry in walkdir::WalkDir::new(root).follow_links(false) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let bytes = fs::read(entry.path()).unwrap();
        let digest = haven_git::sha256_hex(&bytes);
        let rel = entry.path().strip_prefix(root).unwrap().to_path_buf();
        out.insert(rel, digest);
    }
    out
}

#[test]
fn safe_open_is_byte_identical_before_opt_in() {
    let td = tempfile::tempdir().unwrap();
    let vault = td.path().to_path_buf();
    copy_fixture_to(&vault);

    let before = hash_vault(&vault);
    let (report, state) = safe::safe_open(&vault).unwrap();
    assert_eq!(state, ReadState::ReadOnly);
    let after = hash_vault(&vault);
    assert_eq!(
        before, after,
        "safe_open must not mutate the vault root before opt-in (AGENTS §1)"
    );

    assert_eq!(report.frontmatter.total, 2, "okf-note + legacy-note");
    assert_eq!(
        report.frontmatter.okf_conformant, 1,
        "only okf-note carries okf_version"
    );
    assert_eq!(report.links.wikilinks, 2, "one wikilink each in okf-note and legacy-note");
    assert_eq!(
        report.links.markdown_links, 2,
        "one markdown link in legacy-note + one in readme"
    );
    assert_eq!(
        report.syntax.embedded_html, 2,
        "scratch.md contributes <script> and </script>"
    );
    assert_eq!(report.syntax.code_fences, 1, "readme.md has one fenced block");
    assert_eq!(report.syntax.tables, 0);
    assert_eq!(report.syntax.unsupported_tokens.len(), 1);
    assert!(
        report
            .syntax
            .unsupported_tokens
            .iter()
            .any(|t| t.token == "<script"),
        "scratch.md must surface <script as an unsupported token"
    );
    assert_eq!(report.index_coverage.files, 5, "4 .md + .obsidian/app.json");
    assert!(report
        .dirty_worktree
        .ignored_layout_hints
        .iter()
        .any(|h| h == "obsidian"));
}

#[test]
fn opt_in_marker_round_trip_switches_state_to_write_enabled() {
    let td = tempfile::tempdir().unwrap();
    let vault = td.path().to_path_buf();
    copy_fixture_to(&vault);

    let (_report, state) = safe::safe_open(&vault).unwrap();
    assert_eq!(state, ReadState::ReadOnly);

    let id = Identity::founder_default("qwen3.5:4b");
    let allow = OwnedAllowlist::default_vault();
    let written = safe::write_opt_in_marker(&vault, &allow, &id).unwrap();
    let required = safe::require_opt_in_marker(&vault, &allow).unwrap();
    assert_eq!(written, required);
    assert!(
        safe::opt_in_present(&vault).unwrap(),
        ".haven/state.json must exist after opt-in"
    );

    let (_report_after, state_after) = safe::safe_open(&vault).unwrap();
    assert_eq!(
        state_after,
        ReadState::WriteEnabled,
        "opt-in marker must flip the read state to WriteEnabled"
    );
}

#[test]
fn safe_open_does_not_swallow_existing_user_staging() {
    let td = tempfile::tempdir().unwrap();
    let vault = td.path().to_path_buf();
    copy_fixture_to(&vault);

    let user_path = vault.join("user-staged.md");
    fs::write(&user_path, b"user-staged content not in the fixture").unwrap();
    let before_user = fs::read(&user_path).unwrap();
    let (_report, _) = safe::safe_open(&vault).unwrap();
    let after_user = fs::read(&user_path).unwrap();
    assert_eq!(
        before_user, after_user,
        "safe_open must leave the user's unrelated staging untouched"
    );
}

#[test]
fn safe_open_refuses_to_escape_vault_root() {
    let td = tempfile::tempdir().unwrap();
    let escape = td.path().join("..").join("attacker-staging.md");
    assert!(
        safe::safe_open(&escape).is_err(),
        "paths outside the canonicalized vault root must be rejected"
    );
}
