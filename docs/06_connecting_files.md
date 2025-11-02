# 06 - Connecting Files: The Module System

Real-world projects are often split into multiple files for better organization. MON's module system allows you to connect these files, sharing types and data in a clear and explicit way.

### Implicit Exports: Everything is Public

In MON, the module system is simple: **every key-value pair in a file's root object is automatically available to be imported by other files.** You don't need a special `export` keyword.

### Importing an Entire File (`import * as ...`)

This is the safest and most common way to import. It bundles all the members of another file into a single, namespaced object.

**Example:**

Let's say you have a `schemas.mon` file defining your types:

`schemas.mon`
```mon
{
    User: #struct {
        name(String),
    },
    Status: #enum {
        Active,
        Inactive,
    },
}
```

Now, in your `main.mon` file, you can import it:

`main.mon`
```mon
{
    // Import everything from schemas.mon into an object named `schemas`
    import * as schemas from "./schemas.mon"

    // Now you can access the imported types via the namespace
    admin_user :: schemas.User = {
        name: "Admin",
    },

    current_status: $schemas.Status.Active,
}
```

### Importing Specific Members (`import { ... }`)

You can also import specific members from another file directly into your current file's scope. This is useful for commonly used types.

**Example:**

`main.mon`
```mon
{
    // Import only the User struct and Status enum directly
    import { User, Status } from "./schemas.mon"

    // Now you can use them without the `schemas.` prefix
    guest_user :: User = {
        name: "Guest",
    },

    current_status: $Status.Inactive,
}
```

### Challenge 6: Build a Modular Config

1.  Create a file `db_config.mon` that defines a `#struct` named `Database` with a `host(String)` field.
2.  Create a file `app.mon`.
3.  In `app.mon`, use a namespace import (`import * as ...`) to import `db_config.mon`.
4.  Create a validated instance of the `Database` struct using the imported namespace.

---

### Answer to Challenge 6

`db_config.mon`
```mon
{
    Database: #struct {
        host(String),
    },
}
```

`app.mon`
```mon
{
    import * as db from "./db_config.mon"

    production_db :: db.Database = {
        host: "api.production.com",
    },
}
```

Congratulations! You now know how to structure a multi-file MON project. Next, we'll look at some common mistakes and how to fix them.

---

**Next up**: [07 - Common Mistakes and How to Fix Them](07_common_mistakes.md)
