use miette::SourceSpan;
use std::fmt::{Debug, Display};

#[derive(Debug, PartialEq, Clone)]
pub struct MonDocument {
    pub root: MonValue,
    pub imports: Vec<ImportStatement>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MonValue {
    pub kind: MonValueKind,
    pub anchor: Option<String>,
    pub pos_start: usize,
    pub pos_end: usize,
}

impl MonValue {
    pub fn get_source_span(&self) -> SourceSpan {
        SourceSpan::new(self.pos_start.into(), self.pos_end - self.pos_start)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum MonValueKind {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    Object(Vec<Member>),
    Array(Vec<MonValue>),
    Alias(String),
    EnumValue {
        enum_name: String,
        variant_name: String,
    },
    ArraySpread(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Member {
    Pair(Pair),
    Spread(String),
    Import(ImportStatement),
    TypeDefinition(TypeDefinition),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Pair {
    pub key: String,
    pub value: MonValue,
    pub validation: Option<TypeSpec>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ImportStatement {
    pub path: String,
    pub spec: ImportSpec,
    pub pos_start: usize,
    pub pos_end: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ImportSpec {
    Namespace(String),
    Named(Vec<ImportSpecifier>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ImportSpecifier {
    pub name: String,
    pub is_anchor: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TypeDefinition {
    pub name: String,
    pub def_type: TypeDef,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TypeDef {
    Struct(StructDef),
    Enum(EnumDef),
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructDef {
    pub fields: Vec<FieldDef>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FieldDef {
    pub name: String,
    pub type_spec: TypeSpec,
    pub default_value: Option<MonValue>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct EnumDef {
    pub variants: Vec<String>,
}

// Represents a type specification, e.g., `String`, `[Number...]`
#[derive(Debug, PartialEq, Clone)]
pub enum TypeSpec {
    Simple(String),
    Collection(Vec<TypeSpec>),
    Spread(Box<TypeSpec>),
}

impl Display for Member {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Member::Pair(p) => write!(f, "Pair({}: {})", p.key, p.value),
            Member::Spread(s) => write!(f, "Spread(...*{})", s),
            Member::Import(i) => write!(f, "Import({:?})", i),
            Member::TypeDefinition(t) => write!(f, "TypeDef({:?})", t),
        }
    }
}

#[derive(Debug, Default)]
pub struct SymbolTable {
    pub types: std::collections::HashMap<String, TypeDef>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Display for MonDocument {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.root)
    }
}

impl Display for MonValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(anchor) = &self.anchor {
            write!(f, "&{} ", anchor)?;
        }
        match &self.kind {
            MonValueKind::String(s) => write!(f, "\"{}\"", s),
            MonValueKind::Number(n) => write!(f, "{}", n),
            MonValueKind::Boolean(b) => write!(f, "{}", b),
            MonValueKind::Null => write!(f, "null"),
            MonValueKind::Object(o) => {
                write!(f, "{{")?;
                for (i, member) in o.iter().enumerate() {
                    write!(f, "{}", member)?;
                    if i < o.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "}}")
            }
            MonValueKind::Array(a) => {
                write!(f, "[")?;
                for (i, value) in a.iter().enumerate() {
                    write!(f, "{}", value)?;
                    if i < a.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            }
            MonValueKind::Alias(a) => write!(f, "*{}", a),
            MonValueKind::EnumValue {
                enum_name,
                variant_name,
            } => {
                write!(f, "${}.{}", enum_name, variant_name)
            }
            MonValueKind::ArraySpread(s) => write!(f, "...*{}", s),
        }
    }
}
