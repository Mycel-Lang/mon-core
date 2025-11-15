use crate::ast::{Member, MonValue, MonValueKind, SymbolTable, TypeDef, TypeSpec};
use miette::SourceSpan;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum FoundNode<'a> {
    Value(&'a MonValue),
    TypeSpec(&'a TypeSpec),
}

#[derive(Debug, Clone, Copy)]
pub struct SymbolInfo<'a> {
    pub node: FoundNode<'a>,
    pub validation: Option<&'a TypeSpec>,
}

#[cfg(feature = "lsp")]
#[derive(Debug, PartialEq, Clone)]
pub enum SemanticTokenType {
    Struct,
    Enum,
    Alias,
    Anchor,
    Type, // For built-in types like String, Number, etc.
    Keyword,
    Comment,
    String,
    Number,
    Boolean,
    Null,
    Property, // For object keys
}

#[cfg(feature = "lsp")]
#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub struct SemanticToken {
    pub span: SourceSpan,
    pub token_type: SemanticTokenType,
}

pub fn find_symbol_at(value: &'_ MonValue, position: usize) -> Option<SymbolInfo<'_>> {
    if position < value.pos_start || position >= value.pos_end {
        return None;
    }

    if let MonValueKind::Object(members) = &value.kind {
        for member in members {
            if let Member::Pair(pair) = member {
                if let Some(validation) = &pair.validation {
                    if let Some(found) = find_node_in_type_spec(validation, position) {
                        return Some(SymbolInfo {
                            node: found,
                            validation: None, // No validation on a validation
                        });
                    }
                }
                if let Some(mut found) = find_symbol_at(&pair.value, position) {
                    if found.validation.is_none() {
                        found.validation = pair.validation.as_ref();
                    }
                    return Some(found);
                }
            }
        }
    }

    if let MonValueKind::Array(elements) = &value.kind {
        for element in elements {
            if let Some(found) = find_symbol_at(element, position) {
                return Some(found);
            }
        }
    }

    Some(SymbolInfo {
        node: FoundNode::Value(value),
        validation: None,
    })
}

fn find_node_in_type_spec(type_spec: &TypeSpec, position: usize) -> Option<FoundNode<'_>> {
    let span = type_spec.get_span();
    if position < span.offset() || position >= span.offset() + span.len() {
        return None;
    }

    if let TypeSpec::Collection(children, _) = type_spec {
        for child in children {
            if let Some(found) = find_node_in_type_spec(child, position) {
                return Some(found);
            }
        }
    }

    Some(FoundNode::TypeSpec(type_spec))
}

pub fn find_all_usages(root: &MonValue, name: &str) -> Vec<SourceSpan> {
    let mut usages = Vec::new();
    find_all_usages_recursive(root, name, &mut usages);
    usages
}

fn find_all_usages_recursive(value: &MonValue, name: &str, usages: &mut Vec<SourceSpan>) {
    match &value.kind {
        MonValueKind::Alias(alias_name) if alias_name == name => {
            usages.push(value.get_source_span());
        }
        MonValueKind::Object(members) => {
            for member in members {
                if let Member::Pair(pair) = member {
                    if let Some(validation) = &pair.validation {
                        find_all_usages_in_type_spec(validation, name, usages);
                    }
                    find_all_usages_recursive(&pair.value, name, usages);
                }
            }
        }
        MonValueKind::Array(elements) => {
            for element in elements {
                find_all_usages_recursive(element, name, usages);
            }
        }
        _ => {}
    }
}

#[allow(clippy::match_wildcard_for_single_variants)]
fn find_all_usages_in_type_spec(type_spec: &TypeSpec, name: &str, usages: &mut Vec<SourceSpan>) {
    match type_spec {
        TypeSpec::Simple(type_name, span) if type_name == name => {
            usages.push(*span);
        }
        TypeSpec::Collection(children, _) => {
            for child in children {
                find_all_usages_in_type_spec(child, name, usages);
            }
        }
        TypeSpec::Spread(child, _) => {
            find_all_usages_in_type_spec(child, name, usages);
        }
        _ => {}
    }
}

