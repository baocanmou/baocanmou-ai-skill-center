use serde::Deserialize;
use std::{collections::HashMap, sync::OnceLock};

#[derive(Deserialize)]
pub struct Description {
    pub source_sha256: String,
    pub name_zh: String,
    pub category: String,
    pub purpose_zh: String,
    pub purpose_en: String,
    pub features_zh: Vec<String>,
    pub features_en: Vec<String>,
}

fn descriptions() -> &'static HashMap<String, Description> {
    static DESCRIPTIONS: OnceLock<HashMap<String, Description>> = OnceLock::new();
    DESCRIPTIONS.get_or_init(|| {
        serde_json::from_str(include_str!("curated_descriptions.json"))
            .expect("validated bundled descriptions")
    })
}

// App-owned explanations, not upstream Skill content. An upstream edit requires review again.
pub fn for_source(id: &str, sha256: &str) -> Option<&'static Description> {
    descriptions()
        .get(id)
        .filter(|description| description.source_sha256 == sha256)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_reviewed_descriptions_are_complete_and_source_bound() {
        assert_eq!(descriptions().len(), 16);
        for (id, description) in descriptions() {
            assert_eq!(description.source_sha256.len(), 64);
            assert!(!description.name_zh.is_empty());
            assert!(!description.purpose_zh.is_empty());
            assert!(!description.purpose_en.is_empty());
            assert_eq!(description.features_zh.len(), 3);
            assert_eq!(description.features_en.len(), 3);
            assert!(for_source(id, &description.source_sha256).is_some());
            assert!(for_source(id, "changed-upstream-content").is_none());
        }
        assert!(for_source("new-unknown-skill", "unknown").is_none());
    }
}
