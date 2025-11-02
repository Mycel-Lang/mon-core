# MON (Mycel Object Notation) Core

This project is the core Rust implementation for **MON (Mycel Object Notation)**, a human-friendly data notation
language.

## High-Level Overview: Building a MON Library for LSP and Compiler

To build a robust library for MON that can power both a Language Server Protocol (LSP) and a compiler like `mycelc`, we
need a layered architecture that separates parsing the text from understanding its meaning.

Here is a high-level overview of the essential components:

### 1. The Parser & Concrete Syntax Tree (CST/AST)

This is the foundation. The parser's job is to read the raw MON text and turn it into a tree structure that represents
the code exactly as it was written, including comments, whitespace, and source code positions (spans).

* **Input:** Raw MON text as a string.
* **Output:** An **Abstract Syntax Tree (AST)** or a **Concrete Syntax Tree (CST)**.
* **Key Requirement (for LSP):** It must be **error-tolerant**. It should be able to produce a tree even if the code is
  incomplete or has syntax errors, allowing the LSP to provide feedback as the user types.

### 2. The Semantic Analyzer & Semantic Model

This layer takes the raw syntax tree from the parser and gives it meaning. It understands the rules of MON beyond just
the basic syntax.

* **Input:** The AST/CST from the parser.
* **Responsibilities:**
    * **Validation:** Check for errors like duplicate keys, invalid values, etc.
    * **Reference Resolution:** Resolve anchors and spreads, connecting reused data.
    * **Type Checking:** If the MON file uses type definitions, this layer validates that the data conforms to those
      types.
* **Output:** A **Semantic Model**. This is a cleaner, high-level representation of the data's meaning, stripped of
  purely syntactical details. It represents the fully resolved and validated MON data.

### 3. A Query and Traversal API

This is the interface that consumers like the LSP and `mycelc` will use to interact with the data. It provides functions
to navigate and inspect both the raw syntax tree and the semantic model.

* **Examples:**
    * `getNodeAt(position)`: For an LSP to find what the cursor is pointing at.
    * `getTypeOf(node)`: To get the validated type of a value.
    * `getDefinition(reference)`: To find where an anchor is defined.

### How They Work Together

The data flows through the library in a pipeline:

```
Raw Text -> [Parser] -> AST/CST -> [Semantic Analyzer] -> Semantic Model
   ^             ^           ^               ^                 ^
   |             |           |               |                 |
(User Input)  (Syntax       (LSP uses      (Semantic         (Compiler/LSP
              Errors)      for basic       Errors)           uses for deep
                           highlighting)                     understanding)
```

By separating the components, we create a powerful and flexible library:

* The **LSP** can primarily use the **AST/CST** for fast, syntax-aware features like highlighting and formatting, and
  use the **Semantic Model** for deeper understanding like "go to definition" and type-aware autocompletion.
* The **`mycelc` compiler** can skip the raw syntax details and work directly with the clean, validated **Semantic Model
  **, confident that the data is well-formed and correct.
