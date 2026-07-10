//! crates/okf - strict-write / permissive-read OKF v0.1 lint for Haven.
//!
//! This crate only parses Markdown + YAML frontmatter and returns a typed
//! `OkfDoc`. Higher layers (git, index, MCP, editor) own the durable write
//! policy and the OKF versioning logic.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum OkfError {
    #[error("frontmatter is missing the `type` key (OKF strict-write requires it)")]
    MissingType,
    #[error("frontmatter is missing the `okf_version` key (OKF strict-write requires it)")]
    MissingVersion,
    #[error("frontmatter `type` must be a non-empty string")]
    EmptyType,
    #[error("frontmatter `okf_version` must equal {expected}, found {found}")]
    UnsupportedVersion { expected: String, found: String },
    #[error("reserved filename `{path}` must use frontmatter `type: index` or `log`")]
    ReservedFilenameMisTagged { path: String },
    #[error("frontmatter YAML parse failed: {0}")]
    FrontmatterParse(String),
    #[error("document body is empty")]
    EmptyBody,
}

/// Expected OKF schema version this build round-trips. Spec says
/// `okf_version: v0.1` per ADR-001.
pub const OKF_VERSION: &str = "v0.1";

/// Reserved filenames per OKF v0.1.
pub const RESERVED_INDEX: &str = "index.md";
pub const RESERVED_LOG: &str = "log.md";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Frontmatter {
    /// OKF required: non-empty `type` key.
    pub r#type: String,
    /// OKF required: `okf_version: v0.1`.
    pub okf_version: String,
    /// Optional recommended keys per OKF v0.1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub tags: BTreeMap<String, Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    /// Unknown keys preserved verbatim on round-trip; required for
    /// OKF permissive-read invariant.
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_yaml::Value>,
}

impl Frontmatter {
    pub fn new(r#type: impl Into<String>) -> Self {
        Self {
            r#type: r#type.into(),
            okf_version: OKF_VERSION.to_string(),
            title: None,
            description: None,
            resource: None,
            tags: BTreeMap::new(),
            timestamp: None,
            extra: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OkfDoc {
    pub frontmatter: Frontmatter,
    /// Markdown body, kept lossless for round-trip property tests.
    pub body: String,
}

/// Mode decides whether `parse` enforces OKF-strict or accepts permissive reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Used for files Haven is about to write. Refuses missing `type`, missing
    /// `okf_version`, wrong version, or untyped reserved filenames.
    StrictWrite,
    /// Used for files Haven is opening. Unknown `type`, unknown fields, and
    /// missing optional keys must all parse without panic.
    PermissiveRead,
}

pub fn parse(input: &str, mode: Mode) -> Result<OkfDoc, OkfError> {
    let (raw_front, body) = split_frontmatter(input)?;
    if body.trim().is_empty() {
        // Body can be empty in extreme drafts; treat as warning only on
        // permissive reads. Strict-write rejects.
        if matches!(mode, Mode::StrictWrite) {
            return Err(OkfError::EmptyBody);
        }
    }
    match mode {
        Mode::StrictWrite => parse_strict(&raw_front, body, input),
        Mode::PermissiveRead => parse_permissive(&raw_front, body),
    }
}

fn parse_strict(raw: &str, body: &str, full: &str) -> Result<OkfDoc, OkfError> {
    if raw.is_empty() {
        // Strict-write requires frontmatter, but we still want a typed error.
        return Err(OkfError::MissingType);
    }
    let value: serde_yaml::Value =
        serde_yaml::from_str(raw).map_err(|e| OkfError::FrontmatterParse(e.to_string()))?;
    let mut mapping = match value {
        serde_yaml::Value::Mapping(m) => m,
        _ => return Err(OkfError::FrontmatterParse("not a YAML mapping".into())),
    };
    let version = mapping
        .remove("okf_version")
        .ok_or(OkfError::MissingVersion)?;
    let version = match version {
        serde_yaml::Value::String(s) => s,
        _ => {
            return Err(OkfError::FrontmatterParse(
                "okf_version not a string".into(),
            ))
        }
    };
    if version != OKF_VERSION {
        return Err(OkfError::UnsupportedVersion {
            expected: OKF_VERSION.to_string(),
            found: version,
        });
    }
    let r#type = mapping.remove("type").ok_or(OkfError::MissingType)?;
    let r#type_str: String = match r#type {
        serde_yaml::Value::String(s) if !s.is_empty() => s,
        _ => return Err(OkfError::EmptyType),
    };
    let _ = r#type_str;
    let _ = body;
    let _ = full;
    Ok(OkfDoc {
        frontmatter: deserialize_front_strict(raw)?,
        body: body.to_string(),
    })
}

fn parse_permissive(raw: &str, body: &str) -> Result<OkfDoc, OkfError> {
    let fm: Frontmatter = if raw.trim().is_empty() {
        Frontmatter::new("note")
    } else {
        match serde_yaml::from_str(raw) {
            Ok(v) => v,
            Err(_) => Frontmatter::new("unknown"),
        }
    };
    Ok(OkfDoc {
        frontmatter: fm,
        body: body.to_string(),
    })
}

fn deserialize_front_strict(raw: &str) -> Result<Frontmatter, OkfError> {
    let mut fm: Frontmatter =
        serde_yaml::from_str(raw).map_err(|e| OkfError::FrontmatterParse(e.to_string()))?;
    if fm.okf_version != OKF_VERSION {
        return Err(OkfError::UnsupportedVersion {
            expected: OKF_VERSION.to_string(),
            found: fm.okf_version,
        });
    }
    if fm.r#type.is_empty() {
        return Err(OkfError::EmptyType);
    }
    // Sort extras to make the linter deterministic.
    let sorted: BTreeMap<String, serde_yaml::Value> =
        fm.extra.into_iter().collect::<BTreeMap<_, _>>();
    fm.extra = sorted;
    Ok(fm)
}

