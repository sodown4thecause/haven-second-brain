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
    /// OKF required: non-empty `type` key. Empty string in permissive reads
    /// when the document did not declare a type.
    pub r#type: String,
    /// OKF required: `okf_version: v0.1`. Empty string when not declared.
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

    pub fn empty() -> Self {
        Self {
            r#type: String::new(),
            okf_version: String::new(),
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
    /// Whether the document carried a `--- ... ---` block at all. Even
    /// an empty-but-present frontmatter fence must round-trip.
    pub had_frontmatter: bool,
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
    let (raw_front, body, had_frontmatter) = split_frontmatter(input)?;
    match mode {
        Mode::StrictWrite => parse_strict(raw_front, body, had_frontmatter),
        Mode::PermissiveRead => parse_permissive(raw_front, body, had_frontmatter),
    }
}

fn parse_strict(raw: &str, body: &str, _had_frontmatter: bool) -> Result<OkfDoc, OkfError> {
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
    if body.trim().is_empty() {
        return Err(OkfError::EmptyBody);
    }
    Ok(OkfDoc {
        frontmatter: deserialize_front_strict(raw)?,
        body: body.to_string(),
        had_frontmatter: true,
    })
}

/// Permissive parse: never throw on missing `type`/`okf_version`/unknown
/// keys. Preserve the user's original keys verbatim in `extra`.
fn parse_permissive(raw: &str, body: &str, had_frontmatter: bool) -> Result<OkfDoc, OkfError> {
    // A document with no frontmatter block parses an empty YAML mapping;
    // give the file a default `type` so the typed doc is not blank-fields.
    if raw.trim().is_empty() {
        return Ok(OkfDoc {
            frontmatter: Frontmatter::new("note"),
            body: body.to_string(),
            had_frontmatter,
        });
    }
    let mut fm = Frontmatter::empty();
    if let Ok(serde_yaml::Value::Mapping(map)) =
        serde_yaml::from_str::<serde_yaml::Value>(raw)
    {
        for (k, v) in map {
            let Some(key) = k.as_str() else {
                // Non-string keys cannot round-trip; skip.
                continue;
            };
            match key {
                "type" => {
                    if let serde_yaml::Value::String(s) = &v {
                        fm.r#type = s.clone();
                    }
                }
                "okf_version" => {
                    if let serde_yaml::Value::String(s) = &v {
                        fm.okf_version = s.clone();
                    }
                }
                "title" => {
                    if let serde_yaml::Value::String(s) = &v {
                        fm.title = Some(s.clone());
                    }
                }
                "description" => {
                    if let serde_yaml::Value::String(s) = &v {
                        fm.description = Some(s.clone());
                    }
                }
                "resource" => {
                    if let serde_yaml::Value::String(s) = &v {
                        fm.resource = Some(s.clone());
                    }
                }
                "tags" => {
                    if let serde_yaml::Value::Mapping(m) = &v {
                        let mut tags = BTreeMap::new();
                        for (tk, tv) in m {
                            let Some(tag) = tk.as_str() else {
                                continue;
                            };
                            if let serde_yaml::Value::Sequence(seq) = tv {
                                let values = seq
                                    .iter()
                                    .filter_map(|x| match x {
                                        serde_yaml::Value::String(s) => Some(s.clone()),
                                        _ => None,
                                    })
                                    .collect();
                                tags.insert(tag.to_string(), values);
                            }
                        }
                        fm.tags = tags;
                    }
                }
                "timestamp" => {
                    if let serde_yaml::Value::String(s) = &v {
                        fm.timestamp = Some(s.clone());
                    }
                }
                _ => {
                    fm.extra.insert(key.to_string(), v);
                }
            }
            }
        }
    }
    Ok(OkfDoc {
        frontmatter: fm,
        body: body.to_string(),
        had_frontmatter,
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

/// Split Markdown into `(frontmatter_raw, body, had_frontmatter)`. OKF allows
/// frontmatter to be absent; we return empty `raw` in that case. A leading
/// `---` without a closing fence is treated as a body-level thematic break.
fn split_frontmatter(input: &str) -> Result<(&str, &str, bool), OkfError> {
    if !input.starts_with("---") {
        return Ok(("", input, false));
    }
    let mut rest = &input[3..];
    if let Some(stripped) = rest.strip_prefix('\n') {
        rest = stripped;
    } else if let Some(stripped) = rest.strip_prefix("\r\n") {
        rest = stripped;
    }
    let mut fence_start: Option<usize> = None;
    let mut idx = 0usize;
    for line in rest.split_inclusive('\n') {
        if line.trim_end_matches(['\n', '\r']).trim() == "---" {
            fence_start = Some(idx);
            break;
        }
        idx += line.len();
    }
    let fence_offset = match fence_start {
        Some(offset) => offset,
        None => return Ok(("", input, false)),
    };
    // YAML content is everything before the closing fence line.
    let raw = &rest[..fence_offset];
    // Body starts after the closing fence line; the line itself is
    // `---\n` (or `---\r\n`); we don't recursively trim inside `raw`,
    // since `fence_offset` is already at the start of `---`.
    let mut after_fence = &rest[fence_offset..];
    if let Some(stripped) = after_fence.strip_prefix("---\r\n") {
        after_fence = stripped;
    } else if let Some(stripped) = after_fence.strip_prefix("---\n") {
        after_fence = stripped;
    } else if let Some(stripped) = after_fence.strip_prefix("---") {
        after_fence = stripped;
    }
    Ok((raw, after_fence, true))
}

/// Reconstruct the canonical Markdown serialization for an `OkfDoc`.
/// Unknown keys in `Frontmatter.extra` are preserved verbatim via serde_yaml.
/// Empty frontmatter fences round-trip as `---\n---\n`.
pub fn serialize(doc: &OkfDoc) -> Result<String, OkfError> {
    if !doc.had_frontmatter {
        return Ok(doc.body.clone());
    }
    if doc.frontmatter.r#type.is_empty()
        && doc.frontmatter.okf_version.is_empty()
        && doc.frontmatter.title.is_none()
        && doc.frontmatter.description.is_none()
        && doc.frontmatter.resource.is_none()
        && doc.frontmatter.tags.is_empty()
        && doc.frontmatter.timestamp.is_none()
        && doc.frontmatter.extra.is_empty()
    {
        return Ok(format!("---\n---\n{}", doc.body));
    }
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
    fn strict_write_rejects_missing_type_before_empty_body() {
        // Missing type must surface even when the body is empty.
        let raw = "---
okf_version: v0.1
---
";
        let err = parse(raw, Mode::StrictWrite).unwrap_err();
        assert!(matches!(err, OkfError::MissingType));
    }

    #[test]
    fn permissive_read_returns_typed_doc_on_garbage() {
        let raw = "not yaml at all\n---\nbody text\n";
        let parsed = parse(raw, Mode::PermissiveRead).expect("ok");
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
    fn permissive_read_preserves_unknown_keys_on_reserialize() {
        let raw = "---\nokf_version: v0.1\ntype: exotic\nmade_up_key: 42\n---\n# body";
        let parsed = parse(raw, Mode::PermissiveRead).expect("ok");
        let serialized = serialize(&parsed).expect("ok");
        let again = parse(&serialized, Mode::PermissiveRead).expect("ok");
        let made_up = again
            .frontmatter
            .extra
            .get("made_up_key")
            .and_then(|v| v.as_i64())
            .unwrap();
        assert_eq!(made_up, 42);
    }

    #[test]
    fn permissive_read_preserves_unknown_keys_when_type_missing() {
        let raw = "---\ntitle: My note\ncustom: 1\n---\nbody\n";
        let parsed = parse(raw, Mode::PermissiveRead).expect("ok");
        assert_eq!(parsed.frontmatter.title.as_deref(), Some("My note"));
        assert!(parsed.frontmatter.extra.contains_key("custom"));
        assert!(parsed.frontmatter.r#type.is_empty());
    }

    #[test]
    fn body_starting_with_dashes_is_not_a_frontmatter() {
        let parsed = parse("---\nbody\n", Mode::PermissiveRead).expect("ok");
        assert!(!parsed.had_frontmatter);
        assert_eq!(parsed.body, "---\nbody\n");
    }

    #[test]
    fn empty_but_present_frontmatter_round_trips() {
        let raw = "---\n---\nbody\n";
        let parsed = parse(raw, Mode::PermissiveRead).expect("ok");
        assert!(parsed.had_frontmatter);
        let serialized = serialize(&parsed).expect("ok");
        let again = parse(&serialized, Mode::PermissiveRead).expect("ok");
        assert!(again.had_frontmatter);
        assert_eq!(again.body, "body\n");
    }

    #[test]
    fn round_trip_preserves_unknown_keys() {
        let parsed = parse(SAMPLE_OKF, Mode::PermissiveRead).expect("ok");
        let serialized = serialize(&parsed).expect("ok");
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