#[cfg(feature = "lsp")]
#[allow(dead_code)]
#[allow(clippy::used_underscore_binding)]
pub fn generate_semantic_tokens(
    root: &MonValue,
    _symbol_table: &SymbolTable,
    anchors: &HashMap<String, MonValue>,
) -> Vec<SemanticToken> {
    let mut tokens = Vec::new();
    generate_semantic_tokens_recursive(root, _symbol_table, anchors, &mut tokens);
    tokens
}

#[track_caller]
// this is needed because pair.get_span tracks caller for better error messages.
#[allow(dead_code)]
#[allow(clippy::used_underscore_binding)]
fn generate_semantic_tokens_recursive(
    value: &MonValue,
    _symbol_table: &SymbolTable,
    anchors: &HashMap<String, MonValue>,
    tokens: &mut Vec<SemanticToken>,
) {
    match &value.kind {
        MonValueKind::Object(members) => {
            for member in members {
                match member {
                    Member::Pair(pair) => {
                        tokens.push(SemanticToken {
                            span: pair.get_span(),
                            token_type: SemanticTokenType::Property,
                        });
                        if let Some(validation) = &pair.validation {
                            generate_semantic_tokens_for_type_spec(validation, tokens);
                        }
                        generate_semantic_tokens_recursive(
                            &pair.value,
                            _symbol_table,
                            anchors,
                            tokens,
                        );
                    }
                    Member::TypeDefinition(type_def) => {
                        tokens.push(SemanticToken {
                            span: type_def.name_span,
                            token_type: match type_def.def_type {
                                TypeDef::Struct(_) => SemanticTokenType::Struct,
                                TypeDef::Enum(_) => SemanticTokenType::Enum,
                            },
                        });
                        // Recurse into the definition itself
                        match &type_def.def_type {
                            TypeDef::Struct(s) => {
                                for field in &s.fields {
                                    tokens.push(SemanticToken {
                                        span: field.type_spec.get_span(),
                                        token_type: SemanticTokenType::Property,
                                    });
                                    generate_semantic_tokens_for_type_spec(
                                        &field.type_spec,
                                        tokens,
                                    );
                                }
                            }
                            TypeDef::Enum(e) => {
                                for _variant in &e.variants {
                                    tokens.push(SemanticToken {
                                        span: SourceSpan::from((e.pos_start, e.pos_end)),
                                        token_type: SemanticTokenType::Property, // Enum variants can be properties
                                    });
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        MonValueKind::Array(elements) => {
            for element in elements {
                generate_semantic_tokens_recursive(element, _symbol_table, anchors, tokens);
                // Renamed
            }
        }
        MonValueKind::Alias(_) => {
            tokens.push(SemanticToken {
                span: value.get_source_span(),
                token_type: SemanticTokenType::Alias,
            });
        }
        MonValueKind::String(_) => {
            tokens.push(SemanticToken {
                span: value.get_source_span(),
                token_type: SemanticTokenType::String,
            });
        }
        MonValueKind::Number(_) => {
            tokens.push(SemanticToken {
                span: value.get_source_span(),
                token_type: SemanticTokenType::Number,
            });
        }
        MonValueKind::Boolean(_) => {
            tokens.push(SemanticToken {
                span: value.get_source_span(),
                token_type: SemanticTokenType::Boolean,
            });
        }
        MonValueKind::Null => {
            tokens.push(SemanticToken {
                span: value.get_source_span(),
                token_type: SemanticTokenType::Null,
            });
        }
        _ => {} // Handle other MonValueKind variants as needed
    }

    // Handle anchors
    if let Some(anchor_name) = &value.anchor {
        // Find the span of the anchor definition (e.g., "&my_anchor")
        if let Some(anchor_def_value) = anchors.get(anchor_name) {
            tokens.push(SemanticToken {
                span: anchor_def_value.get_source_span(), // This span is for the entire value, not just the '&my_anchor' part
                token_type: SemanticTokenType::Anchor,
            });
        }
    }
}

#[allow(dead_code)]
fn generate_semantic_tokens_for_type_spec(type_spec: &TypeSpec, tokens: &mut Vec<SemanticToken>) {
    match type_spec {
        TypeSpec::Simple(_name, span) => {
            // TODO: Differentiate between built-in types and user-defined types
            tokens.push(SemanticToken {
                span: *span,
                token_type: SemanticTokenType::Type,
            });
        }
        TypeSpec::Collection(children, _) => {
            for child in children {
                generate_semantic_tokens_for_type_spec(child, tokens);
            }
        }
        TypeSpec::Spread(child, _) => {
            generate_semantic_tokens_for_type_spec(child, tokens);
        }
    }
}

#[cfg(all(test, feature = "lsp"))]
mod tests {
    use super::*;
    use crate::ast::{EnumDef, Member, MonValue, MonValueKind, Pair, TypeDef, TypeSpec};
    use miette::SourceSpan;
    use std::collections::HashMap;
    use crate::analyze;

    fn make_simple_value(start: usize, end: usize) -> MonValue {
        MonValue {
            kind: MonValueKind::String("test".into()),
            anchor: None,
            pos_start: start,
            pos_end: end,
        }
    }

    #[test]
    fn test_find_symbol_at_value() {
        let val = make_simple_value(0, 5);
        let found = find_symbol_at(&val, 2);
        assert!(matches!(found.unwrap().node, FoundNode::Value(_)));
    }

    #[test]
    fn test_find_symbol_at_out_of_bounds() {
        let val = make_simple_value(0, 5);
        assert!(find_symbol_at(&val, 10).is_none());
    }

    #[test]
    fn test_find_symbol_at_nested_object() {
        let inner_val = make_simple_value(5, 10);
        let pair = Pair {
            key: "inner".into(),
            value: inner_val.clone(),
            validation: None,
        };
        let val = MonValue {
            kind: MonValueKind::Object(vec![Member::Pair(pair.clone())]),
            anchor: None,
            pos_start: 0,
            pos_end: 15,
        };
        let found = find_symbol_at(&val, 6).unwrap();
        if let FoundNode::Value(v) = found.node {
            assert_eq!(v.pos_start, 5);
        } else {
            panic!("Expected value node");
        }
        assert!(found.validation.is_none());
    }

    #[test]
    fn test_find_symbol_at_nested_with_validation() {
        let ts = TypeSpec::Simple("String".into(), SourceSpan::new(0.into(), 5));
        let inner_val = make_simple_value(5, 10);
        let pair = Pair {
            key: "inner".into(),
            value: inner_val.clone(),
            validation: Some(ts.clone()),
        };
        let val = MonValue {
            kind: MonValueKind::Object(vec![Member::Pair(pair)]),
            anchor: None,
            pos_start: 0,
            pos_end: 15,
        };
        let found = find_symbol_at(&val, 2).unwrap();
        if let FoundNode::TypeSpec(ts_found) = found.node {
            match ts_found {
                TypeSpec::Simple(name, _) => assert_eq!(name, "String"),
                _ => panic!("Expected simple TypeSpec"),
            }
        } else {
            panic!("Expected TypeSpec node");
        }
    }

    #[test]
    fn test_find_all_usages_alias_and_typespec() {
        let ts1 = TypeSpec::Simple("MyType".into(), SourceSpan::new(0.into(), 5));
        let val = MonValue {
            kind: MonValueKind::Object(vec![Member::Pair(Pair {
                key: "x".into(),
                value: MonValue {
                    kind: MonValueKind::Alias("MyType".into()),
                    anchor: None,
                    pos_start: 5,
                    pos_end: 10,
                },
                validation: Some(ts1.clone()),
            })]),
            anchor: None,
            pos_start: 0,
            pos_end: 15,
        };
        let usages = find_all_usages(&val, "MyType");
        assert_eq!(usages.len(), 2);
        assert!(usages.contains(&ts1.get_span()));
        assert!(usages.contains(&SourceSpan::new(5.into(), 5)));
    }

    #[test]
    fn test_find_all_usages_empty_object() {
        let val = MonValue {
            kind: MonValueKind::Object(vec![]),
            anchor: None,
            pos_start: 0,
            pos_end: 0,
        };
        let usages = find_all_usages(&val, "foo");
        assert!(usages.is_empty());
    }

    #[test]
    fn test_generate_semantic_tokens_basic() {
        let val = MonValue {
            kind: MonValueKind::Object(vec![Member::Pair(Pair {
                key: "key".into(),
                value: make_simple_value(5, 10),
                validation: Some(TypeSpec::Simple(
                    "String".into(),
                    SourceSpan::new(0.into(), 1),
                )),
            })]),
            anchor: Some("my_anchor".into()),
            pos_start: 0,
            pos_end: 15,
        };
        let mut anchors = HashMap::new();
        anchors.insert("my_anchor".into(), val.clone());
        let symbol_table = SymbolTable::new();

        let tokens = generate_semantic_tokens(&val, &symbol_table, &anchors);
        assert!(tokens
            .iter()
            .any(|t| t.token_type == SemanticTokenType::Anchor));
        assert!(tokens
            .iter()
            .any(|t| t.token_type == SemanticTokenType::Property));
        assert!(tokens
            .iter()
            .any(|t| t.token_type == SemanticTokenType::Type));
        assert!(tokens
            .iter()
            .any(|t| t.token_type == SemanticTokenType::String));
    }

    #[test]
    fn test_generate_semantic_tokens_enum() {
        let enum_def = TypeDef::Enum(EnumDef {
            variants: vec!["A".into(), "B".into()],
            pos_start: 0,
            pos_end: 10,
        });
        let type_def = crate::ast::TypeDefinition {
            name: "MyEnum".into(),
            name_span: SourceSpan::new(0.into(), 6),
            def_type: enum_def,
            pos_start: 0,
            pos_end: 10,
        };
        let val = MonValue {
            kind: MonValueKind::Object(vec![Member::TypeDefinition(type_def)]),
            anchor: None,
            pos_start: 0,
            pos_end: 10,
        };
        let tokens = generate_semantic_tokens(&val, &SymbolTable::new(), &HashMap::new());
        assert!(tokens
            .iter()
            .any(|t| t.token_type == SemanticTokenType::Enum));
        assert!(tokens
            .iter()
            .any(|t| t.token_type == SemanticTokenType::Property));
    }

    #[test]
    fn test_find_node_in_type_spec_nested() {
        let inner = TypeSpec::Simple("Inner".into(), SourceSpan::new(5.into(), 3));
        let ts = TypeSpec::Collection(vec![inner.clone()], SourceSpan::new(0.into(), 10));
        let found = super::find_node_in_type_spec(&ts, 6).unwrap();
        if let FoundNode::TypeSpec(t) = found {
            match t {
                TypeSpec::Simple(name, _) => assert_eq!(name, "Inner"),
                _ => panic!("Expected simple type"),
            }
        } else {
            panic!("Expected TypeSpec node");
        }
    }

    #[test]
    fn test_spread_type_spec() {
        let inner = TypeSpec::Simple("Inner".into(), SourceSpan::new(0.into(), 1));
        let ts = TypeSpec::Spread(Box::new(inner.clone()), SourceSpan::new(0.into(), 2));
        let mut usages = vec![];
        super::find_all_usages_in_type_spec(&ts, "Inner", &mut usages);
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0], inner.get_span());
    }

    #[test]
    fn test_array_handling() {
        let array_val = MonValue {
            kind: MonValueKind::Array(vec![make_simple_value(0, 5), make_simple_value(6, 10)]),
            anchor: None,
            pos_start: 0,
            pos_end: 10,
        };
        let found1 = find_symbol_at(&array_val, 2);
        assert!(found1.is_some());
        let found2 = find_symbol_at(&array_val, 7);
        assert!(found2.is_some());
        assert!(find_symbol_at(&array_val, 20).is_none());
    }

    #[cfg(feature = "lsp")]
    #[test]
    fn test_get_definition_of_alias() {
        let source = r"
        {
            &my_anchor: { a: 1 },
            value: *my_anchor,
        }
    ";

        let analysis_result = analyze(source, "test.mon").unwrap();

        // Find the position of "*my_anchor"
        let alias_pos = source.find("*my_anchor").unwrap();

        let definition_span = analysis_result.get_definition_at(alias_pos).unwrap();

        // Find the position of "&my_anchor"
        let anchor_def_value_pos = source.find("{ a: 1 }").unwrap();
        let anchor_def_value_end_pos = anchor_def_value_pos + "{ a: 1 }".len();

        assert_eq!(definition_span.offset(), anchor_def_value_pos);
        assert_eq!(
            definition_span.len(),
            anchor_def_value_end_pos - anchor_def_value_pos
        );
    }

    #[test]
    fn test_get_definition_of_type() {
        let source = r#"
        {
            MyType: #struct { field(String) },
            value :: MyType = { field: "hello" },
        }
    "#;

        let analysis_result = analyze(source, "test.mon").unwrap();

        // Find the position of "MyType" in the validation
        let type_pos = source.rfind("MyType").unwrap();

        let definition_span = analysis_result.get_definition_at(type_pos).unwrap();

        // Find the position of the struct definition
        let struct_def_pos = source.find("#struct { field(String) }").unwrap();
        let struct_def_end_pos = struct_def_pos + "#struct { field(String) }".len();

        assert_eq!(definition_span.offset(), struct_def_pos);
        assert_eq!(definition_span.len(), struct_def_end_pos - struct_def_pos);
    }
    #[test]
    fn test_get_type_info() {
        let source = r#"
        {
            MyType: #struct { field(String) },
            value :: MyType = { field: "hello" },
        }
    "#;

        let analysis_result = analyze(source, "test.mon").unwrap();

        // Find the position of "hello"
        let value_pos = source.rfind("\"hello\"").unwrap();

        let type_info = analysis_result.get_type_info_at(value_pos).unwrap();

        assert_eq!(type_info, "MyType");
    }

    #[test]
    fn test_find_references() {
        let source = r"
        {
            &my_anchor: { a: 1 },
            value1: *my_anchor,
            value2: *my_anchor,
        }
    ";

        let analysis_result = analyze(source, "test.mon").unwrap();

        // Find the position of the first "*my_anchor"
        let alias_pos = source.find("*my_anchor").unwrap();

        let references = analysis_result.find_references(alias_pos).unwrap();

        assert_eq!(references.len(), 2);

        let first_ref_pos = source.find("*my_anchor").unwrap();
        let second_ref_pos = source.rfind("*my_anchor").unwrap();

        assert_eq!(references[0].offset(), first_ref_pos);
        assert_eq!(references[1].offset(), second_ref_pos);
    }

    #[test]
    fn test_find_type_references() {
        let source = r#"
        {
            MyType: #struct { field(String) },
            value1 :: MyType = { field: "a" },
            value2 :: MyType = { field: "b" },
        }
    "#;

        let analysis_result = analyze(source, "test.mon").unwrap();

        // Find the position of the last "MyType"
        let type_pos = source.rfind("MyType").unwrap();

        let references = analysis_result.find_references(type_pos).unwrap();

        assert_eq!(references.len(), 2);

        let first_usage_pos = source.find("value1 :: MyType").unwrap() + "value1 :: ".len();
        let second_usage_pos = source.find("value2 :: MyType").unwrap() + "value2 :: ".len();

        assert_eq!(references[0].offset(), first_usage_pos);
        assert_eq!(references[1].offset(), second_usage_pos);
    }

}
