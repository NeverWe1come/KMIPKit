use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AttemptContext {
    pub run_id: String,
    pub language: String,
    pub encoding: String,
    pub api_level: String,
    pub case_id: String,
    pub unique_suffix: String,
}

impl AttemptContext {
    pub fn new(
        run_id: &str,
        language: &str,
        encoding: &str,
        api_level: &str,
        case_id: &str,
    ) -> Self {
        let unique_suffix = Uuid::new_v4().to_string()[..8].to_string();
        Self {
            run_id: run_id.to_string(),
            language: language.to_string(),
            encoding: encoding.to_string(),
            api_level: api_level.to_string(),
            case_id: case_id.to_string(),
            unique_suffix,
        }
    }

    pub fn generate_unique_name(&self, prefix: &str) -> String {
        format!(
            "{prefix}-{}-{}-{}-{}",
            self.language, self.encoding, self.api_level, self.unique_suffix
        )
    }

    pub fn generate_batch_item_id(&self, index: usize) -> String {
        format!("batch-{}-{index}", self.unique_suffix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attempt_context_generates_distinct_names() {
        let ctx1 = AttemptContext::new("run-1", "rust", "TTLV", "level1", "test-create");
        let ctx2 = AttemptContext::new("run-1", "rust", "TTLV", "level2", "test-create");

        let name1 = ctx1.generate_unique_name("kmipkit-key");
        let name2 = ctx2.generate_unique_name("kmipkit-key");

        assert_ne!(name1, name2);
        assert!(name1.contains("rust-TTLV-level1"));
        assert!(name2.contains("rust-TTLV-level2"));
    }
}
