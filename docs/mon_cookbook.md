# Mycel Object Notation (`.mon`) Cookbook

This document provides a practical guide to using the `mon` library for parsing, resolving, and interacting with `.mon` data files.

## Core Components

The `mon` library is composed of three main parts:

1.  **Lexer**: Turns a source string into a stream of `Token`s.
2.  **Parser**: Consumes tokens from the lexer to build an Abstract Syntax Tree (AST).
3.  **Resolver**: Traverses the AST to handle file imports, detect circular dependencies, and build a complete dependency graph.

---

## 1. The Lexer

The lexer is the first step in the compilation pipeline. It performs lexical analysis, converting the raw source text into a sequence of tokens that the parser can understand.

### Usage

You can create a `Lexer` from any string slice. The lexer is an `Iterator`, so you can use standard iterator methods to process the tokens.

```rust
use mycelc::mon::lexer::{Lexer, Token};

let source = r#"
{
    // This is a comment
    "key": "value",
    valid: true,
}
"#;

let lexer = Lexer::new(source);
let tokens: Vec<_> = lexer.map(|r| r.unwrap().value).collect();

assert_eq!(tokens, vec![
    Token::LBrace,
    Token::String("key"),
    Token::Colon,
    Token::String("value"),
    Token::Comma,
    Token::Identifier("valid"),
    Token::Colon,
    Token::Boolean(true),
    Token::Comma,
    Token::RBrace,
    Token::Eof,
]);
```

### Features

-   **JSON Superset**: Handles all standard JSON syntax plus comments (`//`), trailing commas, and unquoted identifiers for keys.
-   **MON Extensions**: Recognizes `import()`, anchors (`&`), aliases (`*`), and the spread operator (`...`).
-   **Error Reporting**: Produces specific errors for unterminated strings, invalid numbers, and unexpected characters.

---

## 2. The Parser

The parser takes the token stream from the lexer and constructs an Abstract Syntax Tree (AST) that represents the semantic structure of the document.

### AST Structure

The main AST nodes are defined in `mycelc::mon::ast`:

-   `MonDocument`: The root of a parsed file.
-   `MonValue`: Represents any value in MON, containing a `MonValueKind` and optional metadata like an `anchor`.
-   `MonValueKind`: An enum for the different types of values (`Object`, `Array`, `String`, `Import`, `Alias`, etc.).
-   `Member`: A member of an `Object`, which can be a `Pair` (key-value) or a `Spread`.

### Usage

Create a `Parser` and call the `parse_document()` method.

```rust
use mycelc::mon::parser::Parser;

let source = r#"{
    user: &main_user {
        name: "Alice",
        id: 123,
    }
}"#;

let mut parser = Parser::new(source);
let ast = parser.parse_document();

assert!(ast.is_ok());
println!("{:#?}", ast.unwrap());
```

---

## 3. The Resolver

The resolver is the high-level entry point for handling `.mon` files and their dependencies. It orchestrates the lexer and parser and manages a cache to avoid re-parsing files.

### Usage

The primary method is `resolve()`, which takes the path to a root `.mon` file.

```rust
use mycelc::mon::resolver::Resolver;
use std::path::Path;

// Assuming you have test_files/mon/resolve_root.mon and its imports.
let mut resolver = Resolver::new();
let root_path = Path::new("test_files/mon/resolve_root.mon");

match resolver.resolve(root_path) {
    Ok(document) => {
        println!("Successfully resolved document!");
        // You can now inspect the AST in `document`.
        // Note: `import` nodes are still present in the AST.
    }
    Err(e) => {
        // The resolver returns a miette::Report for rich diagnostics.
        eprintln!("{:?}", e);
    }
}
```

### Features

-   **Dependency Resolution**: Recursively loads and parses all `import()` statements.
-   **Caching**: Caches parsed files in memory to prevent redundant file I/O and parsing, significantly improving performance on projects with many imports.
-   **Circular Dependency Detection**: Automatically detects and reports an error if an import cycle is found.
-   **Rich Error Reporting**: Uses `miette` to produce beautiful, informative error messages that point directly to the source code location of the error (e.g., file not found, parse error, circular import).

#### Example Error (Circular Dependency)

If `a.mon` imports `b.mon` and `b.mon` imports `a.mon`, the resolver will produce an error like this:

```text
Error: ⅇ Circular dependency detected

  × Circular dependency detected
   ╭─[test_files/mon/circ_b.mon:2:13]
 2 │     b: import("circ_a.mon")
   ·             ────────── import causing cycle
   ╰─▶ 
```

---

## 4. Best Practices & Case Studies

This section provides advice on how to write clean, maintainable `.mon` files.

### Prefer Readability

`.mon` is a superset of JSON, but its extra features are designed to improve human readability. Take advantage of them.

-   **DO** use unquoted keys for simple identifiers.
-   **DO** use comments (`//`) to explain complex data structures or the purpose of certain values.
-   **DON'T** write `.mon` as if it were strict JSON. While valid, it's less ergonomic.

**Good:** [`docs/mon_examples/config.mon`](mon_docs/mon_examples/config.mon)
```mon
// Server settings
server: {
    host: "127.0.0.1",
    port: 8080,
},
```

**Bad:** [`docs/mon_examples/config_bad.mon`](mon_docs/mon_examples/config_bad.mon)
```mon
{
    "server": {
        "host": "127.0.0.1",
        "port": 8080
    }
}
```

### Compose Configuration with Imports

A powerful pattern is to define a `base.mon` file with common defaults, and then create environment-specific files that import and override it.

This keeps your configurations DRY (Don't Repeat Yourself).

See the following example files for a demonstration:
-   [`docs/mon_examples/base.mon`](mon_docs/mon_examples/base.mon)
-   [`docs/mon_examples/development.mon`](mon_docs/mon_examples/development.mon)
-   [`docs/mon_examples/production.mon`](mon_docs/mon_examples/production.mon)

### Use Anchors for Repetition Within a File

Anchors (`&`) and aliases (`*`) are perfect for reusing the same data structure multiple times within a single file.

```mon
{
    default_user: &user {
        permissions: ["read"],
        features: [],
    },

    user_a: *user,
    user_b: *user,
}
```

### Common Pitfalls

#### Alias Shadowing with Spread

When you use the spread operator (`...*alias`), any keys defined locally in the object will **always** take precedence over the keys from the alias. This can be a source of confusion if you're not aware of it.

See [`docs/mon_examples/shadowing.mon`](mon_docs/mon_examples/shadowing.mon) for an example.

#### Overly Complex Aliases

While aliases are powerful, creating long chains or complex webs of aliases can make your data difficult to understand and debug. Prefer clarity and simplicity.

See [`docs/mon_examples/overly_complex.mon`](mon_docs/mon_examples/overly_complex.mon) for an example of what to avoid.

---

## Deep Dive into MON's Compositional Features: `&`, `*`, and `...`

Mycel Object Notation (`.mon`) extends JSON with powerful features designed to make static data graphs more ergonomic, maintainable, and compositional. The `&` (anchor), `*` (alias), and `...` (spread) operators are central to this philosophy, enabling developers to define reusable data blocks and construct complex configurations without repetition.

### 1. Philosophy of Composition

Traditional JSON, while universal, can become verbose and repetitive for complex configurations or data definitions. MON addresses this by introducing mechanisms for:

*   **DRY (Don't Repeat Yourself):** Avoid duplicating identical or nearly identical data structures.
*   **Modularity:** Break down large data definitions into smaller, manageable, and reusable units.
*   **Static Data Graphs:** Build interconnected data structures that are resolved entirely at compile-time, ensuring predictability and preventing runtime side-effects.

These features operate strictly within the confines of a single `.mon` file or across explicitly imported files, maintaining a clear and predictable data flow.

### 2. Anchors (`&`)

**Purpose:** An anchor allows you to name a specific data value (a primitive, object, or array) within a single `.mon` file, making it reusable elsewhere in that *same file*.

**Syntax:** `&anchor_name value`

The `&` symbol precedes an identifier (`anchor_name`), which is immediately followed by the value it names.

**Example:**

```mon
{
    // Define a reusable set of database credentials
    db_credentials: &prod_db_creds {
        host: "prod.example.com",
        user: "admin",
        password: "secure_password",
    },
    // ... other data
}
```

**Behavior:**

*   When the parser encounters `&prod_db_creds { ... }`, it associates the identifier `prod_db_creds` with the object `{ host: ..., user: ..., password: ... }`.
*   The anchored value itself is part of the document's structure. In the example above, `db_credentials` would be a key whose value is the anchored object.

**Scope:**

*   **Strictly File-Local:** This is a fundamental rule. An anchor defined in `file_A.mon` cannot be referenced by an alias in `file_B.mon`, even if `file_B.mon` imports `file_A.mon`. This design choice prevents naming conflicts across files and ensures that each `.mon` file remains a self-contained, understandable unit.

**Implementation Details:**

*   **Lexer (`src/mon/lexer.rs`):**
    *   The lexer identifies the `&` character followed by an identifier (alphanumeric or underscore characters).
    *   It produces a `Token::Anchor(&'a str)` where the `&'a str` is the `anchor_name`.
*   **Parser (`src/mon/parser.rs`):**
    *   The `parse_value` function checks for an `&` token before parsing the actual value.
    *   If an `&` is found, the `anchor_name` is extracted from the token and stored in the `anchor` field of the `MonValue` struct in the AST.
*   **Resolver (`src/mon/resolver.rs` - future processing):**
    *   The resolver's current role is to load and parse files. The actual resolution of anchors and aliases (i.e., building a symbol table for each file and performing substitutions) would occur in a subsequent AST transformation pass, after all files have been loaded and parsed. This pass would build a map of `anchor_name -> MonValue` for each document.

### 3. Aliases (`*`)

**Purpose:** An alias allows you to reuse a value that has been previously defined with an anchor within the *same `.mon` file*.

**Syntax:** `*anchor_name`

The `*` symbol precedes the `anchor_name` that refers to a previously defined anchor.

**Example:**

```mon
{
    db_credentials: &prod_db_creds {
        host: "prod.example.com",
        user: "admin",
        password: "secure_password",
    },

    // Reuse the production database credentials for a reporting service
    reporting_service_db: *prod_db_creds,

    // Another service using the same credentials
    analytics_db: *prod_db_creds,
}
```

**Behavior:**

*   When an alias `*prod_db_creds` is encountered, it is conceptually replaced by a **deep copy** of the value associated with the `prod_db_creds` anchor.
*   **Deep Copy Semantics:** This is critical. If the aliased value is an object or an array, any modifications made to `reporting_service_db` (e.g., adding a new field) will *not* affect `db_credentials` or `analytics_db`. Each alias gets its own independent copy of the data.

**Scope:**

*   **Strictly File-Local:** Like anchors, aliases can only refer to anchors defined within the same `.mon` file.

**Implementation Details:**

*   **Lexer (`src/mon/lexer.rs`):**
    *   The lexer identifies the `*` character followed by an identifier.
    *   It produces a `Token::Alias(&'a str)` where the `&'a str` is the `anchor_name`.
*   **Parser (`src/mon/parser.rs`):**
    *   The `parse_value` function recognizes `Token::Alias` and creates a `MonValueKind::Alias` node in the AST, storing the `anchor_name`.
*   **Resolver (`src/mon/resolver.rs` - future processing):**
    *   A later AST transformation pass would iterate through the AST. When it encounters a `MonValueKind::Alias` node, it would:
        1.  Look up `anchor_name` in the file's symbol table (map of anchors).
        2.  Retrieve the `MonValue` associated with the anchor.
        3.  Perform a deep copy of that `MonValue`.
        4.  Replace the `MonValueKind::Alias` node with the deep copy.

### 4. Spread Operator (`...`)

**Purpose:** The spread operator provides a concise way to merge the members (key-value pairs) of an aliased object into another object. It's particularly useful for extending or overriding base configurations.

**Syntax:** `...*anchor_name` (used as a member within an object)

The `...` must be immediately followed by an alias expression (`*anchor_name`).

**Example:**

```mon
{
    // Base configuration for a service
    &base_service_config {
        port: 8080,
        timeout_ms: 2000,
        log_level: "info",
    },

    // Development service configuration
    dev_service: {
        ...*base_service_config, // Inherit all members from base_service_config
        port: 3000,              // Override the port for development
        debug_mode: on,          // Add a new field
    },

    // Production service configuration
    prod_service: {
        ...*base_service_config,
        port: 80,
        log_level: "error",      // Override log level for production
    },
}
```

**Behavior:**

*   The `...*anchor_name` construct is only valid as a member within an object.
*   It instructs the system to take all key-value pairs from the object referenced by `*anchor_name` and include them in the current object.
*   **Conflict Resolution (Shallow Merge):** If a key from the aliased object already exists in the current object (or is defined later in the same object), the local definition takes precedence (it "wins"). This is a shallow merge; nested objects are not recursively merged.

**Scope:**

*   **Strictly File-Local:** Since it relies on aliases, the spread operator is also strictly file-local.

**Implementation Details:**

*   **Lexer (`src/mon/lexer.rs`):**
    *   The lexer identifies the `...` sequence as `Token::Spread`.
*   **Parser (`src/mon/parser.rs`):**
    *   In the `parse_object` function, when `Token::Spread` is encountered, the parser then expects a `MonValue` of `MonValueKind::Alias`.
    *   It creates a `Member::Spread` node in the AST, storing the `anchor_name` from the alias.
*   **Resolver (`src/mon/resolver.rs` - future processing):**
    *   A later AST transformation pass would process `Member::Spread` nodes. It would:
        1.  Resolve the `anchor_name` to retrieve the aliased `MonValue`.
        2.  Verify that the aliased `MonValue` is an `Object`. If not, it's a semantic error.
        3.  Iterate through the members of the aliased object and add them to the current object.
        4.  During this merge, it would apply the conflict resolution rule: if a key from the aliased object already exists in the current object, the local definition is kept.

### 5. Interaction with Imports (`import()`)

The `import()` mechanism allows you to compose `.mon` files from other `.mon` files. A crucial aspect of MON's design is how anchors, aliases, and spreads interact with imports.

**Key Rule: Anchors and Aliases are STRICTLY FILE-LOCAL.**

**Behavior:**

*   When `import("path/to/file.mon")` is encountered, the `Resolver` loads and parses `file.mon`.
*   **Internal Resolution:** Any anchors and aliases *within* `file.mon` are resolved *before* `file.mon`'s content is conceptually "substituted" into the importing file.
*   **No Cross-File Anchor/Alias Visibility:**
    *   Anchors defined in `file_A.mon` are **not** visible or usable by aliases in `file_B.mon` (the imported file).
    *   Anchors defined in `file_B.mon` are **not** exposed to `file_A.mon` (the importing file).
*   **Content Substitution:** The `import()` effectively replaces itself with the fully resolved content of the imported `.mon` file.

**Example:**

`base.mon`:
```mon
{
    &common_settings {
        version: "1.0",
        debug: false,
    },
    // ...
}
```

`config.mon`:
```mon
{
    // This import brings in the *content* of base.mon.
    // The `common_settings` anchor from base.mon is NOT directly accessible here.
    base_data: import("./base.mon"),

    // This would be an error, as `common_settings` is not anchored in config.mon's scope.
    // my_settings: *common_settings,
}
```

**Implementation Details (Resolver):**

*   The `Resolver`'s `load_and_parse` function handles the recursive loading of imported files.
*   The `walk_imports` function identifies `MonValueKind::Import` nodes in the AST.
*   When an `Import` node is found, the `Resolver` recursively calls `load_and_parse` for the imported file.
*   The result of `load_and_parse` is an `Arc<MonDocument>`.
*   The actual "substitution" of the imported AST into the importing AST (replacing the `Import` node with the content of the imported document) is a task for a later AST transformation pass, which would operate on the fully resolved graph of `MonDocument`s provided by the `Resolver`.

### 6. Why this design? (Philosophy Revisited)

The strict file-local scoping of `&`, `*`, and `...` in conjunction with `import()` is a deliberate design choice that prioritizes:

*   **Predictability:** You always know where an anchor or alias is defined and what its scope is. There's no "spooky action at a distance" where a change in one file unexpectedly affects another through a hidden alias.
*   **Modularity:** Each `.mon` file can be understood and reasoned about in isolation, making it easier to move, refactor, and reuse.
*   **Security:** By limiting the scope of these compositional features, the system reduces the attack surface and prevents unintended data leakage or manipulation across file boundaries.
*   **Static Analysis:** Tools (like an LSP or a linter) can more easily analyze and validate `.mon` files because the resolution rules are clear and local.