/// Split Markdown into `(frontmatter_raw, body)`. OKF allows frontmatter to be
/// absent; we return empty `raw` in that case.
fn split_frontmatter(input: &str) -> Result<(&str, &str), OkfError> {
    if !input.starts_with("---") {
        return Ok(("", input));
    }
    // Find the closing fence on its own line.
    let mut rest = &input[3..];
    if let Some(stripped) = rest.strip_prefix('\n') {
        rest = stripped;
    } else if let Some(stripped) = rest.strip_prefix("\r\n") {
        rest = stripped;
    }
    let mut split_at: Option<usize> = None;
    let mut idx = 0usize;
    for line in rest.split_inclusive('\n') {
        if line.trim_end_matches(['\n', '\r']).trim() == "---" {
            split_at = Some(idx + line.len());
            break;
        }
        idx += line.len();
    }
    let Some(offset) = split_at else {
        return Err(OkfError::FrontmatterParse(
            "frontmatter opened with `---` but no closing fence".into(),
        ));
    };
    let body_start_in_rest = offset;
    strip_body(rest, body_start_in_rest)
}

fn strip_body<'a>(rest: &'a str, body_start: usize) -> Result<(&'a str, &'a str), OkfError> {
    let pre_raw_end = body_start.saturating_sub("---".len());
    let raw = &rest[..pre_raw_end];
    let body = &rest[body_start..];
    // Trim the leading newline after the closing fence.
    let body = body
        .strip_prefix("\r\n")
        .or_else(|| body.strip_prefix('\n'))
        .unwrap_or(body);
    Ok((raw, body))
}

/// Reconstruct the canonical Markdown serialization for an `OkfDoc`.
/// Unknown keys in `Frontmatter.extra` are preserved verbatim via serde_yaml.
pub fn serialize(doc: &OkfDoc) -> Result<String, OkfError> {
    let front_yaml = serde_yaml::to_string(&doc.frontmatter)
        .map_err(|e| OkfError::FrontmatterParse(e.to_string()))?;
    Ok(format!("---\n{front_yaml}---\n{}", doc.body))
}

