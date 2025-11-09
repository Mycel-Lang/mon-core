use crate::ast::{MonDocument, MonValue, SymbolTable};
use crate::error::MonError;
use crate::parser::Parser;
use crate::resolver::Resolver;
use crate::serialization::{to_value, Value};
use serde_json;
use serde_yaml;
use std::collections::HashMap;
use std::path::PathBuf;

/// The result of a successful analysis of a MON document.
/// This struct contains the fully resolved document and provides
/// methods for serialization and further inspection, making it
/// suitable for both direct consumption and for powering an LSP.
pub struct AnalysisResult {
    pub document: MonDocument,
    pub symbol_table: SymbolTable,
    pub anchors: HashMap<String, MonValue>,
}

impl AnalysisResult {
    /// Serializes the resolved MON data into a generic, serializable `Value`.
    pub fn to_value(&self) -> Value {
        to_value(&self.document.root)
    }

    /// Serializes the resolved MON data into a pretty-printed JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        let value = self.to_value();
        serde_json::to_string_pretty(&value)
    }

    /// Serializes the resolved MON data into a YAML string.
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        let value = self.to_value();
        serde_yaml::to_string(&value)
    }
}

pub fn analyze(source: &str, file_name: &str) -> Result<AnalysisResult, MonError> {
    let mut parser = Parser::new_with_name(source, file_name.to_string())?;
    let document = parser.parse_document()?;

    let mut resolver = Resolver::new();
    let mut path = PathBuf::from(file_name);
    if path.is_relative() {
        path = std::env::current_dir().unwrap().join(path);
    }

    let resolved_doc = resolver.resolve(document, source, path, None)?;

    Ok(AnalysisResult {
        document: resolved_doc,
        symbol_table: resolver.symbol_table,
        anchors: resolver.anchors,
    })
}

