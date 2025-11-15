#[allow(dead_code)]
use crate::ast::{MonDocument, MonValue, SymbolTable, TypeSpec};
use crate::error::MonError;

#[cfg(feature = "lsp")]
use crate::ast::{Member, MonValueKind};
#[cfg(feature = "lsp")]
use crate::lsp;
use crate::parser::Parser;
use crate::resolver::Resolver;
use crate::serialization::{to_value, Value};
#[cfg(feature = "lsp")]
use miette::SourceSpan;
use serde::{Serialize, Serializer};
use serde_json;
use serde_yaml;
use std::collections::HashMap;
use std::fmt::Display;
use std::path::PathBuf;

/// The result of a successful analysis of a MON document.
/// This struct contains the fully resolved document and provides
/// methods for serialization and further inspection, making it
/// suitable for both direct consumption and for powering an LSP.
pub struct AnalysisResult {
    pub document: MonDocument,
    pub unresolved_document: MonDocument,
    pub symbol_table: SymbolTable,
    pub anchors: HashMap<String, MonValue>,
}

impl Serialize for AnalysisResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = self.to_value();
        value.serialize(serializer)
    }
}

impl AnalysisResult {
    /// Serializes the resolved MON data into a generic, serializable `Value`.
    #[must_use]
    pub fn to_value(&self) -> Value {
        to_value(&self.document.root)
    }

    /// Serializes the resolved MON data into a pretty-printed JSON string.
    ///
    /// # Errors
    /// Returns a `serde_json::Error` if serialization fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self)
    }

    /// Serializes the resolved MON data into a YAML string.
    ///
    /// # Errors
    /// Returns a `serde_yaml::Error` if serialization fails.
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(&self)
    }

    #[cfg(feature = "lsp")]
    /// Finds the definition of the symbol at the given character position.
    /// This is the core of "go to definition".
    #[must_use]
    pub fn get_definition_at(&self, position: usize) -> Option<SourceSpan> {
        let node = find_node_at(&self.unresolved_document.root, position)?;

        match node {
            FoundNode::Value(value) => match &value.kind {
                MonValueKind::Alias(alias_name) => {
                    let anchor_def = self.anchors.get(alias_name)?;
                    Some(anchor_def.get_source_span())
                }
                _ => None,
            },
            FoundNode::TypeSpec(type_spec) => match type_spec {
                TypeSpec::Simple(name, _) => {
                    let type_def = self.symbol_table.types.get(name)?;
                    Some(type_def.def_type.get_span())
                }
                _ => None,
            },
        }
    }

    #[cfg(feature = "lsp")]
    /// Gets information about the type of the symbol at the given character position.
    /// This is the core of "hover" tooltips.
    #[must_use]
    pub fn get_type_info_at(&self, position: usize) -> Option<String> {
        let symbol_info = lsp::find_symbol_at(&self.unresolved_document.root, position)?;

        if let Some(validation) = symbol_info.validation {
            return Some(validation.to_string());
        }

        match symbol_info.node {
            lsp::FoundNode::Value(value) => Some(value.kind.to_string()),
            lsp::FoundNode::TypeSpec(type_spec) => Some(type_spec.to_string()),
        }
    }
    #[cfg(feature = "lsp")]
    /// Finds all references to the symbol at the given character position.
    pub fn find_references(&self, position: usize) -> Option<Vec<SourceSpan>> {
        let symbol_info = lsp::find_symbol_at(&self.unresolved_document.root, position)?;

        let (name_to_find, definition_span) = match symbol_info.node {
            lsp::FoundNode::Value(value) => match &value.kind {
                MonValueKind::Alias(alias_name) => {
                    let anchor_def = self.anchors.get(alias_name)?;
                    (alias_name.clone(), anchor_def.get_source_span())
                }
                _ => return None,
            },
            lsp::FoundNode::TypeSpec(type_spec) => match type_spec {
                TypeSpec::Simple(name, _) => {
                    let type_def = self.symbol_table.types.get(name)?;
                    (name.clone(), type_def.name_span)
                }
                _ => return None,
            },
        };

        let usages = lsp::find_all_usages(&self.unresolved_document.root, &name_to_find)
            .into_iter()
            .filter(|span| *span != definition_span)
            .collect();
        Some(usages)
    }
}

