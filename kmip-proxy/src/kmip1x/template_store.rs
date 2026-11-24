//! Thread-safe local store emulating KMIP 1.X `Template` objects (removed in KMIP 2.0).
//!
//! Intercepts `Register(Template)`, `Locate(Template)`, `Get(Template)`, `Get Attributes`,
//! `Get Attribute List`, and `Destroy(Template)`, and merges stored template attributes into
//! `Create`, `Create Key Pair`, `Register`, `Re-key`, `Re-key Key Pair`, `Derive Key`,
//! `Certify`, `Re-certify`, `Create Split Key`, `Join Split Key`, and `Import`.
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};

use kmipkit_ttlv::{Item, Value};

use crate::kmip1x::tags::{
    OBJECT_TYPE_TEMPLATE, TAG_ATTRIBUTE, TAG_ATTRIBUTE_NAME, TAG_NAME, TAG_NAME_TYPE,
    TAG_NAME_VALUE, TAG_TEMPLATE, TAG_UNIQUE_IDENTIFIER,
};
use crate::kmip1x::ttlv_helpers::{
    clone_item, extract_text_string, find_child, find_children, ttlv_item, ttlv_struct,
};

#[derive(Debug)]
pub struct StoredTemplate {
    pub unique_id: String,
    pub names: Vec<String>,
    /// Original KMIP 1.X `Attribute` structures stored in the template.
    pub attributes_1x: Vec<Item>,
}

impl StoredTemplate {
    pub fn try_clone(&self) -> Result<Self, String> {
        let mut attrs = Vec::with_capacity(self.attributes_1x.len());
        for a in &self.attributes_1x {
            attrs.push(clone_item(a)?);
        }
        Ok(Self {
            unique_id: self.unique_id.clone(),
            names: self.names.clone(),
            attributes_1x: attrs,
        })
    }

    /// Reconstructs a KMIP 1.X `Template` object structure (`0x420090`) for `Get`.
    pub fn to_template_object(&self) -> Result<Item, String> {
        let mut children = Vec::with_capacity(self.attributes_1x.len());
        for a in &self.attributes_1x {
            children.push(clone_item(a)?);
        }
        ttlv_struct(TAG_TEMPLATE, children)
    }

    /// Returns all KMIP 1.X attribute names present in this template.
    #[must_use]
    pub fn attribute_names(&self) -> Vec<String> {
        let mut names = vec!["Unique Identifier".to_string(), "Object Type".to_string()];
        if !self.names.is_empty() {
            names.push("Name".to_string());
        }
        for attr in &self.attributes_1x {
            if let Some(name_item) = find_child(attr, TAG_ATTRIBUTE_NAME) {
                if let Some(n) = extract_text_string(&name_item) {
                    if !names.contains(&n) {
                        names.push(n);
                    }
                }
            }
        }
        names
    }
}

#[derive(Debug, Default)]
struct TemplateStoreInner {
    by_id: BTreeMap<String, StoredTemplate>,
    id_by_name: BTreeMap<String, String>,
}

/// Thread-safe in-memory KMIP 1.X `Template` registry.
pub struct TemplateStore {
    next_id: AtomicU64,
    inner: RwLock<TemplateStoreInner>,
}

