# MON Cookbook: Practical Recipes

Welcome to the MON Cookbook! This guide provides practical examples and patterns to help you solve common data-structuring problems using MON. Think of it as a collection of recipes for writing clean, efficient, and powerful configurations.

## Recipe 1: Your First Configuration File

MON is designed for human readability. Let's create a simple server configuration.

**Ingredients:**
*   Unquoted keys for simplicity.
*   Comments (`//`) for notes.
*   Basic data types: `String`, `Number`, `Boolean`.

**Instructions:**

```mon
// config.mon
{
    // The name of our application
    app_name: "Awesome App",

    // Network settings
    host: "localhost",
    port: 8080,

    // Feature flags. MON accepts on/true and off/false for booleans.
    enable_feature_x: on,
    enable_https: false,

    // A list of admin users
    admins: ["alice", "bob"],
}
```

**Key Takeaway:** MON looks familiar but is less strict than JSON. You can write configurations that are easier for people to read and edit.

---

## Recipe 2: Avoiding Repetition (The DRY Principle)

One of MON's most powerful features is its ability to reuse data within a file. This is perfect for defining a value once and using it multiple times.

**Ingredients:**
*   **Anchors (`&`):** To create a reusable template.
*   **Aliases (`*`):** To make a perfect, independent copy of a template.
*   **Spreads (`...*`):** To merge a template's content and add or override values.

### 2a. Creating a Reusable Template with Anchors

An anchor (`&`) gives a nickname to any value (`object`, `array`, `string`, etc.), turning it into a template.

```mon
{
    // The & makes `default_user` a template.
    // Note: The anchor itself doesn't appear in the final output.
    &default_user: {
        theme: "dark",
        notifications: on,
    },
}
```

### 2b. Making Copies with Aliases

An alias (`*`) makes a **deep copy** of an anchor. Each copy is completely separate.

```mon
{
    &default_user: { theme: "dark" },

    // alice_settings is a perfect copy of the template
    alice_settings: *default_user,

    // bob_settings is another, totally independent copy
    bob_settings: *default_user,
}
```

### 2c. Extending Templates with Spreads

A spread (`...*`) unpacks an anchor's content into a new object or array, allowing you to extend it.

**For Objects:** Spreads merge keys. If a key exists in both the template and your new object, the **local value wins**.

```mon
{
    &base_config: {
        host: "localhost",
        log_level: "info",
    },

    prod_config: {
        ...*base_config,      // 1. Inherit host and log_level
        host: "api.myapp.com", // 2. Override the host
        log_level: "error",    // 3. Override the log_level
    },
}
```

**For Arrays:** Spreads combine elements.

```mon
{
    &base_permissions: ["READ", "WRITE"],

    admin_permissions: [
        "LOGIN",
        ...*base_permissions, // Inserts "READ" and "WRITE" here
        "DELETE",
    ],
}
```

---

## Recipe 3: Modular Data with Imports

You can split your MON configurations across multiple files. All `import` statements must appear at the top of the file, before the opening `{` of the root object.

**Ingredients:**
*   `import * as ... from "path"`: The safest way to import.
*   `import { Member } from "path"`: To import specific, named parts.
*   `import { &Anchor } from "path"`: An advanced technique to share templates between files.

### 3a. Namespace Imports (Safest)

This bundles everything from another file into a single name, preventing conflicts.

**`schemas.mon`**
```mon
{
    User: #struct { name(String) },
    Status: #enum { Active, Inactive },
}
```

**`main.mon`**
```mon
// Import everything from schemas.mon into the `schemas` name
import * as schemas from "./schemas.mon"

{
    // Access imported members using the namespace
    admin :: schemas.User = { name: "Admin" },
    current_status: $schemas.Status.Active,
}
```

### 3b. Named Imports (Convenient)

Import specific parts directly into your file's scope.

**`main.mon`**
```mon
// Import only User and Status
import { User, Status } from "./schemas.mon"

{
    // Use them without a prefix
    guest :: User = { name: "Guest" },
    current_status: $Status.Inactive,
}
```

### 3c. Sharing Templates Across Files (Advanced)

You can import an anchor from another file. This makes the anchored *value* a new, local anchor in your current file, ready for reuse.

**`templates.mon`**
```mon
{
    // Define a reusable template for a base user
    &base_user: {
        permissions: ["READ"],
        is_active: true,
    },
}
```

**`main.mon`**
```mon
// Import the &base_user anchor from templates.mon
import { &base_user } from "./templates.mon"

{
    // Now you can use `base_user` as a local anchor!
    admin_user: {
        ...*base_user, // Spread the template
        name: "Admin User",
        permissions: ["READ", "WRITE", "DELETE"], // Override permissions
    },
}
```

---

## Recipe 4: Enforcing Data Quality

MON's optional type system helps you prevent mistakes by validating your data's structure.

**Ingredients:**
*   `#enum`: For creating a type with a fixed set of choices.
*   `#struct`: For defining the "blueprint" of an object.
*   `::`: The validation operator to check an object against a struct.

### 4a. Defining a Struct Blueprint

A struct defines what keys an object should have and what their value types should be.

```mon
{
    // A blueprint for a Product object
    Product: #struct {
        id(String),                 // Required String field
        price(Number),              // Required Number field
        in_stock(Boolean) = false,  // Optional Boolean, defaults to false
    },
}
```

### 4b. Validating Your Data

Use the `::` operator to validate an object against a struct. The MON compiler will report an error if the data doesn't match the blueprint.

```mon
{
    Product: #struct { id(String), price(Number) },

    // This object is valid and matches the Product struct
    my_book :: Product = {
        id: "978-0321765723",
        price: 55.99,
    },

    // INVALID: The compiler would throw an error here because `price` is a string.
    // my_widget :: Product = {
    //     id: "widget-123",
    //     price: "25.00",
    // },
}
```

---

## Common Pitfalls & Tips

*   **Spread Precedence:** When using a spread (`...*`), local keys always win. Any key you define in your object will override a key from the spread template.
*   **Keep It Simple:** Anchors and spreads are powerful, but creating long, complex chains of them can make your data hard to debug. Prefer clarity and simplicity.
*   **Start with Namespace Imports:** When learning, `import * as ...` is your friend. It's the clearest way to see where data is coming from and prevents accidental name clashes.