#[cfg(feature = "lsp")]
#[derive(Debug, Clone, Copy)]
enum FoundNode<'a> {
    Value(&'a MonValue),
    TypeSpec(&'a TypeSpec),
}

#[cfg(feature = "lsp")]
/// Finds the most specific AST node that contains the given character position.
fn find_node_at(value: &MonValue, position: usize) -> Option<FoundNode<'_>> {
    if position < value.pos_start || position >= value.pos_end {
        return None;
    }

    if let MonValueKind::Object(members) = &value.kind {
        for member in members {
            if let Member::Pair(pair) = member {
                if let Some(validation) = &pair.validation {
                    if let Some(found) = find_node_in_type_spec(validation, position) {
                        return Some(found);
                    }
                }
                if let Some(found) = find_node_at(&pair.value, position) {
                    return Some(found);
                }
            }
        }
    }

    if let MonValueKind::Array(elements) = &value.kind {
        for element in elements {
            if let Some(found) = find_node_at(element, position) {
                return Some(found);
            }
        }
    }

    Some(FoundNode::Value(value))
}

#[cfg(feature = "lsp")]
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

/// Analyzes a MON source string, parsing, resolving, and validating it.
///
/// This is the primary entry point for processing MON data. It returns an
/// `AnalysisResult` on success, which contains the fully resolved document
/// and provides methods for serialization and LSP-related queries.
///
/// # Arguments
///
/// * `source` - The MON source code as a string.
/// * `file_name` - The name of the file being analyzed (used for error reporting).
///
/// # Errors
///
/// Returns a `MonError` if parsing, resolution, or validation fails.
///
/// # Panics
///
/// Panics if the current directory cannot be determined when `file_name` is relative.
pub fn analyze(source: &str, file_name: &str) -> Result<AnalysisResult, MonError> {
    let mut parser = Parser::new_with_name(source, file_name.to_string())?;
    let document = parser.parse_document()?;
    let unresolved_document = document.clone();

    let mut resolver = Resolver::new();
    let mut path = PathBuf::from(file_name);
    if path.is_relative() {
        path = std::env::current_dir().unwrap().join(path);
    }

    let resolved_doc = resolver.resolve(document, source, path, None)?;

    Ok(AnalysisResult {
        document: resolved_doc,
        unresolved_document,
        symbol_table: resolver.symbol_table,
        anchors: resolver.anchors,
    })
}

impl Display for TypeSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeSpec::Simple(name, _) => write!(f, "{name}"),
            TypeSpec::Collection(types, _) => {
                write!(f, "[")?;
                for (i, t) in types.iter().enumerate() {
                    write!(f, "{t}")?;
                    if i < types.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            }
            TypeSpec::Spread(t, _) => write!(f, "{t}..."),
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::analyze;

    #[test]
    fn test_simple_parse_to_json() {
        let source = r#"
        {
            name: "My App",
            version: 1.0,
            is_enabled: true,
            features: ["a", "b", "c"],
            config: {
                host: "localhost",
                port: 8080.0,
            }
        }
    "#;

        let expected_json = serde_json::json!({
            "name": "My App",
            "version": 1.0,
            "is_enabled": true,
            "features": ["a", "b", "c"],
            "config": {
                "host": "localhost",
                "port": 8080.0,
            }
        });

        let analysis_result = analyze(source, "test.mon").unwrap();
        let result = analysis_result.to_json().unwrap();
        let result_json: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(result_json, expected_json);
    }

    #[test]
    fn test_analyze_semantic_info() {
        let source = r"
        {
            MyType: #struct { field(String) },
            &my_anchor: { a: 1 },
            value: *my_anchor,
        }
    ";

        let analysis_result = analyze(source, "test.mon").unwrap();

        // Check symbol table
        assert!(analysis_result.symbol_table.types.contains_key("MyType"));

        // Check anchors
        assert!(analysis_result.anchors.contains_key("my_anchor"));
    }

    #[test]
    fn test_simple_parse_to_yaml() {
        let source = r#"
        {
            name: "My App",
            version: 1.0,
            is_enabled: true,
        }
    "#;

        let expected_yaml = "is_enabled: true\nname: My App\nversion: 1.0\n";

        let analysis_result = analyze(source, "test.mon").unwrap();
        let result = analysis_result.to_yaml().unwrap();

        assert_eq!(result, expected_yaml);
    }

}