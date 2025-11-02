# 07 - Common Mistakes and How to Fix Them

As you build more complex MON files, you'll inevitably run into errors. This is a normal part of the process! The MON compiler is designed to give you clear, helpful feedback to pinpoint the exact problem.

Here are some common mistakes and how to interpret the error messages.

### 1. Syntax Errors

These are typos or structural mistakes where the file breaks the basic rules of the MON language.

**Example: Missing a comma**
```mon
{
    host: "localhost"
    port: 8080
}
```
**Error Message:** The parser will expect a comma after `"localhost"` but will find the key `port` instead. It will report an `Unexpected Token` error on line 3.

**How to Fix:** Read the error carefully. It often tells you what it expected. Go to the specified line and fix the typo.

**Example: Missing Root Object**
```mon
// This file is missing the surrounding {...}
name: "My App"
```
**Error Message:** The parser expects every file to start with `{`. It will report an error on the first line.

**How to Fix:** Ensure your entire file content is wrapped in a single root object `{...}`.

### 2. Resolution Errors

These errors happen when the compiler tries to connect your files and data, but can't find something.

**Example: Importing a non-existent member**
```mon
// main.mon
{
    import { NonExistentType } from "./schemas.mon"
}
```
**Error Message:** `Resolution Error: Member 'NonExistentType' not found in module './schemas.mon'.`

**How to Fix:** Check the `schemas.mon` file to ensure you are exporting a member with that exact name and that you haven't made a typo.

**Example: Circular Import**
```mon
// a.mon
{ import * as b from "./b.mon" }

// b.mon
{ import * as a from "./a.mon" }
```
**Error Message:** `Resolution Error: Circular dependency detected: a.mon -> b.mon -> a.mon`

**How to Fix:** You must restructure your code to break the loop. Often, this means creating a third file that both `a.mon` and `b.mon` can import from without importing each other.

### 3. Validation Errors

These happen when you use the `::` operator, and the data on the right-hand side doesn't match the `#struct` blueprint.

**Example: Wrong data type**
```mon
{
    User: #struct { id(Number) },
    my_user :: User = { id: "user-123" } // id should be a Number, not a String
}
```
**Error Message:** `Validation Error: Type mismatch for field 'id'. Expected Number, but got String.`

**How to Fix:** Correct the value to match the type defined in the struct (e.g., `id: 123`).

**Example: Extra field**
```mon
{
    User: #struct { name(String) },
    my_user :: User = {
        name: "Alice",
        age: 30 // The 'age' field is not defined in the User struct
    }
}
```
**Error Message:** `Validation Error: Found unexpected field 'age' in struct 'User'.`

**How to Fix:** Remove the extra field or add it to the `#struct` definition if it's meant to be there.

---

You've now seen how to write MON and how to debug it. The final page in this guide is a complete syntax reference to help you as you build your own projects.

---

**Next up**: [08 - Full Syntax Reference](08_syntax_reference.md)
