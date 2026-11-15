//! Serializable JSON representation of managed objects on disk.

use std::collections::HashMap;

use serde_json::Value;

/// An individual attribute value stored in the object manifest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredAttribute {
    pub name: String,
    pub index: u32,
    pub value: Value,
}

/// A human-readable JSON-persisted managed object.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredObject {
    pub unique_identifier: String,
    pub object_type: String,
    pub state: String,
    pub created_at: i64,
    pub initial_date: Option<i64>,
    pub activation_date: Option<i64>,
    pub deactivation_date: Option<i64>,
    pub destroy_date: Option<i64>,
    pub cryptographic_algorithm: Option<String>,
    pub cryptographic_length: Option<u32>,
    pub cryptographic_usage_mask: Option<u32>,
    pub key_format_type: Option<String>,
    pub key_material_base64: Option<String>,
    pub attributes: HashMap<String, Vec<StoredAttribute>>,
}

impl StoredObject {
    /// Creates a fresh object in `PreActive` or `Active` state.
    pub fn new(
        unique_identifier: impl Into<String>,
        object_type: impl Into<String>,
        state: impl Into<String>,
        created_at: i64,
    ) -> Self {
        Self {
            unique_identifier: unique_identifier.into(),
            object_type: object_type.into(),
            state: state.into(),
            created_at,
            initial_date: Some(created_at),
            activation_date: None,
            deactivation_date: None,
            destroy_date: None,
            cryptographic_algorithm: None,
            cryptographic_length: None,
            cryptographic_usage_mask: None,
            key_format_type: None,
            key_material_base64: None,
            attributes: HashMap::new(),
        }
    }

    /// Converts this object into a formatted JSON value.
    #[must_use]
    pub fn to_json_value(&self) -> Value {
        let mut map = serde_json::Map::new();
        map.insert(
            "unique_identifier".to_owned(),
            Value::String(self.unique_identifier.clone()),
        );
        map.insert(
            "object_type".to_owned(),
            Value::String(self.object_type.clone()),
        );
        map.insert("state".to_owned(), Value::String(self.state.clone()));
        map.insert(
            "created_at".to_owned(),
            Value::Number(self.created_at.into()),
        );

        if let Some(d) = self.initial_date {
            map.insert("initial_date".to_owned(), Value::Number(d.into()));
        }
        if let Some(d) = self.activation_date {
            map.insert("activation_date".to_owned(), Value::Number(d.into()));
        }
        if let Some(d) = self.deactivation_date {
            map.insert("deactivation_date".to_owned(), Value::Number(d.into()));
        }
        if let Some(d) = self.destroy_date {
            map.insert("destroy_date".to_owned(), Value::Number(d.into()));
        }
        if let Some(ref a) = self.cryptographic_algorithm {
            map.insert(
                "cryptographic_algorithm".to_owned(),
                Value::String(a.clone()),
            );
        }
        if let Some(l) = self.cryptographic_length {
            map.insert("cryptographic_length".to_owned(), Value::Number(l.into()));
        }
        if let Some(m) = self.cryptographic_usage_mask {
            map.insert(
                "cryptographic_usage_mask".to_owned(),
                Value::Number(m.into()),
            );
        }
        if let Some(ref f) = self.key_format_type {
            map.insert("key_format_type".to_owned(), Value::String(f.clone()));
        }
        if let Some(ref k) = self.key_material_base64 {
            map.insert("key_material_base64".to_owned(), Value::String(k.clone()));
        }

        let mut attrs_obj = serde_json::Map::new();
        for (name, list) in &self.attributes {
            let arr = list
                .iter()
                .map(|item| {
                    let mut item_map = serde_json::Map::new();
                    item_map.insert("index".to_owned(), Value::Number(item.index.into()));
                    item_map.insert("value".to_owned(), item.value.clone());
                    Value::Object(item_map)
                })
                .collect();
            attrs_obj.insert(name.clone(), Value::Array(arr));
        }
        map.insert("attributes".to_owned(), Value::Object(attrs_obj));

        Value::Object(map)
    }

    /// Reconstructs a stored object from a JSON value.
    pub fn from_json_value(val: &Value) -> Option<Self> {
        let obj = val.as_object()?;
        let unique_identifier = obj.get("unique_identifier")?.as_str()?.to_owned();
        let object_type = obj.get("object_type")?.as_str()?.to_owned();
        let state = obj.get("state")?.as_str()?.to_owned();
        let created_at = obj.get("created_at")?.as_i64()?;

        let mut res = Self::new(unique_identifier, object_type, state, created_at);
        res.initial_date = obj.get("initial_date").and_then(Value::as_i64);
        res.activation_date = obj.get("activation_date").and_then(Value::as_i64);
        res.deactivation_date = obj.get("deactivation_date").and_then(Value::as_i64);
        res.destroy_date = obj.get("destroy_date").and_then(Value::as_i64);
        res.cryptographic_algorithm = obj
            .get("cryptographic_algorithm")
            .and_then(Value::as_str)
            .map(str::to_owned);
        res.cryptographic_length = obj
            .get("cryptographic_length")
            .and_then(Value::as_u64)
            .map(|n| n as u32);
        res.cryptographic_usage_mask = obj
            .get("cryptographic_usage_mask")
            .and_then(Value::as_u64)
            .map(|n| n as u32);
        res.key_format_type = obj
            .get("key_format_type")
            .and_then(Value::as_str)
            .map(str::to_owned);
        res.key_material_base64 = obj
            .get("key_material_base64")
            .and_then(Value::as_str)
            .map(str::to_owned);

        if let Some(attrs_map) = obj.get("attributes").and_then(Value::as_object) {
            for (name, list_val) in attrs_map {
                if let Some(arr) = list_val.as_array() {
                    let list = arr
                        .iter()
                        .filter_map(|item| {
                            let item_obj = item.as_object()?;
                            let index =
                                item_obj.get("index").and_then(Value::as_u64).unwrap_or(0) as u32;
                            let val = item_obj.get("value")?.clone();
                            Some(StoredAttribute {
                                name: name.clone(),
                                index,
                                value: val,
                            })
                        })
                        .collect();
                    res.attributes.insert(name.clone(), list);
                }
            }
        }

        Some(res)
    }
}
