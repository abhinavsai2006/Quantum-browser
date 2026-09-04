//! Request Rule Definitions & Tracker Categories

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrackerCategory {
    Advertising,
    Analytics,
    SocialTracker,
    Fingerprinting,
    Cryptomining,
    Malicious,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterAction {
    Allow,
    Block(TrackerCategory),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkFilterRule {
    pub raw_pattern: String,
    pub domain_suffix: String,
    pub path_substring: Option<String>,
    pub category: TrackerCategory,
    pub is_exception: bool,
}

impl NetworkFilterRule {
    pub fn parse_abp_line(line: &str, default_cat: TrackerCategory) -> Option<Self> {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('!') || trimmed.starts_with('#') {
            return None;
        }

        let (is_exception, content) = if let Some(stripped) = trimmed.strip_prefix("@@") {
            (true, stripped)
        } else {
            (false, trimmed)
        };

        if let Some(domain_part) = content.strip_prefix("||") {
            let parts: Vec<&str> = domain_part.split('^').collect();
            let domain_suffix = parts[0].to_lowercase();
            let path_substring = if parts.len() > 1 && !parts[1].is_empty() {
                Some(parts[1].to_string())
            } else {
                None
            };

            Some(NetworkFilterRule {
                raw_pattern: trimmed.to_string(),
                domain_suffix,
                path_substring,
                category: default_cat,
                is_exception,
            })
        } else {
            Some(NetworkFilterRule {
                raw_pattern: trimmed.to_string(),
                domain_suffix: content.to_lowercase(),
                path_substring: None,
                category: default_cat,
                is_exception,
            })
        }
    }
}
