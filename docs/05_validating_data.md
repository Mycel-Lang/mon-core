# 05 - Validating Data with Structs

Defining a `#struct` is like creating a blueprint. Now, let's learn how to use that blueprint to check if your data is built correctly. This is done with the `::` validation operator.

### The `::` Operator: Checking Your Data

You use `::` after a key to declare, "The following object must match this struct's blueprint." The MON compiler will then check the object for you.

**Example:**

```mon
{
    // 1. Define the blueprint
    User: #struct {
        id(Number),
        username(String),
        is_active(Boolean) = true,
    },

    // 2. Use the blueprint to validate an instance
    alice :: User = {
        id: 1,
        username: "alice",
    },

    // This instance would cause an error because `id` is a string, not a number.
    // bob :: User = {
    //     id: "2",
    //     username: "bob",
    // },
}
```

If validation fails, the compiler will give you a clear error telling you exactly what went wrong (e.g., missing key, wrong value type, or extra key).

### Advanced Validation: Collection Types

The type system can also validate the contents of arrays. This is incredibly powerful for ensuring lists of data are consistent.

Here are the patterns you can use inside a struct definition:

| Pattern | Meaning |
| :--- | :--- |
| `[String]` | An array with **exactly one** element of type `String`. |
| `[Number...]` | An array with **zero or more** elements, all of type `Number`. |
| `[String, Number]` | A "tuple" array with **exactly two** elements: a `String` first, then a `Number`. |
| `[String, Any...]` | An array with **one or more** elements. The first must be a `String`, and the rest can be `Any` type. |
| `[Boolean..., String]` | An array with **one or more** elements. The last must be a `String`, and all preceding elements must be `Boolean`. |

**Note:** The `Any` type is a special wildcard that matches any valid MON value.

**Example of Collection Validation:**

```mon
{
    LogEntry: #struct {
        // The `data` field must be an array starting with a String, followed by any other items.
        data([String, Any...]),
    },

    // This instance is valid.
    login_event :: LogEntry = {
        data: ["USER_LOGIN", { id: 123, time: "2025-10-26" }],
    },

    // This instance would fail because it doesn't start with a String.
    // invalid_event :: LogEntry = {
    //     data: [123, "USER_LOGIN"],
    // },
}
```

### Challenge 5: Validate a Product

Using your `product.mon` file from the last challenge, create a new key `my_laptop`. Use the `::` operator to validate it against your `Product` struct. Fill in the object with valid data for a laptop.

---

### Answer to Challenge 5

```mon
{
    Product: #struct {
        id(String),
        name(String),
        price(Number),
        in_stock(Boolean) = false,
    },

    my_laptop :: Product = {
        id: "PROD-12345",
        name: "SuperBook Pro 15-inch",
        price: 2499.99,
        in_stock: true,
    },
}
```

You are now able to define and validate complex data structures! The final step is to learn how to connect data across different files.

---

**Next up**: [06 - Connecting Files: The Module System](06_connecting_files.md)
