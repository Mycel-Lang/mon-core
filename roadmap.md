# MON (Mycel Object Notation) Core - Roadmap

Based on the project's goals, here is a proposed roadmap for `mon-core`, outlining the key phases to build a robust and
feature-rich library.

### Phase 1: Core Parsing and Syntax (Largely Complete)

This phase focuses on correctly parsing the MON language syntax into a structured representation.

* **[Done!] Lexer Implementation:** Convert raw text into a stream of tokens.
* **[Done!] AST/CST Definition:** Define the Abstract Syntax Tree that represents the structure of a MON document,
  including all language features (objects, arrays, anchors, types, etc.).
* **[Done!] Parser Implementation:** Build the AST from the token stream. The parser is designed to be error-tolerant to
  support future LSP features.
* **[Done!] Basic Error Reporting:** Implement initial syntax error reporting using `miette` for clear diagnostics.

### Phase 2: Semantic Analysis and Resolution (In Progress)

This phase gives meaning to the syntax tree by resolving references and validating rules.

* **[Done!] Import Resolution:** Handle `import` statements to read and parse dependent files.
* **[Done!] Circular Dependency Detection:** Prevent infinite loops during import resolution.
* **[Done!] Anchor, Alias, and Spread Resolution:** Implement the core composition logic to correctly handle `&`, `*`,
  and `...*`.
* **[In Progress] Type System Validation:**
    * **[Done!]** Validate data against `#struct` and `#enum` definitions within a single file.
    * **[Done!]** Validate data that uses types imported from other files.
    * **[Done!]** Enforce type rules for fields, including checking for missing/extra fields and type mismatches.
    * **[Done!]** Implement default value injection for optional struct fields.
* **[DONE!] Advanced Collection Validation:** Fully implement validation for complex array types (e.g.,
  `[String, Number...]`).

### Phase 3: Public API and Tooling

This phase focuses on creating a stable public API that external tools can use.

* **[To-Do] Create a Public `parse` Function:** Expose a simple top-level function that takes a MON string and returns a
  fully resolved and validated result.
* **[To-Do] JSON Serialization:** Provide a mechanism to convert a resolved MON document into a canonical JSON string,
  which is the most common use case for consumers.
*   **[x] Improve API Ergonomics:** Refine the public-facing data structures and error types to be intuitive and easy to use for library consumers.

### Phase 4: Language Server and Developer Experience

With a stable core library, the focus shifts to building a rich developer experience.

* **[To-Do] LSP Wrapper Crate:** Create a new crate that wraps `mon-core` and implements the Language Server Protocol.
* **[To-Do] Implement Core LSP Features:**
    * **Diagnostics:** Provide real-time parsing and semantic error feedback in the editor.
    * **Syntax Highlighting:** (Handled by editor extensions, but the parser enables it).
    * **Go to Definition:** Allow jumping from an alias/spread to its anchor, or from a variable to its type definition.
    * **Auto-Completion:** Suggest keys, enum variants, and types.
* **[To-Do] Advanced LSP Features:**
    * **Hover Information:** Show type information and documentation on hover.
    * **Code Formatting:** Implement a canonical formatter for MON files.

### Phase 5: Compiler and Integration (`mycelc`)

The final phase is to use `mon-core` as the engine for a dedicated MON compiler.

* **[To-Do] `mycelc` Crate:** Create the compiler binary (`mycelc`).
* **[To-Do] Command-Line Interface:** Design a CLI for compiling MON files to JSON, validating files, and other
  utilities.
* **[To-Do] Integration:** Use `mon-core` as a library to handle all the parsing, resolution, and validation logic
  within the compiler.
