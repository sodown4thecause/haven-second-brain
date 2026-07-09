use crate::{DocType, Document, OKF_VERSION};

/// OKF conformance linter.
///
/// The linter operates in two modes:
/// - **Strict** (for writes): all required fields must be present; body links
///   must resolve; reserved-file rules apply.
/// - **Permissive** (for reads): only structural issues are flagged; missing
///   optional fields and broken links are ignored per spec §9.
///
/// Lint results carry a severity to allow the editor to surface them
/// appropriately: errors block saves in strict mode, warnings are advisory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct LintMessage {
    pub severity: Severity,
    pub message: String,
}

pub struct Linter {
    mode: Mode,
    known_paths: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Strict,
    Permissive,
}

impl Linter {
    pub fn new(mode: Mode) -> Self {
        Self {
            mode,
            known_paths: Vec::new(),
        }
    }

    pub fn with_known_paths(mut self, paths: Vec<String>) -> Self {
        self.known_paths = paths;
        self
    }

    /// Lint a parsed document. Returns all issues found.
    pub fn lint(&self, doc: &Document) -> Vec<LintMessage> {
        let mut msgs = Vec::new();

        // Structural: frontmatter must have type field
        if doc.frontmatter.doc_type.as_str().is_empty() {
            msgs.push(LintMessage {
                severity: Severity::Error,
                message: "Document is missing required `type` field".into(),
            });
        }

        // Structural: okf_version should be present
        if doc.frontmatter.okf_version.is_none() {
            msgs.push(LintMessage {
                severity: Severity::Error,
                message: format!(
                    "Document is missing `okf_version`; expected \"{}\"",
                    OKF_VERSION
                ),
            });
        }

        if self.mode == Mode::Strict {
            // Recommended fields
            if doc.frontmatter.title.is_none() {
                msgs.push(LintMessage {
                    severity: Severity::Warning,
                    message: "Document is missing recommended field `title`".into(),
                });
            }
            if doc.frontmatter.timestamp.is_none() {
                msgs.push(LintMessage {
                    severity: Severity::Warning,
                    message: "Document is missing recommended field `timestamp`".into(),
                });
            }

            // Link resolution (only when known_paths is populated)
            if !self.known_paths.is_empty() {
                let links = crate::extract_links(&doc.body);
                for link in &links {
                    if !link.starts_with("http") && !self.known_paths.contains(link) {
                        msgs.push(LintMessage {
                            severity: Severity::Warning,
                            message: format!("Link target not found in bundle: {link}"),
                        });
                    }
                }
            }
        }

        msgs
    }

    /// Lint a parsed document, applying reserved-file rules for `index.md` /
    /// `log.md` per OKF spec §6/§7.
    ///
    /// The `path` is the bundle-relative path to the document. Reserved
    /// filenames must carry their matching `type`: `index.md` → `type: index`,
    /// `log.md` → `type: log`. In strict mode a mismatch is an error; in
    /// permissive mode it is a warning (spec §9 — be permissive reading).
    pub fn lint_with_path(&self, doc: &Document, path: &str) -> Vec<LintMessage> {
        let mut msgs = self.lint(doc);

        let filename = std::path::Path::new(path)
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or(path);
        let expected = match filename {
            "index.md" => Some(DocType::Index),
            "log.md" => Some(DocType::Log),
            _ => None,
        };
        if let Some(expected_type) = expected {
            if doc.frontmatter.doc_type != expected_type {
                let severity = if self.mode == Mode::Strict {
                    Severity::Error
                } else {
                    Severity::Warning
                };
                msgs.push(LintMessage {
                    severity,
                    message: format!(
                        "Reserved file `{filename}` must have `type: {expected_type}`, found `type: {}`",
                        doc.frontmatter.doc_type
                    ),
                });
            }
        }

        msgs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DocType, parse};

    fn note_doc(title: &str) -> Document {
        parse(&format!(
            "---\ntype: note\ntitle: {title}\nokf_version: \"{OKF_VERSION}\"\n---\n\nBody."
        ))
        .unwrap()
    }

    #[test]
    fn strict_mode_flags_missing_title() {
        let doc = parse("---\ntype: note\n---\n\nBody.").unwrap();
        let linter = Linter::new(Mode::Strict);
        let msgs = linter.lint(&doc);
        assert!(msgs.iter().any(|m| m.message.contains("title")));
    }

    #[test]
    fn permissive_mode_accepts_missing_title() {
        let doc = parse("---\ntype: note\n---\n\nBody.").unwrap();
        let linter = Linter::new(Mode::Permissive);
        let msgs = linter.lint(&doc);
        assert!(!msgs.iter().any(|m| m.message.contains("title")));
    }

    #[test]
    fn flags_broken_internal_links_in_strict_mode() {
        let doc = parse(
            "---\ntype: note\ntitle: T\nokf_version: \"0.1\"\n---\n\nSee [link](missing.md).",
        )
        .unwrap();
        let linter = Linter::new(Mode::Strict).with_known_paths(vec!["notes/exists.md".into()]);
        let msgs = linter.lint(&doc);
        assert!(msgs.iter().any(|m| m.message.contains("missing.md")));
    }

    #[test]
    fn reserved_index_md_requires_index_type_strict() {
        let doc = note_doc("T");
        let linter = Linter::new(Mode::Strict);
        let msgs = linter.lint_with_path(&doc, "index.md");
        assert!(
            msgs.iter()
                .any(|m| m.severity == Severity::Error && m.message.contains("index.md")),
            "expected error for index.md with non-index type: {msgs:?}"
        );
    }

    #[test]
    fn reserved_log_md_requires_log_type_strict() {
        let doc = note_doc("T");
        let linter = Linter::new(Mode::Strict);
        let msgs = linter.lint_with_path(&doc, "log.md");
        assert!(
            msgs.iter()
                .any(|m| m.severity == Severity::Error && m.message.contains("log.md")),
            "expected error for log.md with non-log type: {msgs:?}"
        );
    }

    #[test]
    fn reserved_file_correct_type_no_error() {
        let doc = parse("---\ntype: index\nokf_version: \"0.1\"\n---\n\nIndex body.").unwrap();
        let linter = Linter::new(Mode::Strict);
        let msgs = linter.lint_with_path(&doc, "index.md");
        assert!(
            !msgs.iter().any(|m| m.message.contains("Reserved file")),
            "no reserved-file error expected when type matches: {msgs:?}"
        );
    }

    #[test]
    fn reserved_file_mismatch_is_warning_in_permissive() {
        let doc = note_doc("T");
        let linter = Linter::new(Mode::Permissive);
        let msgs = linter.lint_with_path(&doc, "log.md");
        assert!(
            msgs.iter()
                .any(|m| m.severity == Severity::Warning && m.message.contains("log.md")),
            "expected warning (not error) in permissive mode: {msgs:?}"
        );
    }

    #[test]
    fn reserved_check_uses_filename_only() {
        let doc = note_doc("T");
        let linter = Linter::new(Mode::Strict);
        let msgs = linter.lint_with_path(&doc, "sub/dir/index.md");
        assert!(msgs.iter().any(|m| m.message.contains("index.md")));
    }
}
