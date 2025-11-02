#[derive(Debug, PartialEq, Clone)]
pub struct MonDocument {
    pub root: MonValue,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MonValue {
    pub kind: MonValueKind,
    pub anchor: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum MonValueKind {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    Object(Vec<Member>),
    Array(Vec<MonValue>),
    Import(String),
    Alias(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Member {
    Pair(Pair),
    Spread(String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Pair {
    pub key: String,
    pub value: MonValue,
    pub type_spec: Option<String>,
}

// --- Type and Symbol Table Definitions ---

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
    pub field_type: String,
    pub default_value: Option<MonValue>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct EnumDef {
    pub variants: Vec<String>,
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