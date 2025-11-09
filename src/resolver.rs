use crate::ast::{
    ImportSpec, ImportStatement, Member, MonDocument, MonValue, MonValueKind,
    SymbolTable as AstSymbolTable, TypeDef, TypeSpec,
};
use crate::error::{ResolverError, ValidationError};
use miette::NamedSource;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct Resolver {
    // Stores resolved documents by their absolute path
    resolved_documents: HashMap<PathBuf, MonDocument>,
    // Stack to detect circular dependencies during import resolution
    resolving_stack: Vec<(PathBuf, Option<ImportStatement>)>, 
    // Global symbol table for types
    pub symbol_table: AstSymbolTable,
    // Global map for anchors
    pub anchors: HashMap<String, MonValue>,
}

impl Resolver {
    pub fn new() -> Self {
        Resolver {
            resolved_documents: HashMap::new(),
            resolving_stack: Vec::new(), // Initialized with the new type
            symbol_table: AstSymbolTable::new(),
            anchors: HashMap::new(),
        }
    }

    pub fn resolve(
        &mut self,
        document: MonDocument,
        source_text: &str,
        file_path: PathBuf,
        causing_import: Option<ImportStatement>,
    ) -> Result<MonDocument, ResolverError> {
        // Add the current file to the resolving stack to detect cycles
        if let Some((_, Some(existing_causing_import))) =
            self.resolving_stack.iter().find(|(p, _)| p == &file_path)
        {
            let cycle_str = self
                .resolving_stack
                .iter()
                .map(|(p, _)| p.to_string_lossy().to_string())
                .collect::<Vec<String>>()
                .join(" -> ");
            return Err(ResolverError::CircularDependency {
                cycle: format!("{} -> {}", cycle_str, file_path.to_string_lossy()),
                src: NamedSource::new(
                    file_path.to_string_lossy().to_string(),
                    source_text.to_string(),
                ),
                span: (
                    existing_causing_import.pos_start,
                    existing_causing_import.pos_end - existing_causing_import.pos_start,
                )
                    .into(),
            });
        }
        self.resolving_stack
            .push((file_path.clone(), causing_import)); // Push with the provided causing_import

        // 1. Process imports
        let current_dir = file_path.parent().unwrap_or_else(|| Path::new("."));
        let source_arc = Arc::new(source_text.to_string());

        for import_statement in &document.imports {
            let imported_path_str = import_statement.path.trim_matches('"');
            let absolute_imported_path = current_dir.join(imported_path_str);

            // Check if already resolved
            if self
                .resolved_documents
                .contains_key(&absolute_imported_path)
            {
                continue;
            }

            // Read the imported file content
            let imported_source_text =
                std::fs::read_to_string(&absolute_imported_path).map_err(|_| {
                    ResolverError::ModuleNotFound {
                        path: imported_path_str.to_string(),
                        src: NamedSource::new(
                            file_path.to_string_lossy().to_string(),
                            source_arc.to_string(),
                        ),
                        span: (
                            import_statement.pos_start,
                            import_statement.pos_end - import_statement.pos_start,
                        )
                            .into(),
                    }
                })?;

            // Parse the imported file
            let mut parser = crate::parser::Parser::new_with_name(
                &imported_source_text,
                absolute_imported_path.to_string_lossy().to_string(),
            )?;
            let imported_document = parser.parse_document()?;

            // Recursively resolve the imported document
            let resolved_imported_document = self.resolve(
                imported_document,
                &imported_source_text,
                absolute_imported_path.clone(),
                Some(import_statement.clone()), // Pass the import statement
            )?;

            self.resolved_documents
                .insert(absolute_imported_path, resolved_imported_document);
        }

        // After resolving all imports, process named imports to populate the symbol table
        for import_statement in &document.imports {
            if let ImportSpec::Named(specifiers) = &import_statement.spec {
                let imported_path_str = import_statement.path.trim_matches('"');
                let absolute_imported_path = current_dir.join(imported_path_str);
                if let Some(imported_doc) = self.resolved_documents.get(&absolute_imported_path) {
                    if let MonValueKind::Object(members) = &imported_doc.root.kind {
                        for specifier in specifiers {
                            if !specifier.is_anchor {
                                for member in members {
                                    if let Member::TypeDefinition(td) = member {
                                        if td.name == specifier.name {
                                            self.symbol_table
                                                .types
                                                .insert(specifier.name.clone(), td.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 2. Collect type definitions and anchors from the current document
        if let MonValueKind::Object(members) = &document.root.kind {
            for member in members {
                match member {
                    Member::TypeDefinition(type_def) => {
                        self.symbol_table
                            .types
                            .insert(type_def.name.clone(), type_def.clone());
                    }
                    Member::Pair(pair) => {
                        if let Some(anchor_name) = &pair.value.anchor {
                            self.anchors.insert(anchor_name.clone(), pair.value.clone());
                        }
                    }
                    _ => {}
                }
            }
        }

        // 3. Resolve aliases and spreads
        let resolved_root = self.resolve_value(document.root, &file_path, source_text)?;

        // 4. Validate the resolved document
        // This will involve iterating through the resolved_root and applying validations
        // where `:: Type` is specified.
        let final_resolved_root =
            self.validate_document_root(resolved_root, &document.imports, &file_path, source_text)?;

        let resolved_doc = MonDocument {
            root: final_resolved_root,
            imports: document.imports, // Imports are already processed
        };

        // Remove the current file from the stack
        self.resolving_stack.pop();

        Ok(resolved_doc)
    }
    // Helper function to recursively resolve aliases and spreads within a MonValue
    fn resolve_value(
        &mut self,
        mut value: MonValue,
        file_path: &PathBuf,
        source_text: &str,
    ) -> Result<MonValue, ResolverError> {
        let alias_span = value.get_source_span();

        match &mut value.kind {
            MonValueKind::Alias(alias_name) => {
                // Resolve alias: find the anchored value and return a deep copy
                let anchor_value = self.anchors.get(alias_name).ok_or_else(|| {
                    // TODO: Get actual span for the alias
                    ResolverError::AnchorNotFound {
                        name: alias_name.clone(),
                        src: NamedSource::new(
                            file_path.to_string_lossy().to_string(),
                            source_text.to_string(),
                        ),
                        span: alias_span,
                    }
                })?;
                Ok(anchor_value.clone()) // Return a deep copy
            }
            MonValueKind::Object(members) => {
                let mut resolved_members = Vec::new();
                for member in members.drain(..) {
                    match member {
                        Member::Spread(spread_name) => {
                            // Resolve object spread: merge members from anchored object
                            let anchor_value = self.anchors.get(&spread_name).ok_or_else(|| {
                                // TODO: Get actual span for the spread
                                ResolverError::AnchorNotFound {
                                    name: spread_name.clone(),
                                    src: NamedSource::new(
                                        file_path.to_string_lossy().to_string(),
                                        source_text.to_string(),
                                    ),
                                    span: alias_span,
                                }
                            })?;
                            if let MonValueKind::Object(spread_members) = &anchor_value.kind {
                                let spread_members_clone = spread_members.clone();
                                for spread_member in spread_members_clone {
                                    // Recursively resolve spread members
                                    resolved_members.push(self.resolve_value_member(
                                        spread_member,
                                        file_path,
                                        source_text,
                                    )?);
                                }
                            } else {
                                return Err(ResolverError::SpreadOnNonObject {
                                    name: spread_name.clone(),
                                    src: NamedSource::new(
                                        file_path.to_string_lossy().to_string(),
                                        source_text.to_string(),
                                    ),
                                    span: alias_span,
                                });
                            }
                        }
                        _ => {
                            // Recursively resolve other members
                            resolved_members.push(self.resolve_value_member(
                                member,
                                file_path,
                                source_text,
                            )?);
                        }
                    }
                }
                // Handle key overriding for object spreads (local keys win)
                let mut final_members_map: HashMap<String, Member> = HashMap::new();
                for member in resolved_members {
                    if let Member::Pair(pair) = member {
                        final_members_map.insert(pair.key.clone(), Member::Pair(pair));
                    } else {
                        // Non-pair members (like TypeDefinition) are just added
                        // This might need refinement depending on how TypeDefinitions are handled after resolution
                        final_members_map.insert(format!("{:?}", member), member); // Dummy key for now
                    }
                }
                let final_members = final_members_map.into_values().collect();
                Ok(MonValue {
                    kind: MonValueKind::Object(final_members),
                    anchor: value.anchor,
                    pos_start: value.pos_start,
                    pos_end: value.pos_end,
                })
            }
            MonValueKind::Array(elements) => {
                let mut resolved_elements = Vec::new();
                for element in elements.drain(..) {
                    match element.kind {
                        MonValueKind::ArraySpread(spread_name) => {
                            // Resolve array spread: concatenate elements from anchored array
                            let anchor_value = self.anchors.get(&spread_name).ok_or_else(|| {
                                // TODO: Get actual span for the spread
                                ResolverError::AnchorNotFound {
                                    name: spread_name.clone(),
                                    src: NamedSource::new(
                                        file_path.to_string_lossy().to_string(),
                                        source_text.to_string(),
                                    ),
                                    span: alias_span.into(),
                                }
                            })?;
                            if let MonValueKind::Array(spread_elements) = &anchor_value.kind {
                                let spread_elements_clone = spread_elements.clone();
                                for spread_element in spread_elements_clone {
                                    // Recursively resolve spread elements
                                    resolved_elements.push(self.resolve_value(
                                        spread_element,
                                        file_path,
                                        source_text,
                                    )?);
                                }
                            } else {
                                // TODO: Get actual span for the spread
                                return Err(ResolverError::SpreadOnNonArray {
                                    name: spread_name.clone(),
                                    src: NamedSource::new(
                                        file_path.to_string_lossy().to_string(),
                                        source_text.to_string(),
                                    ),
                                    span: alias_span,
                                });
                            }
                        }
                        _ => {
                            // Recursively resolve other elements
                            resolved_elements.push(self.resolve_value(
                                element,
                                file_path,
                                source_text,
                            )?);
                        }
                    }
                }
                Ok(MonValue {
                    kind: MonValueKind::Array(resolved_elements),
                    anchor: value.anchor,
                    pos_start: value.pos_start,
                    pos_end: value.pos_end,
                })
            }
            _ => Ok(value), // Other literal values don't need further resolution
        }
    }

    // Helper to resolve a Member (Pair, TypeDefinition, etc.)
    fn resolve_value_member(
        &mut self,
        mut member: Member,
        file_path: &PathBuf,
        source_text: &str,
    ) -> Result<Member, ResolverError> {
        match &mut member {
            Member::Pair(pair) => {
                pair.value = self.resolve_value(pair.value.clone(), file_path, source_text)?;
                Ok(member)
            }
            // Type definitions and imports are already processed or don't need further resolution here
            _ => Ok(member),
        }
    }
    // Helper function to validate the root of the document after resolution
    fn validate_document_root(
        &mut self,
        mut root_value: MonValue,
        imports: &[ImportStatement], // Change this parameter
        file_path: &PathBuf,
        source_text: &str,
    ) -> Result<MonValue, ResolverError> {
        if let MonValueKind::Object(members) = &mut root_value.kind {
            for member in members.iter_mut() {
                if let Member::Pair(pair) = member {
                    if let Some(type_spec) = &pair.validation {
                        // Perform validation for this pair
                        self.validate_value(
                            &mut pair.value,
                            type_spec,
                            &pair.key,
                            imports, // Pass the imports here
                            file_path,
                            source_text,
                        )?;
                    }
                }
            }
        }
        Ok(root_value)
    }

    // Helper function to recursively validate a MonValue against a TypeSpec
    fn validate_value(
        &mut self,
        value: &mut MonValue,
        type_spec: &TypeSpec,
        field_name: &str,            // For error reporting
        imports: &[ImportStatement], // Change this parameter
        file_path: &PathBuf,
        source_text: &str,
    ) -> Result<(), ResolverError> {
        match type_spec {
            TypeSpec::Simple(type_name, _) => {
                // Handle built-in types and user-defined types (structs/enums)
                match type_name.as_str() {
                    "String" => {
                        if !matches!(value.kind, MonValueKind::String(_)) {
                            return Err(ResolverError::Validation(ValidationError::TypeMismatch {
                                field_name: field_name.to_string(),
                                expected_type: "String".to_string(),
                                found_type: format!("{:?}", value.kind),
                                src: NamedSource::new(
                                    file_path.to_string_lossy().to_string(),
                                    source_text.to_string(),
                                ),
                                span: (value.pos_start, value.pos_end - value.pos_start).into(),
                            }));
                        }
                    }
                    "Number" => {
                        if !matches!(value.kind, MonValueKind::Number(_)) {
                            return Err(ResolverError::Validation(ValidationError::TypeMismatch {
                                field_name: field_name.to_string(),
                                expected_type: "Number".to_string(),
                                found_type: format!("{:?}", value.kind),
                                src: NamedSource::new(
                                    file_path.to_string_lossy().to_string(),
                                    source_text.to_string(),
                                ),
                                span: (value.pos_start, value.pos_end - value.pos_start).into(),
                            }));
                        }
                    }
                    "Boolean" => {
                        if !matches!(value.kind, MonValueKind::Boolean(_)) {
                            return Err(ResolverError::Validation(ValidationError::TypeMismatch {
                                field_name: field_name.to_string(),
                                expected_type: "Boolean".to_string(),
                                found_type: format!("{:?}", value.kind),
                                src: NamedSource::new(
                                    file_path.to_string_lossy().to_string(),
                                    source_text.to_string(),
                                ),
                                span: (value.pos_start, value.pos_end - value.pos_start).into(),
                            }));
                        }
                    }
                    "Null" => {
                        if !matches!(value.kind, MonValueKind::Null) {
                            return Err(ResolverError::Validation(ValidationError::TypeMismatch {
                                field_name: field_name.to_string(),
                                expected_type: "Null".to_string(),
                                found_type: format!("{:?}", value.kind),
                                src: NamedSource::new(
                                    file_path.to_string_lossy().to_string(),
                                    source_text.to_string(),
                                ),
                                span: (value.pos_start, value.pos_end - value.pos_start).into(),
                            }));
                        }
                    }
                    "Object" => {
                        if !matches!(value.kind, MonValueKind::Object(_)) {
                            return Err(ResolverError::Validation(ValidationError::TypeMismatch {
                                field_name: field_name.to_string(),
                                expected_type: "Object".to_string(),
                                found_type: format!("{:?}", value.kind),
                                src: NamedSource::new(
                                    file_path.to_string_lossy().to_string(),
                                    source_text.to_string(),
                                ),
                                span: (value.pos_start, value.pos_end - value.pos_start).into(),
                            }));
                        }
                    }
                    "Array" => {
                        if !matches!(value.kind, MonValueKind::Array(_)) {
                            return Err(ResolverError::Validation(ValidationError::TypeMismatch {
                                field_name: field_name.to_string(),
                                expected_type: "Array".to_string(),
                                found_type: format!("{:?}", value.kind),
                                src: NamedSource::new(
                                    file_path.to_string_lossy().to_string(),
                                    source_text.to_string(),
                                ),
                                span: (value.pos_start, value.pos_end - value.pos_start).into(),
                            }));
                        }
                    }
                    "Any" => { /* Always valid, like you :D */ }
                    _ => {
                        // User-defined type (Struct or Enum)
                        let (namespace, type_name_part) =
                            if let Some((ns, tn)) = type_name.split_once('.') {
                                (Some(ns), tn)
                            } else {
                                (None, type_name.as_str())
                            };

                        let type_def = if let Some(namespace) = namespace {
                            // Find the import statement for this namespace
                            let import_statement = imports
                                .iter()
                                .find(|i| {
                                    if let ImportSpec::Namespace(ns) = &i.spec {
                                        ns == namespace
                                    } else {
                                        false
                                    }
                                })
                                .ok_or_else(|| {
                                    ResolverError::Validation(ValidationError::UndefinedType {
                                        type_name: type_name.to_string(),
                                        src: NamedSource::new(
                                            file_path.to_string_lossy().to_string(),
                                            source_text.to_string(),
                                        ),
                                        span: (value.pos_start, value.pos_end - value.pos_start)
                                            .into(),
                                    })
                                })?;

                            let imported_path_str = import_statement.path.trim_matches('"');
                            let parent_dir = file_path.parent().ok_or_else(|| {
                                // This case is unlikely but good to handle.
                                // It means the file path is something like "/" or "C:\"
                                ResolverError::ModuleNotFound {
                                    path: import_statement.path.clone(),
                                    src: NamedSource::new(
                                        file_path.to_string_lossy().to_string(),
                                        source_text.to_string(),
                                    ),
                                    span: (
                                        import_statement.pos_start,
                                        import_statement.pos_end - import_statement.pos_start,
                                    )
                                        .into(),
                                }
                            })?;
                            let absolute_imported_path = parent_dir.join(imported_path_str);

                            let imported_doc = self
                                .resolved_documents
                                .get(&absolute_imported_path)
                                .ok_or_else(|| {
                                    // This indicates a logic error in the resolver, as the document
                                    // should have been resolved and stored during the initial import pass.
                                    ResolverError::ModuleNotFound {
                                        path: absolute_imported_path.to_string_lossy().to_string(),
                                        src: NamedSource::new(
                                            file_path.to_string_lossy().to_string(),
                                            source_text.to_string(),
                                        ),
                                        span: (value.pos_start, value.pos_end - value.pos_start)
                                            .into(),
                                    }
                                })?;

                            if let MonValueKind::Object(members) = &imported_doc.root.kind {
                                members.iter().find_map(|m| {
                                    if let Member::TypeDefinition(td) = m {
                                        if td.name == type_name_part {
                                            return Some(td.def_type.clone());
                                        }
                                    }
                                    None
                                })
                            } else {
                                None
                            }
                        } else {
                            self.symbol_table.types.get(type_name_part).map(|td| td.def_type.clone())
                        };

                        if let Some(type_def) = type_def {
                            match type_def {
                                TypeDef::Struct(struct_def) => {
                                    // Validate against struct
                                    if let MonValueKind::Object(value_members) = &mut value.kind {
                                        let mut value_map: HashMap<String, &mut MonValue> =
                                            HashMap::new();
                                        for member in value_members.iter_mut() {
                                            if let Member::Pair(pair) = member {
                                                value_map.insert(pair.key.clone(), &mut pair.value);
                                            }
                                        }

                                        let mut new_members = Vec::new();
                                        for field_def in &struct_def.fields {
                                            if let Some(field_value) =
                                                value_map.get_mut(&field_def.name)
                                            {
                                                // Field exists, validate its type
                                                self.validate_value(
                                                    field_value,
                                                    &field_def.type_spec,
                                                    &field_def.name,
                                                    imports, // Pass the imports here
                                                    file_path,
                                                    source_text,
                                                )?;
                                            } else {
                                                // Field missing
                                                if field_def.default_value.is_none() {
                                                    return Err(ResolverError::Validation(
                                                        ValidationError::MissingField {
                                                            field_name: field_def.name.clone(),
                                                            struct_name: type_name.to_string(),
                                                            src: NamedSource::new(
                                                                file_path
                                                                    .to_string_lossy()
                                                                    .to_string(),
                                                                source_text.to_string(),
                                                            ),
                                                            span: (
                                                                value.pos_start,
                                                                value.pos_end - value.pos_start,
                                                            )
                                                                .into(),
                                                        },
                                                    ));
                                                } else {
                                                    // Field is missing, but has a default value.
                                                    // We need to insert it into the object.
                                                    if let Some(default_value) =
                                                        &field_def.default_value
                                                    {
                                                        new_members.push(Member::Pair(
                                                            crate::ast::Pair {
                                                                key: field_def.name.clone(),
                                                                value: default_value.clone(),
                                                                validation: None,
                                                            },
                                                        ));
                                                    }
                                                }
                                            }
                                        }
                                        value_members.extend(new_members);

                                        // Check for extra fields
                                        for member in value_members.iter() {
                                            if let Member::Pair(pair) = member {
                                                if !struct_def
                                                    .fields
                                                    .iter()
                                                    .any(|f| f.name == pair.key)
                                                {
                                                    return Err(ResolverError::Validation(
                                                        ValidationError::UnexpectedField {
                                                            field_name: pair.key.clone(),
                                                            struct_name: type_name.to_string(),
                                                            src: NamedSource::new(
                                                                file_path
                                                                    .to_string_lossy()
                                                                    .to_string(),
                                                                source_text.to_string(),
                                                            ),
                                                            span: (
                                                                value.pos_start,
                                                                value.pos_end - value.pos_start,
                                                            )
                                                                .into(),
                                                        },
                                                    ));
                                                }
                                            }
                                        }
                                    } else {
                                        return Err(ResolverError::Validation(
                                            ValidationError::TypeMismatch {
                                                field_name: field_name.to_string(),
                                                expected_type: type_name.to_string(),
                                                found_type: format!("{:?}", value.kind),
                                                src: NamedSource::new(
                                                    file_path.to_string_lossy().to_string(),
                                                    source_text.to_string(),
                                                ),
                                                span: (
                                                    value.pos_start,
                                                    value.pos_end - value.pos_start,
                                                )
                                                    .into(),
                                            },
                                        ));
                                    }
                                }
                                TypeDef::Enum(enum_def) => {
                                    // Validate against enum
                                    if let MonValueKind::EnumValue {
                                        enum_name,
                                        variant_name,
                                    } = &value.kind
                                    {
                                        if enum_name != type_name {
                                            return Err(ResolverError::Validation(
                                                ValidationError::TypeMismatch {
                                                    field_name: field_name.to_string(),
                                                    expected_type: format!("enum '{}'", type_name),
                                                    found_type: format!("enum '{}'", enum_name),
                                                    src: NamedSource::new(
                                                        file_path.to_string_lossy().to_string(),
                                                        source_text.to_string(),
                                                    ),
                                                    span: (
                                                        value.pos_start,
                                                        value.pos_end - value.pos_start,
                                                    )
                                                        .into(),
                                                },
                                            ));
                                        }
                                        if !enum_def.variants.contains(variant_name) {
                                            return Err(ResolverError::Validation(
                                                ValidationError::UndefinedEnumVariant {
                                                    variant_name: variant_name.clone(),
                                                    enum_name: type_name.to_string(),
                                                    src: NamedSource::new(
                                                        file_path.to_string_lossy().to_string(),
                                                        source_text.to_string(),
                                                    ),
                                                    span: (
                                                        value.pos_start,
                                                        value.pos_end - value.pos_start,
                                                    )
                                                        .into(),
                                                },
                                            ));
                                        }
                                    } else {
                                        return Err(ResolverError::Validation(
                                            ValidationError::TypeMismatch {
                                                field_name: field_name.to_string(),
                                                expected_type: format!("enum '{}'", type_name),
                                                found_type: format!("{:?}", value.kind),
                                                src: NamedSource::new(
                                                    file_path.to_string_lossy().to_string(),
                                                    source_text.to_string(),
                                                ),
                                                span: (
                                                    value.pos_start,
                                                    value.pos_end - value.pos_start,
                                                )
                                                    .into(),
                                            },
                                        ));
                                    }
                                }
                            }
                        } else {
                            return Err(ResolverError::Validation(
                                ValidationError::UndefinedType {
                                    type_name: type_name.to_string(),
                                    src: NamedSource::new(
                                        file_path.to_string_lossy().to_string(),
                                        source_text.to_string(),
                                    ),
                                    span: (value.pos_start, value.pos_end - value.pos_start).into(),
                                },
                            ));
                        }
                    }
                }
            }
            TypeSpec::Collection(collection_types, _) => {
                // Handle array validation
                if let MonValueKind::Array(elements) = &mut value.kind {
                    self.validate_collection(
                        elements,
                        collection_types,
                        field_name,
                        imports, // Pass the imports here
                        file_path,
                        source_text,
                    )?;
                } else {
                    return Err(ResolverError::Validation(ValidationError::TypeMismatch {
                        field_name: field_name.to_string(),
                        expected_type: "Array".to_string(),
                        found_type: format!("{:?}", value.kind),
                        src: NamedSource::new(
                            file_path.to_string_lossy().to_string(),
                            source_text.to_string(),
                        ),
                        span: (value.pos_start, value.pos_end - value.pos_start).into(),
                    }));
                }
            }
            TypeSpec::Spread(_, _) => {
                // Spread types are handled during parsing/resolution, not validation directly
                return Ok(());
            }
        }
        return Ok(());
    }

    fn validate_collection(
        &mut self,
        elements: &mut [MonValue],
        collection_types: &[TypeSpec],
        field_name: &str,
        imports: &[ImportStatement], // Change this parameter
        file_path: &PathBuf,
        source_text: &str,
    ) -> Result<(), ResolverError> {
        // Case 1: [T] - Exactly one element of type T
        if collection_types.len() == 1 && !matches!(collection_types[0], TypeSpec::Spread(_, _)) {
            self.validate_value(
                &mut elements[0],
                &collection_types[0],
                field_name,
                imports, // Pass the imports here
                file_path,
                source_text,
            )?;
            return Ok(());
        }

        // Case 2: [T...] - Zero or more elements of type T
        if collection_types.len() == 1 && matches!(collection_types[0], TypeSpec::Spread(_, _)) {
            if let TypeSpec::Spread(inner_type, _) = &collection_types[0] {
                for element in elements {
                    self.validate_value(
                        element,
                        inner_type,
                        field_name,
                        imports,
                        file_path,
                        source_text,
                    )?;
                }
                return Ok(());
            }
        }

        // Case 3: Tuple-like [T1, T2, ...]
        let has_spread = collection_types
            .iter()
            .any(|t| matches!(t, TypeSpec::Spread(_, _)));
        if !has_spread {
            if elements.len() != collection_types.len() {
                // TODO: Better error for wrong number of elements
                return Err(ResolverError::Validation(ValidationError::TypeMismatch {
                    field_name: field_name.to_string(),
                    expected_type: format!("tuple with {} elements", collection_types.len()),
                    found_type: format!("tuple with {} elements", elements.len()),
                    src: NamedSource::new(
                        file_path.to_string_lossy().to_string(),
                        source_text.to_string(),
                    ),
                    span: (
                        elements.first().map_or(0, |e| e.pos_start),
                        elements.last().map_or(0, |e| e.pos_end)
                            - elements.first().map_or(0, |e| e.pos_start),
                    )
                        .into(),
                }));
            }
            for (i, element) in elements.iter_mut().enumerate() {
                self.validate_value(
                    element,
                    &collection_types[i],
                    field_name,
                    imports, // Pass the imports here
                    file_path,
                    source_text,
                )?;
            }
            return Ok(());
        }

        // Case 4: [T1, T2...] - One or more elements, first is T1, rest are T2
        if collection_types.len() == 2
            && !matches!(collection_types[0], TypeSpec::Spread(_, _))
            && matches!(collection_types[1], TypeSpec::Spread(_, _))
        {
            if elements.is_empty() {
                return Err(ResolverError::Validation(ValidationError::TypeMismatch {
                    field_name: field_name.to_string(),
                    expected_type: format!("array with at least 1 element"),
                    found_type: format!("empty array"),
                    src: NamedSource::new(
                        file_path.to_string_lossy().to_string(),
                        source_text.to_string(),
                    ),
                    span: (
                        elements.first().map_or(0, |e| e.pos_start),
                        elements.last().map_or(0, |e| e.pos_end)
                            - elements.first().map_or(0, |e| e.pos_start),
                    )
                        .into(),
                }));
            }
            self.validate_value(
                &mut elements[0],
                &collection_types[0],
                field_name,
                imports, // Pass the imports here
                file_path,
                source_text,
            )?;
            if let TypeSpec::Spread(inner_type, _) = &collection_types[1] {
                for element in &mut elements[1..] {
                    self.validate_value(
                        element,
                        inner_type,
                        field_name,
                        imports,
                        file_path,
                        source_text,
                    )?;
                }
            }
            return Ok(());
        }

        // Case 5: [T1..., T2] - One or more elements, last is T2, rest are T1
        if collection_types.len() == 2
            && matches!(collection_types[0], TypeSpec::Spread(_, _))
            && !matches!(collection_types[1], TypeSpec::Spread(_, _))
        {
            if elements.is_empty() {
                return Err(ResolverError::Validation(ValidationError::TypeMismatch {
                    field_name: field_name.to_string(),
                    expected_type: format!("array with at least 1 element"),
                    found_type: format!("empty array"),
                    src: NamedSource::new(
                        file_path.to_string_lossy().to_string(),
                        source_text.to_string(),
                    ),
                    span: (
                        elements.first().map_or(0, |e| e.pos_start),
                        elements.last().map_or(0, |e| e.pos_end)
                            - elements.first().map_or(0, |e| e.pos_start),
                    )
                        .into(),
                }));
            }
            let (head, last) = elements.split_at_mut(elements.len() - 1);
            self.validate_value(
                last.first_mut().unwrap(), // Get the last element
                &collection_types[1],
                field_name,
                imports, // Pass the imports here
                file_path,
                source_text,
            )?;
            if let TypeSpec::Spread(inner_type, _) = &collection_types[0] {
                for element in head {
                    self.validate_value(
                        element,
                        inner_type,
                        field_name,
                        imports,
                        file_path,
                        source_text,
                    )?;
                }
            }
            return Ok(());
        }

        // If none of the specific cases match, it's an unimplemented complex collection type
        return Err(ResolverError::Validation(
            ValidationError::UnimplementedCollectionValidation {
                field_name: field_name.to_string(),
                src: NamedSource::new(
                    file_path.to_string_lossy().to_string(),
                    source_text.to_string(),
                ),
                span: (
                    elements.first().map_or(0, |e| e.pos_start),
                    elements.last().map_or(0, |e| e.pos_end)
                        - elements.first().map_or(0, |e| e.pos_start),
                )
                    .into(),
            },
        ));
    }
}
