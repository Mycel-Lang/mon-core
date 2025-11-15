use crate::ast::{Member, MonValue, MonValueKind};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
}

pub(crate) fn to_value(mon_value: &MonValue) -> Value {
    match &mon_value.kind {
        MonValueKind::String(s) => Value::String(s.clone()),
        MonValueKind::Number(n) => Value::Number(*n),
        MonValueKind::Boolean(b) => Value::Boolean(*b),
        MonValueKind::Null => Value::Null,
        MonValueKind::Array(arr) => Value::Array(arr.iter().map(to_value).collect()),
        MonValueKind::Object(obj) => {
            let mut map = BTreeMap::new();
            for member in obj {
                if let Member::Pair(pair) = member {
                    // We only include pairs in the final JSON output.
                    // Type definitions, anchors, etc., are not part of the data.
                    map.insert(pair.key.clone(), to_value(&pair.value));
                }
            }
            Value::Object(map)
        }
        // Aliases, Spreads, etc., should be resolved by this point.
        // If we encounter them here, it's a logic error in the resolver.
        _ => Value::Null, // Or panic, depending on desired strictness.
    }
}
