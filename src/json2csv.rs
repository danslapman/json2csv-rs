use crate::schema::{JsonPath, JsonPathElement};
use serde_json::Value;
use std::collections::HashSet;

fn prepend(prefix: JsonPathElement, path: HashSet<JsonPath>) -> HashSet<JsonPath> {
    if path.is_empty() {
        HashSet::from([vec![prefix]])
    } else {
        path.into_iter()
            .map(|mut p| {
                p.insert(0, prefix.clone());
                p
            })
            .collect()
    }
}

pub fn compute_paths(flat: bool, value: Value) -> Option<HashSet<JsonPath>> {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => Some(HashSet::new()),
        Value::Array(values) => Some(
            values
                .into_iter()
                .map(|v| compute_paths(flat, v))
                .filter_map(|so| so)
                .map(|jp| {
                    if flat {
                        jp
                    } else {
                        prepend(JsonPathElement::Iterator, jp)
                    }
                })
                .fold(HashSet::new(), |mut acc, jp| {
                    acc.extend(jp);
                    acc
                }),
        )
        .filter(|hs| !hs.is_empty()),
        Value::Object(entries) => Some(
            entries
                .into_iter()
                .filter(|(_, value)| non_empty_json(value))
                .filter_map(|(key, value)| compute_paths(false, value).map(|v| (key, v)))
                .map(|(key, value)| prepend(JsonPathElement::Key(key), value))
                .fold(HashSet::new(), |mut acc, jp| {
                    acc.extend(jp);
                    acc
                }),
        )
        .filter(|hs| !hs.is_empty()),
    }
}

pub fn non_empty_json(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Array(values) if values.is_empty() => false,
        Value::Object(entries) if entries.is_empty() => false,
        _ => true,
    }
}

pub fn show_value(value: Value) -> String {
    match value {
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s,
        _ => "".to_string(),
    }
}
