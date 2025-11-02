# 04 - Defining Types: Structs and Enums

While MON is flexible, sometimes you need to enforce a specific structure for your data. The optional type system helps you do just that, preventing typos and ensuring consistency. The two main ways to define types are **Enums** and **Structs**.

### Enums (`#enum`): A Fixed Set of Choices

An Enum is a type that can only be one of several predefined values. It's perfect for things like status, categories, or modes.

**Syntax:**

```mon
{
    // Defines a type named `Status` that can only be one of these three values.
    Status: #enum {
        Active,
        Inactive,
        Pending,
    },
}
```

To use an enum value, you reference it with a dollar sign `$` followed by the type name and the variant name.

**Example Usage:**

```mon
{
    Status: #enum { Active, Inactive, Pending },

    // Using the enum value
    current_status: $Status.Active,
}
```

### Structs (`#struct`): Defining an Object's Schema

A Struct defines the "shape" of an object. It specifies what keys are allowed, what type their values should be, and can provide default values for optional keys.

**Syntax:**

```mon
{
    // Defines a schema for a User object
    User: #struct {
        // `id` is a required field of type Number
        id(Number),

        // `username` is a required field of type String
        username(String),

        // `email` is optional and defaults to null if not provided
        email(String) = null,

        // `is_active` is optional and defaults to `true`
        is_active(Boolean) = true,
    },
}
```

**Built-in Types:** You can use `String`, `Number`, `Boolean`, `Null`, `Array`, `Object`, and `Any` (which allows any value).

### Challenge 4: Define a Product Schema

Create a file `product.mon`. Inside it, define a `#struct` named `Product`. The `Product` struct should have:
1.  A required `id` of type `String`.
2.  A required `name` of type `String`.
3.  A required `price` of type `Number`.
4.  An optional `in_stock` field of type `Boolean` that defaults to `false`.

---

### Answer to Challenge 4

```mon
{
    Product: #struct {
        id(String),
        name(String),
        price(Number),
        in_stock(Boolean) = false,
    },
}
```

Fantastic! You've defined your first custom types. But how do you actually *use* them to validate your data? That's next!

---

**Next up**: [05 - Validating Data with Structs](05_validating_data.md)
