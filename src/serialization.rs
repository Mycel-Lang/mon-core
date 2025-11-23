//! # MON AST to Serializable Value Conversion
//!
//! This module provides the logic for converting a resolved MON Abstract Syntax Tree (AST)
//! into a generic, serializable data structure. This is the final step in the pipeline
//! before the MON data can be output as JSON, YAML, or any other format supported by `serde`.
//!
//! ## Architectural Overview
//!
//! The process is straightforward:
//!
//! 1.  A fully resolved and validated [`MonValue`](crate::ast::MonValue) from the AST is passed
//!     to the internal `to_value` function.
//! 2.  The function recursively traverses the `MonValue`, converting it into a tree of
//!     [`Value`] enums.
//! 3.  During this conversion, language-specific AST nodes that are not part of the data model—such
//!     as `TypeDefinition` members—are discarded. Only `Pair` members are included in the final object.
//! 4.  The resulting [`Value`] is designed to be directly serializable by `serde`. It uses a
//!     `BTreeMap` for objects to ensure that the output has a deterministic order of keys,
//!     which is good practice for configuration and data files.
//!
//! ## Use Cases
//!
//! This module is used internally by [`AnalysisResult`](crate::api::AnalysisResult) to provide
//! the `to_json()` and `to_yaml()` methods. Direct interaction with this module is typically not
//! necessary for end-users, as the public API in the [`api`](crate::api) module provides a more
//! convenient interface.
//!
//! ```rust
//! use mon_core::api::analyze;
//!
//! # fn main() -> Result<(), mon_core::error::MonError> {
//! let source = "{ b: 2, a: 1 }";
//!
//! // The serialization module is used behind the scenes by `to_json`.
//! let result = analyze(source, "test.mon")?;
//! let json = result.to_json().unwrap();
//!
//! // Note that the keys are sorted alphabetically because of the BTreeMap.
//! assert_eq!(json, "{\n  \"a\": 1.0,\n  \"b\": 2.0\n}");
//! # Ok(())
//! # }
//! ```
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
        MonValueKind::Null
        | MonValueKind::Alias(_)
        | MonValueKind::EnumValue { .. }
        | MonValueKind::ArraySpread(_) => Value::Null, // Or panic, depending on desired strictness.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Member, MonValue, MonValueKind, Pair};
    use std::collections::BTreeMap;

    fn make_value(kind: MonValueKind) -> MonValue {
        MonValue {
            kind,
            anchor: None,
            pos_start: 0,
            pos_end: 0,
        }
    }

    #[test]
    fn test_string_conversion() {
        let mon_val = make_value(MonValueKind::String("hello".to_string()));
        let result = to_value(&mon_val);
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_number_conversion() {
        let mon_val = make_value(MonValueKind::Number(42.5));
        let result = to_value(&mon_val);
        assert_eq!(result, Value::Number(42.5));
    }

    #[test]
    fn test_boolean_conversion() {
        let mon_val = make_value(MonValueKind::Boolean(true));
        assert_eq!(to_value(&mon_val), Value::Boolean(true));

        let mon_val2 = make_value(MonValueKind::Boolean(false));
        assert_eq!(to_value(&mon_val2), Value::Boolean(false));
    }

    #[test]
    fn test_null_conversion() {
        let mon_val = make_value(MonValueKind::Null);
        assert_eq!(to_value(&mon_val), Value::Null);
    }

    #[test]
    fn test_array_conversion() {
        let arr = vec![
            make_value(MonValueKind::Number(1.0)),
            make_value(MonValueKind::Number(2.0)),
            make_value(MonValueKind::Number(3.0)),
        ];
        let mon_val = make_value(MonValueKind::Array(arr));
        let result = to_value(&mon_val);

        assert_eq!(
            result,
            Value::Array(vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0),
            ])
        );
    }

    #[test]
    fn test_object_conversion() {
        let pair = Pair {
            key: "test".to_string(),
            value: make_value(MonValueKind::String("value".to_string())),
            validation: None,
        };
        let obj = vec![Member::Pair(pair)];
        let mon_val = make_value(MonValueKind::Object(obj));
        let result = to_value(&mon_val);

        let mut expected_map = BTreeMap::new();
        expected_map.insert("test".to_string(), Value::String("value".to_string()));
        assert_eq!(result, Value::Object(expected_map));
    }

    #[test]
    fn test_object_excludes_non_pair_members() {
        let pair = Pair {
            key: "data".to_string(),
            value: make_value(MonValueKind::Number(123.0)),
            validation: None,
        };
        let obj = vec![Member::Pair(pair), Member::Spread("ignored".to_string())];
        let mon_val = make_value(MonValueKind::Object(obj));
        let result = to_value(&mon_val);

        let mut expected_map = BTreeMap::new();
        expected_map.insert("data".to_string(), Value::Number(123.0));
        assert_eq!(result, Value::Object(expected_map));
    }

    #[test]
    fn test_alias_converts_to_null() {
        let mon_val = make_value(MonValueKind::Alias("some_anchor".to_string()));
        assert_eq!(to_value(&mon_val), Value::Null);
    }

    #[test]
    fn test_enum_value_converts_to_null() {
        let mon_val = make_value(MonValueKind::EnumValue {
            enum_name: "Status".to_string(),
            variant_name: "Active".to_string(),
        });
        assert_eq!(to_value(&mon_val), Value::Null);
    }

    #[test]
    fn test_array_spread_converts_to_null() {
        let mon_val = make_value(MonValueKind::ArraySpread("anchor".to_string()));
        assert_eq!(to_value(&mon_val), Value::Null);
    }

    #[test]
    fn test_nested_object() {
        let inner_pair = Pair {
            key: "inner".to_string(),
            value: make_value(MonValueKind::Number(42.0)),
            validation: None,
        };
        let inner_obj = vec![Member::Pair(inner_pair)];
        let outer_pair = Pair {
            key: "outer".to_string(),
            value: make_value(MonValueKind::Object(inner_obj)),
            validation: None,
        };
        let outer_obj = vec![Member::Pair(outer_pair)];
        let mon_val = make_value(MonValueKind::Object(outer_obj));

        let result = to_value(&mon_val);

        let mut inner_map = BTreeMap::new();
        inner_map.insert("inner".to_string(), Value::Number(42.0));
        let mut outer_map = BTreeMap::new();
        outer_map.insert("outer".to_string(), Value::Object(inner_map));

        assert_eq!(result, Value::Object(outer_map));
    }
}
