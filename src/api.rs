use crate::ast::{Member, MonDocument, MonValue, MonValueKind, SymbolTable, TypeSpec};
use crate::error::MonError;
use crate::lsp;
use crate::parser::Parser;
use crate::resolver::Resolver;
use crate::serialization::{to_value, Value};
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
    pub fn to_value(&self) -> Value {
        to_value(&self.document.root)
    }

    /// Serializes the resolved MON data into a pretty-printed JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self)
    }

    /// Serializes the resolved MON data into a YAML string.
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(&self)
    }

    /// Finds the definition of the symbol at the given character position.
    /// This is the core of "go to definition".
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

    /// Gets information about the type of the symbol at the given character position.
    /// This is the core of "hover" tooltips.
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

#[derive(Debug, Clone, Copy)]
enum FoundNode<'a> {
    Value(&'a MonValue),
    TypeSpec(&'a TypeSpec),
}

/// Finds the most specific AST node that contains the given character position.
fn find_node_at<'a>(value: &'a MonValue, position: usize) -> Option<FoundNode<'a>> {
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

fn find_node_in_type_spec<'a>(
    type_spec: &'a TypeSpec,
    position: usize,
) -> Option<FoundNode<'a>> {
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
            TypeSpec::Simple(name, _) => write!(f, "{}", name),
            TypeSpec::Collection(types, _) => {
                write!(f, "[")?;
                for (i, t) in types.iter().enumerate() {
                    write!(f, "{}", t)?;
                    if i < types.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            }
            TypeSpec::Spread(t, _) => write!(f, "{}...", t),
        }
    }
}

