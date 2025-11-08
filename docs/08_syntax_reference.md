# 08 - MON Full Syntax Reference

This document provides a complete, quick-reference guide to all syntax and features in the MON language.

### 1. File Structure

A valid MON file can optionally start with one or more `import` statements, followed by a single root object.

```mon
// Optional imports first
import * as my_module from "./my_module.mon"

// Then, the single root object
{
    // All content, including declarations and values, goes inside this object.
}
```

### 2. Primitives

| Type | Examples |
| :--- | :--- |
| **String** | `"hello"`, `"with spaces"` |
| **Number** | `123`, `-45.6` |
| **Boolean**| `true`, `false`, `on`, `off` |
| **Null** | `null` |

### 3. Comments

Single-line comments start with `//`.

```mon
{
    // This is a comment.
    key: "value", // This is also a comment.
}
```

### 4. Objects and Keys

*   Objects are collections of `key: value` pairs inside `{}`.
*   Keys can be unquoted identifiers or quoted strings.
*   A trailing comma is allowed after the last member.

```mon
{
    unquoted_key: "value",
    "quoted-key-with-hyphens": 123,
}
```

### 5. Arrays

*   Arrays are ordered lists of values inside `[]`.
*   A trailing comma is allowed after the last element.

```mon
{
    my_array: ["a", 1, true, null],
}
```

### 6. Composition

| Feature | Syntax | Description |
| :--- | :--- | :--- |
| **Anchor** | `&my_anchor: value,` | Gives a `value` a file-local nickname. Must be a key-value pair. |
| **Alias** | `*my_anchor` | Creates a deep copy of the anchored value. |
| **Object Spread** | `{ ...*my_anchor }` | Merges keys from an anchored object. Local keys override spread keys. |
| **Array Spread** | `[ ...*my_anchor ]` | Inserts elements from an anchored array into a new array. |

### 7. Type System

| Feature | Syntax | Description |
| :--- | :--- | :--- |
| **Enum Definition** | `MyEnum: #enum { A, B },` | Defines a type with a fixed set of choices. |
| **Enum Access** | `$MyEnum.A` | References a specific variant of an enum. |
| **Struct Definition**| `MyStruct: #struct { f(T), g(N)=d },` | Defines a schema for an object. `f` is a required field of type `T`. `g` is an optional field of type `N` with a default value `d`. |
| **Struct Validation**| `my_instance :: MyStruct = { ... }` | Validates that the object literal on the right conforms to the `MyStruct` schema. |

### 8. Collection Types

Used within `#struct` field definitions (e.g., `my_field([String...])`).

| Pattern | Meaning |
| :--- | :--- |
| `[T]` | An array with **exactly one** element of type `T`. |
| `[T...]` | An array with **zero or more** elements, all of type `T`. |
| `[T1, T2]` | A tuple with **exactly two** elements of specified types. |
| `[T1, T2...]` | An array with **one or more** elements, where the first is `T1` and the rest are `T2`. |
| `[T1..., T2]` | An array with **one or more** elements, where the last is `T2` and the rest are `T1`. |
| `Any` | A special type that matches any value. |

### 9. Module System

Import statements must be declared at the top of a file, before the root object.

*   **Implicit Exports**: All top-level keys in a file are importable.
*   **Namespace Import**: `import * as ns from "./file.mon"`
*   **Named Import**: `import { Member1, &Anchor2 } from "./file.mon"`

---

This concludes the MON "0 to Hero" guide. You are now equipped with the full knowledge of the MON language.
