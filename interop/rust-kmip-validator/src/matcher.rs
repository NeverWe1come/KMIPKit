use crate::models::{ServerCapabilities, TestCase};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchDisposition {
    Supported,
    Unsupported { reasons: Vec<String> },
}

pub struct CapabilityMatcher<'a> {
    capabilities: &'a ServerCapabilities,
}

impl<'a> CapabilityMatcher<'a> {
    pub fn new(capabilities: &'a ServerCapabilities) -> Self {
        Self { capabilities }
    }

    pub fn evaluate(&self, test_case: &TestCase) -> MatchDisposition {
        let mut reasons = Vec::new();

        // 1. Operation requirement
        if let Some(ref op) = test_case.requires.operation {
            let is_supported = self
                .capabilities
                .operations
                .get(op)
                .map(|s| s.supported)
                .unwrap_or(false);
            if !is_supported {
                reasons.push(format!("Operation '{op}' is not supported by server"));
            }
        }

        // 2. Object type requirement
        if let Some(ref obj) = test_case.requires.object_type {
            let is_supported = self
                .capabilities
                .managed_objects
                .get(obj)
                .map(|s| s.supported)
                .unwrap_or(false);
            if !is_supported {
                reasons.push(format!("Managed object '{obj}' is not supported by server"));
            }
        }

        // 3. Attribute requirement
        if let Some(ref attr) = test_case.requires.attribute {
            let is_supported = self
                .capabilities
                .attributes
                .get(attr)
                .map(|s| s.supported)
                .unwrap_or(false);
            if !is_supported {
                reasons.push(format!("Attribute '{attr}' is not supported by server"));
            }
        }

        if reasons.is_empty() {
            MatchDisposition::Supported
        } else {
            MatchDisposition::Unsupported { reasons }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CaseRequirements, IsolationRecipe, OperationSupport, ServerMetadata};
    use std::collections::HashMap;

    #[test]
    fn test_matcher_evaluates_supported_and_unsupported() {
        let mut operations = HashMap::new();
        operations.insert(
            "Create".to_string(),
            OperationSupport {
                supported: true,
                notes: None,
            },
        );
        operations.insert(
            "Archive".to_string(),
            OperationSupport {
                supported: false,
                notes: None,
            },
        );

        let caps = ServerCapabilities {
            schema_version: "1.0".to_string(),
            server: ServerMetadata {
                id: "test-kms".to_string(),
                vendor: "TestVendor".to_string(),
                product: "TestProduct".to_string(),
                version: None,
            },
            transports: None,
            encodings: HashMap::new(),
            operations,
            managed_objects: HashMap::new(),
            attributes: HashMap::new(),
        };

        let matcher = CapabilityMatcher::new(&caps);

        let case_supported = TestCase {
            case_id: "test-create".to_string(),
            category: "lifecycle".to_string(),
            operation: "Create".to_string(),
            oasis_section: None,
            description: None,
            requires: CaseRequirements {
                operation: Some("Create".to_string()),
                ..Default::default()
            },
            isolation_recipe: IsolationRecipe {
                unique_name_prefix: "test".to_string(),
                setup_steps: vec![],
                target_step: "create".to_string(),
                teardown_steps: vec![],
                normalized_fields: vec![],
            },
        };

        let case_unsupported = TestCase {
            case_id: "test-archive".to_string(),
            category: "lifecycle".to_string(),
            operation: "Archive".to_string(),
            oasis_section: None,
            description: None,
            requires: CaseRequirements {
                operation: Some("Archive".to_string()),
                ..Default::default()
            },
            isolation_recipe: IsolationRecipe {
                unique_name_prefix: "test".to_string(),
                setup_steps: vec![],
                target_step: "archive".to_string(),
                teardown_steps: vec![],
                normalized_fields: vec![],
            },
        };

        assert_eq!(
            MatchDisposition::Supported,
            matcher.evaluate(&case_supported)
        );
        assert!(matches!(
            matcher.evaluate(&case_unsupported),
            MatchDisposition::Unsupported { .. }
        ));
    }
}
