use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::fmt;

/// The type of a vault note, enforced as an enum to prevent arbitrary values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NoteType {
    SessionLog,
    Idea,
    Research,
    Troubleshooting,
    Manifest,
    Index,
}

impl fmt::Display for NoteType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NoteType::SessionLog => write!(f, "session-log"),
            NoteType::Idea => write!(f, "idea"),
            NoteType::Research => write!(f, "research"),
            NoteType::Troubleshooting => write!(f, "troubleshooting"),
            NoteType::Manifest => write!(f, "manifest"),
            NoteType::Index => write!(f, "index"),
        }
    }
}

/// The status of a vault note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NoteStatus {
    Active,
    Completed,
    Abandoned,
}

impl fmt::Display for NoteStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NoteStatus::Active => write!(f, "active"),
            NoteStatus::Completed => write!(f, "completed"),
            NoteStatus::Abandoned => write!(f, "abandoned"),
        }
    }
}

/// The confidence level of a note's content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Confidence {
    High,
    Medium,
    Low,
}

impl fmt::Display for Confidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Confidence::High => write!(f, "high"),
            Confidence::Medium => write!(f, "medium"),
            Confidence::Low => write!(f, "low"),
        }
    }
}

/// Frontmatter schema for every note in the vault.
///
/// Every note must have this frontmatter at the top of its markdown content.
/// The `updated` field is auto-set on write operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frontmatter {
    /// Tags for categorization (e.g., "project/my-rust-project", "type/session-log")
    pub tags: Vec<String>,
    /// Date the note was created (YYYY-MM-DD)
    pub created: NaiveDate,
    /// Date the note was last updated (YYYY-MM-DD)
    pub updated: NaiveDate,
    /// The type of this note
    #[serde(rename = "type")]
    pub note_type: NoteType,
    /// The status of this note
    pub status: NoteStatus,
    /// Confidence level of the content
    pub confidence: Confidence,
}

impl Frontmatter {
    /// Create a new frontmatter with the current date for created/updated.
    pub fn new(note_type: NoteType, tags: Vec<String>) -> Self {
        let today = chrono::Local::now().date_naive();
        Self {
            tags,
            created: today,
            updated: today,
            note_type,
            status: NoteStatus::Active,
            confidence: Confidence::Medium,
        }
    }

    /// Validate that all required fields are present and valid.
    pub fn validate(&self) -> Result<(), FrontmatterError> {
        if self.tags.is_empty() {
            return Err(FrontmatterError::MissingField("tags".to_string()));
        }

        // Every note should have at least a type/ tag
        let has_type_tag = self.tags.iter().any(|t| t.starts_with("type/"));
        if !has_type_tag {
            return Err(FrontmatterError::MissingTypeTag);
        }

        if self.created > self.updated {
            return Err(FrontmatterError::InvalidDateRange {
                created: self.created.to_string(),
                updated: self.updated.to_string(),
            });
        }

        Ok(())
    }

    /// Parse frontmatter from a markdown string (between --- delimiters).
    ///
    /// Uses serde_yaml for safe deserialization — serde_yaml does not support
    /// dangerous YAML features like custom tags or arbitrary code execution.
    pub fn from_markdown(content: &str) -> Result<Option<Self>, FrontmatterError> {
        if !content.starts_with("---") {
            return Ok(None);
        }

        let rest = &content[3..];
        let end = rest.find("---").ok_or(FrontmatterError::MissingEndDelimiter)?;

        let yaml_str = &rest[..end];
        let fm: Self = serde_yaml::from_str(yaml_str)
            .map_err(|e| FrontmatterError::ParseError(e.to_string()))?;

        fm.validate()?;
        Ok(Some(fm))
    }

    /// Serialize frontmatter to a YAML block with --- delimiters.
    pub fn to_markdown_prefix(&self) -> Result<String, FrontmatterError> {
        let yaml = serde_yaml::to_string(self)
            .map_err(|e| FrontmatterError::SerializeError(e.to_string()))?;
        // serde_yaml adds a leading "---\n" which we handle ourselves
        let yaml = yaml.trim_start_matches("---\n").trim_start_matches("---\r\n");
        Ok(format!("---\n{yaml}---\n"))
    }