/// Lint a file according to its reserved filename. `index.md` must carry
/// `type: index`; `log.md` must carry `type: log`.
pub fn lint_reserved_filename(path: &str, doc: &Frontmatter) -> Result<(), OkfError> {
    let tail = path.rsplit(['/', '\\']).next().unwrap_or(path);
    match tail {
        RESERVED_INDEX if doc.r#type != "index" => Err(OkfError::ReservedFilenameMisTagged {
            path: tail.to_string(),
        }),
        RESERVED_LOG if doc.r#type != "log" => Err(OkfError::ReservedFilenameMisTagged {
            path: tail.to_string(),
        }),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_OKF: &str = "---
okf_version: v0.1
type: note
title: Sample
tags:
  topics:
    - seeding
---

# Sample body

OKF body goes here.
";

    #[test]
    fn strict_write_accepts_okf_doc() {
        let parsed = parse(SAMPLE_OKF, Mode::StrictWrite).expect("ok");
        assert_eq!(parsed.frontmatter.r#type, "note");
        assert_eq!(parsed.frontmatter.okf_version, "v0.1");
        assert!(parsed.body.contains("# Sample body"));
    }

    #[test]
    fn strict_write_rejects_missing_type() {
        let raw = "---
okf_version: v0.1
---
\n# body";
        let err = parse(raw, Mode::StrictWrite).unwrap_err();
        assert!(matches!(err, OkfError::MissingType));
    }

    #[test]
    fn strict_write_rejects_wrong_version() {
        let raw = "---
okf_version: v0.0
type: note
---
\n# body";
        let err = parse(raw, Mode::StrictWrite).unwrap_err();
        match err {
            OkfError::UnsupportedVersion { expected, found } => {
                assert_eq!(expected, "v0.1");
                assert_eq!(found, "v0.0");
            }
            e => panic!("unexpected error: {e:?}"),
        }
    }

    #[test]
    fn permissive_read_returns_pseudo_frontmatter_on_garbage() {
        let raw = "not yaml at all\n---\nbody text\n";
        let parsed = parse(raw, Mode::PermissiveRead).expect("ok");
        // Permissive reads never panic; a synthetic type is allowed.
        assert!(!parsed.frontmatter.r#type.is_empty());
        assert!(parsed.body.contains("body text"));
    }

    #[test]
    fn permissive_read_accepts_unknown_keys() {
        let raw = "---
okf_version: v0.1
type: exotic
made_up_key: 42
---
\n# body";
        let parsed = parse(raw, Mode::PermissiveRead).expect("ok");
        assert_eq!(parsed.frontmatter.r#type, "exotic");
        assert!(parsed.frontmatter.extra.contains_key("made_up_key"));
    }

    #[test]
    fn round_trip_preserves_unknown_keys() {
        let parsed = parse(SAMPLE_OKF, Mode::PermissiveRead).expect("ok");
        let serialized = serialize(&parsed).expect("ok");
        // Re-parse the same doc permissively and the extras stay.
        let again = parse(&serialized, Mode::PermissiveRead).expect("ok");
        assert_eq!(parsed, again);
    }

    #[test]
    fn reserved_index_requires_index_type() {
        let mut doc = Frontmatter::new("note");
        let _err = lint_reserved_filename("index.md", &doc).unwrap_err();
        doc.r#type = "index".into();
        assert!(lint_reserved_filename("index.md", &doc).is_ok());
    }

    #[test]
    fn reserved_log_requires_log_type() {
        let mut doc = Frontmatter::new("note");
        assert!(lint_reserved_filename("log.md", &doc).is_err());
        doc.r#type = "log".into();
        assert!(lint_reserved_filename("log.md", &doc).is_ok());
    }
}