impl Default for TemplateStore {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateStore {
    #[must_use]
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1),
            inner: RwLock::new(TemplateStoreInner::default()),
        }
    }

    /// Registers a new KMIP 1.X Template and returns its generated Unique Identifier.
    pub fn register(&self, names: Vec<String>, attributes_1x: Vec<Item>) -> String {
        let seq = self.next_id.fetch_add(1, Ordering::Relaxed);
        let unique_id = format!("kmip1x-template-{seq:06}");

        let stored = StoredTemplate {
            unique_id: unique_id.clone(),
            names: names.clone(),
            attributes_1x,
        };

        if let Ok(mut guard) = self.inner.write() {
            for name in names {
                guard.id_by_name.insert(name, unique_id.clone());
            }
            guard.by_id.insert(unique_id.clone(), stored);
        }

        unique_id
    }

    #[must_use]
    pub fn contains_id(&self, unique_id: &str) -> bool {
        self.inner
            .read()
            .map(|g| g.by_id.contains_key(unique_id))
            .unwrap_or(false)
    }

    #[must_use]
    pub fn get_by_id(&self, unique_id: &str) -> Option<StoredTemplate> {
        self.inner
            .read()
            .ok()?
            .by_id
            .get(unique_id)
            .and_then(|t| t.try_clone().ok())
    }

    #[must_use]
    pub fn get_by_name(&self, name: &str) -> Option<StoredTemplate> {
        let guard = self.inner.read().ok()?;
        let id = guard.id_by_name.get(name)?;
        guard.by_id.get(id).and_then(|t| t.try_clone().ok())
    }

    /// Locates stored templates matching an optional Name filter.
    #[must_use]
    pub fn locate(&self, name_filter: Option<&str>) -> Vec<String> {
        let Ok(guard) = self.inner.read() else {
            return Vec::new();
        };
        if let Some(name) = name_filter {
            return guard.id_by_name.get(name).cloned().into_iter().collect();
        }
        guard.by_id.keys().cloned().collect()
    }

    /// Destroys a stored template by Unique Identifier.
    pub fn destroy(&self, unique_id: &str) -> bool {
        let Ok(mut guard) = self.inner.write() else {
            return false;
        };
        if let Some(tpl) = guard.by_id.remove(unique_id) {
            for name in tpl.names {
                if guard.id_by_name.get(&name).map(String::as_str) == Some(unique_id) {
                    guard.id_by_name.remove(&name);
                }
            }
            return true;
        }
        false
    }

    /// Builds KMIP 1.X `Get Attributes` response items for a locally stored Template.
    pub fn build_get_attributes_items(
        &self,
        unique_id: &str,
        requested_names: &[String],
    ) -> Result<Option<Vec<Item>>, String> {
        let Some(tpl) = self.get_by_id(unique_id) else {
            return Ok(None);
        };

        let mut all_attrs = Vec::new();
        all_attrs.push(ttlv_struct(
            TAG_ATTRIBUTE,
            vec![
                ttlv_item(
                    TAG_ATTRIBUTE_NAME,
                    Value::text_string("Unique Identifier".to_string()),
                )?,
                ttlv_item(
                    crate::kmip1x::tags::TAG_ATTRIBUTE_VALUE,
                    Value::text_string(tpl.unique_id.clone()),
                )?,
            ],
        )?);
        all_attrs.push(ttlv_struct(
            TAG_ATTRIBUTE,
            vec![
                ttlv_item(
                    TAG_ATTRIBUTE_NAME,
                    Value::text_string("Object Type".to_string()),
                )?,
                ttlv_item(
                    crate::kmip1x::tags::TAG_ATTRIBUTE_VALUE,
                    Value::enumeration(OBJECT_TYPE_TEMPLATE),
                )?,
            ],
        )?);

        for name in &tpl.names {
            let name_struct = ttlv_struct(
                crate::kmip1x::tags::TAG_ATTRIBUTE_VALUE,
                vec![
                    ttlv_item(TAG_NAME_VALUE, Value::text_string(name.clone()))?,
                    ttlv_item(TAG_NAME_TYPE, Value::enumeration(1))?,
                ],
            )?;
            all_attrs.push(ttlv_struct(
                TAG_ATTRIBUTE,
                vec![
                    ttlv_item(TAG_ATTRIBUTE_NAME, Value::text_string("Name".to_string()))?,
                    name_struct,
                ],
            )?);
        }

        all_attrs.extend(tpl.attributes_1x);

        let filtered = if requested_names.is_empty() {
            all_attrs
        } else {
            all_attrs
                .into_iter()
                .filter(|attr| {
                    find_child(attr, TAG_ATTRIBUTE_NAME)
                        .and_then(|n| extract_text_string(&n))
                        .is_some_and(|n| requested_names.iter().any(|req| req == &n))
                })
                .collect()
        };

        let mut payload = vec![ttlv_item(
            TAG_UNIQUE_IDENTIFIER,
            Value::text_string(unique_id.to_string()),
        )?];
        payload.extend(filtered);
        Ok(Some(payload))
    }

    /// Extracts names from a `Template-Attribute` or `Template` structure.
    #[must_use]
    pub fn extract_names_from_structure(container: &Item) -> Vec<String> {
        let mut names = Vec::new();
        for name_item in find_children(container, TAG_NAME) {
            if let Some(nv) = find_child(&name_item, TAG_NAME_VALUE) {
                if let Some(s) = extract_text_string(&nv) {
                    names.push(s);
                }
            }
        }
        // Also inspect Attribute { Name: "Name", Value: { Name Value } }
        for attr in find_children(container, TAG_ATTRIBUTE) {
            let is_name = find_child(&attr, TAG_ATTRIBUTE_NAME)
                .and_then(|n| extract_text_string(&n))
                .is_some_and(|n| n == "Name");
            if is_name {
                if let Some(val) = find_child(&attr, crate::kmip1x::tags::TAG_ATTRIBUTE_VALUE) {
                    if let Some(nv) = find_child(&val, TAG_NAME_VALUE) {
                        if let Some(s) = extract_text_string(&nv) {
                            if !names.contains(&s) {
                                names.push(s);
                            }
                        }
                    }
                }
            }
        }
        names
    }
}
