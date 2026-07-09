pub mod linter;
pub mod parser;

pub const OKF_VERSION: &str = "0.1";

/// Core OKF document types as defined in the spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocType {
    Note,
    Skill,
    Observation,
    Index,
    Log,
    Template,
}

impl DocType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Note => "note",
            Self::Skill => "skill",
            Self::Observation => "observation",
            Self::Index => "index",
            Self::Log => "log",
            Self::Template => "template",
        }
    }
}

impl std::fmt::Display for DocType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for DocType {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "note" => Ok(Self::Note),
            "skill" => Ok(Self::Skill),
            "observation" => Ok(Self::Observation),
            "index" => Ok(Self::Index),
            "log" => Ok(Self::Log),
            "template" => Ok(Self::Template),
            "" => Err(Error::EmptyType),
            other => Err(Error::UnknownType(other.to_string())),
        }
    }
}

/// Parsed frontmatter for an OKF document.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Frontmatter {
    #[serde(rename = "type")]
    pub doc_type: DocType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(rename = "okf_version", skip_serializing_if = "Option::is_none")]
    pub okf_version: Option<String>,
    #[serde(flatten)]
    pub extra: serde_yaml::Value,
}

/// A parsed OKF document with frontmatter and body.
#[derive(Debug, Clone)]
pub struct Document {
    pub frontmatter: Frontmatter,
    pub body: String,
}

/// Errors that can occur during OKF operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("missing required frontmatter field 'type'")]
    MissingType,
    #[error("empty 'type' field")]
    EmptyType,
    #[error("unknown document type: {0}")]
    UnknownType(String),
    #[error("YAML parse error: {0}")]
    YamlError(#[from] serde_yaml::Error),
    #[error("missing frontmatter delimiter")]
    MissingDelimiter,
}

/// Re-export common items for convenience.
pub use linter::Linter;
pub use parser::{extract_links, parse, to_markdown};
