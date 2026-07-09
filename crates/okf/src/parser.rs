use crate::{Document, Error, Frontmatter, OKF_VERSION};

/// Parse an OKF document from raw Markdown text.
///
/// Returns the parsed `Document` on success. On failure, returns an explicit
/// `Error`. The parser is *permissive*: missing optional fields are silently
/// accepted and set to `None`. Unknown frontmatter keys are preserved in
/// `extra` for round-trip fidelity.
pub fn parse(raw: &str) -> Result<Document, Error> {
    let (frontmatter_str, body) = split_frontmatter(raw)?;
    let mut frontmatter: Frontmatter = serde_yaml::from_str(&frontmatter_str)?;

    if frontmatter.doc_type.as_str().is_empty() {
        return Err(Error::EmptyType);
    }

    if frontmatter.okf_version.is_none() {
        frontmatter.okf_version = Some(OKF_VERSION.to_string());
    }

    Ok(Document { frontmatter, body })
}

/// Serialize a `Document` back to OKF-conformant Markdown.
pub fn to_markdown(doc: &Document) -> String {
    // serde_yaml deserializes missing extra keys as `Value::Null`; serializing
    // a Null via `#[serde(flatten)]` can emit a stray `~` entry. Normalize to
    // an empty mapping so the output stays clean on round-trip.
    let mut fm = doc.frontmatter.clone();
    if fm.extra.is_null() {
        fm.extra = serde_yaml::Value::Mapping(serde_yaml::Mapping::new());
    }
    let mut yaml = serde_yaml::to_string(&fm).unwrap_or_default();
    yaml = yaml.trim_end().to_string();
    if doc.body.is_empty() {
        format!("---\n{yaml}\n---\n")
    } else {
        format!("---\n{yaml}\n---\n\n{}", doc.body.trim_end())
    }
}

/// Extract inline Markdown links from the document body.
///
/// Returns a list of link targets from `[text](target)` syntax.
pub fn extract_links(body: &str) -> Vec<String> {
    let re = regex::Regex::new(r"\[([^\]]*)\]\(([^)]+)\)").unwrap();
    re.captures_iter(body)
        .filter_map(|cap| cap.get(2))
        .map(|m| m.as_str().to_string())
        .collect()
}

/// Split raw text into frontmatter (YAML between `---` delimiters) and body.
fn split_frontmatter(raw: &str) -> Result<(String, String), Error> {
    let trimmed = raw.trim_start();
    if !trimmed.starts_with("---") {
        return Err(Error::MissingDelimiter);
    }
    let after_first = &trimmed[3..];
    let end_idx = after_first
        .find("\n---")
        .unwrap_or_else(|| after_first.find("---").unwrap_or(0));
    if end_idx == 0 {
        return Err(Error::MissingDelimiter);
    }
    let frontmatter = after_first[..end_idx].trim().to_string();
    let body_start = end_idx + 4; // skip "\n---"
    let body = after_first[body_start..].trim_start().to_string();
    Ok((frontmatter, body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_note() {
        let raw = "---\ntype: note\ntitle: Test Note\n---\n\nBody content.";
        let doc = parse(raw).unwrap();
        assert_eq!(doc.frontmatter.doc_type, DocType::Note);
        assert_eq!(doc.frontmatter.title.as_deref(), Some("Test Note"));
        assert_eq!(doc.body, "Body content.");
        assert_eq!(doc.frontmatter.okf_version.as_deref(), Some(OKF_VERSION));
    }

    #[test]
    fn round_trip_preserves_unknown_keys() {
        let raw = "---\ntype: note\ntitle: Test\ncustom_field: value\n---\n\nBody.";
        let doc = parse(raw).unwrap();
        let output = to_markdown(&doc);
        assert!(output.contains("custom_field: value"));
        let reparsed = parse(&output).unwrap();
        assert_eq!(reparsed.frontmatter.title.as_deref(), Some("Test"));
    }

    #[test]
    fn missing_type_is_error() {
        let raw = "---\ntitle: No type\n---\n\nBody.";
        assert!(parse(raw).is_err());
    }

    #[test]
    fn extract_links_from_body() {
        let body = "See [here](notes/other.md) and [there](https://example.com).";
        let links = extract_links(body);
        assert_eq!(links, vec!["notes/other.md", "https://example.com"]);
    }

    #[test]
    fn parse_skill_document() {
        let raw = "---\ntype: skill\ntitle: My Skill\ndescription: Does things\n---\n\nSkill body.";
        let doc = parse(raw).unwrap();
        assert_eq!(doc.frontmatter.doc_type, DocType::Skill);
    }

    #[test]
    fn round_trip_no_extra_keys() {
        let raw = "---\ntype: note\ntitle: Simple\n---\n\nBody.";
        let doc = parse(raw).unwrap();
        let output = to_markdown(&doc);
        assert!(
            !output.contains("~"),
            "no stray null marker expected: {output}"
        );
        let reparsed = parse(&output).unwrap();
        assert_eq!(reparsed.frontmatter.doc_type, DocType::Note);
        assert_eq!(reparsed.frontmatter.title.as_deref(), Some("Simple"));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn frontmatter_round_trip(
            type_str in prop_oneof![
                Just("note"),
                Just("skill"),
                Just("observation"),
                Just("template"),
                Just("index"),
                Just("log"),
            ],
            title in "[a-zA-Z0-9 ]{0,40}",
            body in "[a-zA-Z0-9 .,\n]{0,200}",
        ) {
            let raw = format!("---\ntype: {type_str}\ntitle: {title}\n---\n\n{body}");
            let doc1 = parse(&raw).expect("first parse should succeed");
            let md = to_markdown(&doc1);
            let doc2 = parse(&md).expect("second parse should succeed");
            prop_assert_eq!(doc2.frontmatter.doc_type, doc1.frontmatter.doc_type);
            prop_assert_eq!(doc2.frontmatter.title, doc1.frontmatter.title);
            prop_assert_eq!(doc2.body.trim(), doc1.body.trim());
        }

        #[test]
        fn extract_links_finds_all_markdown_links(
            text in "[a-zA-Z0-9 .,]*",
            n_links in 0usize..8,
        ) {
            let mut body = String::new();
            let mut expected = Vec::new();
            for i in 0..n_links {
                body.push_str(&text);
                let target = format!("target{i}.md");
                body.push_str(&format!("[link{i}]({target})"));
                expected.push(target);
                body.push_str(&text);
            }
            let links = extract_links(&body);
            prop_assert_eq!(links, expected);
        }
    }
}