    /// Set the updated field to today's date.
    pub fn touch(&mut self) {
        self.updated = chrono::Local::now().date_naive();
    }
}

/// Errors for frontmatter validation and parsing.
#[derive(Debug, thiserror::Error)]
pub enum FrontmatterError {
    #[error("missing required field: {0}")]
    MissingField(String),

    #[error("note must have at least one 'type/' tag")]
    MissingTypeTag,

    #[error("invalid date range: created ({created}) is after updated ({updated})")]
    InvalidDateRange { created: String, updated: String },

    #[error("frontmatter parse error: {0}")]
    ParseError(String),

    #[error("frontmatter serialize error: {0}")]
    SerializeError(String),

    #[error("missing closing --- delimiter in frontmatter")]
    MissingEndDelimiter,

    #[error("invalid note type: {0}")]
    InvalidNoteType(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_frontmatter_parses_correctly() {
        let fm = Frontmatter {
            tags: vec!["project/test".to_string(), "type/session-log".to_string()],
            created: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2025, 6, 15).unwrap(),
            note_type: NoteType::SessionLog,
            status: NoteStatus::Active,
            confidence: Confidence::High,
        };
        assert!(fm.validate().is_ok());
    }

    #[test]
    fn missing_tags_produces_error() {
        let fm = Frontmatter {
            tags: vec![],
            created: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            note_type: NoteType::Idea,
            status: NoteStatus::Active,
            confidence: Confidence::Medium,
        };
        assert!(matches!(fm.validate(), Err(FrontmatterError::MissingField(_))));
    }

    #[test]
    fn missing_type_tag_produces_error() {
        let fm = Frontmatter {
            tags: vec!["project/test".to_string()],
            created: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            note_type: NoteType::Idea,
            status: NoteStatus::Active,
            confidence: Confidence::Medium,
        };
        assert!(matches!(fm.validate(), Err(FrontmatterError::MissingTypeTag)));
    }

    #[test]
    fn invalid_date_range_produces_error() {
        let fm = Frontmatter {
            tags: vec!["type/idea".to_string()],
            created: NaiveDate::from_ymd_opt(2025, 12, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            note_type: NoteType::Idea,
            status: NoteStatus::Active,
            confidence: Confidence::Medium,
        };
        assert!(matches!(fm.validate(), Err(FrontmatterError::InvalidDateRange { .. })));
    }

    #[test]
    fn roundtrip_markdown_frontmatter() {
        let fm = Frontmatter {
            tags: vec!["project/test".to_string(), "type/session-log".to_string()],
            created: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            updated: NaiveDate::from_ymd_opt(2025, 6, 15).unwrap(),
            note_type: NoteType::SessionLog,
            status: NoteStatus::Active,
            confidence: Confidence::High,
        };

        let md = fm.to_markdown_prefix().unwrap();
        let body = "# Hello\nSome content";
        let full = format!("{md}{body}");

        let parsed = Frontmatter::from_markdown(&full).unwrap().unwrap();
        assert_eq!(parsed.tags, fm.tags);
        assert_eq!(parsed.created, fm.created);
        assert_eq!(parsed.updated, fm.updated);
        assert_eq!(parsed.note_type, fm.note_type);
        assert_eq!(parsed.status, fm.status);
        assert_eq!(parsed.confidence, fm.confidence);
    }

    #[test]
    fn no_frontmatter_returns_none() {
        let content = "# Just a note\nNo frontmatter here";
        assert!(Frontmatter::from_markdown(content).unwrap().is_none());
    }

    #[test]
    fn touch_updates_date() {
        let mut fm = Frontmatter::new(NoteType::Idea, vec!["type/idea".to_string()]);
        let original = fm.updated;
        fm.touch();
        assert!(fm.updated >= original);
    }

    #[test]
    fn invalid_note_type_rejected() {
        let yaml = "tags:\n  - type/foobar\ntype: foobar\n";
        let result: Result<Frontmatter, _> = serde_yaml::from_str(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn note_type_display() {
        assert_eq!(NoteType::SessionLog.to_string(), "session-log");
        assert_eq!(NoteType::Idea.to_string(), "idea");
        assert_eq!(NoteType::Troubleshooting.to_string(), "troubleshooting");
    }
}
