# 09 - MON EBNF Grammar Reference

This document provides a comprehensive EBNF (Extended Backus-Naur Form) grammar for the MON language. It is intended as
a formal supplement to the syntax reference.

**Notation:**

* `::=` means "is defined as".
* `|` separates alternatives.
* `[ ... ]` indicates an optional item.
* `{ ... }` indicates repetition (zero or more times).
* `( ... )` is used for grouping.
* Terminals (like `"{`", `":"`, `"#struct"`) are enclosed in double quotes.

---

### 1. Top-Level Structure

```ebnf
Document ::= { ImportStatement } Object
```

### 2. Values

```ebnf
Value ::= Object
        | Array
        | Alias
        | EnumValue
        | Literal

Literal ::= String | Number | Boolean | Null
```

### 3. Object and Array

```ebnf
Object ::= "{" [ MemberList ] "}"

MemberList ::= Member { "," Member } [ "," ]

Array ::= "[" [ ValueList ] "]"

ValueList ::= Value { "," Value } [ "," ]
```

### 4. Object Members

```ebnf
Member ::= Pair | TypeDefinition | Spread

(* A key-value pair, which may include validation. *)
Pair ::= KeyPart [ Validation ] ( ":" | "=" ) Value

KeyPart ::= [ Anchor ] Key

Key ::= Identifier | String
```

### 5. Composition (Anchors, Aliases, Spreads)

```ebnf
Anchor ::= "&" Identifier

Alias ::= "*" Identifier

Spread ::= "..." Alias
```

### 6. Type System

```ebnf
(* A type definition is a key-value pair where the value is a struct or enum. *)
TypeDefinition ::= Identifier ":" ( StructDefinition | EnumDefinition )

StructDefinition ::= "#struct" "{" [ FieldList ] "}"

FieldList ::= FieldDefinition { "," FieldDefinition } [ "," ]

FieldDefinition ::= Identifier "(" Type ")" [ "=" Value ]

EnumDefinition ::= "#enum" "{" [ Identifier { "," Identifier } [ "," ] ] "}"

(* Validation is attached to a key in a Pair. *)
Validation ::= "::" Type

(* A Type can be a collection, a user-defined type, or a built-in primitive. *)
Type ::= CollectionType | Identifier | "String" | "Number" | "Boolean" | "Null" | "Object" | "Array" | "Any"

(* Array/collection type specifier, e.g., [String], [String...], [String, Number] *)
CollectionType ::= "[" Type [ "..." ] { "," Type [ "..." ] } "]"

(* Accessing a variant of an enum, e.g., $MyEnum.Variant *)
EnumValue ::= "$" Identifier "." Identifier
```

### 7. Module System

```ebnf
ImportStatement ::= "import" ( NamespaceImport | NamedImport ) "from" String

NamespaceImport ::= "*" "as" Identifier

NamedImport ::= "{" [ ImportSpecifier { "," ImportSpecifier } [ "," ] ] "}"

ImportSpecifier ::= [ "&" ] Identifier
```

### 8. Lexical Primitives

```ebnf
Identifier ::= (A-Z | a-z | "_") { A-Z | a-z | 0-9 | "_" }

String ::= '"' { Any character with standard JSON escapes for \", \\, etc. } '"'

Number ::= [ "-" ] ( "0" | 1-9 { 0-9 } ) [ "." { 0-9 } ]

Boolean ::= "true" | "false" | "on" | "off"

Null ::= "null"
```
