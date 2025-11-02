# 03 - Reusing Data: Anchors, Aliases, and Spreads

One of MON's most powerful features is its ability to help you avoid repetition. You can define a piece of data once and
reuse it throughout your file. This is achieved through a three-part system: **Anchors**, **Aliases**, and **Spreads**.

### 1. Anchors (`&`): Creating a Reusable Template

An **Anchor** gives a nickname to a value, creating a reusable template.

* **Syntax:** You create an anchor by making it a key in an object, prefixed with an ampersand `&`.
* **Behavior:** The compiler finds all anchor declarations first (this is called "_hoisting_"), so you can define an
  anchor anywhere in the file and use it anywhere else.

```mon
{
    // This object is now a template named `default_user`
    &default_user: {
        theme: "dark",
        notifications: on,
        log_level: "info",
    },

    // This array is now a template named `base_permissions`
    &base_permissions: ["READ", "WRITE"],
}
```

### 2. Aliases (`*`): Making an Exact, Deep Copy

An **Alias** makes a complete and independent copy of an anchored value.

* **Syntax:** You create an alias by using an asterisk `*` followed by the anchor's name as the value in a key-value
  pair.
* **Behavior:** An alias performs a **deep copy**. It means that if the anchored value contains objects or arrays, the
  alias creates a brand new, fully independent copy of them. This prevents "spooky action at a distance," where changing
  a nested value in one copy accidentally changes it in another. It ensures that each alias is a completely safe,
  isolated duplicate.

**Example:**

```mon
{
    &default_user: { theme: "dark" },

    // alice_settings gets its own, independent copy of the object
    alice_settings: *default_user,

    // bob_settings also gets its own, independent copy
    bob_settings: *default_user,
}
```

**Canonical JSON Output:**

The final JSON representation of the file above would be:

```json
{
  "alice_settings": {
    "theme": "dark"
  },
  "bob_settings": {
    "theme": "dark"
  }
}
```

*(Note: The anchor definition `&default_user` is a template and does not appear in the final output.)*

### 3. The Spread Operator (`...*`): Merging and Extending

The **Spread Operator** is the most powerful composition tool. It unpacks the contents of an anchored value and merges
them into a new object or array.

#### Spreading Objects

* **Behavior:** When spreading an object, the operation is a **shallow merge**. The keys and values from the anchored
  object are copied into the new object. If a key exists in both the anchor and the new object, the **local key's value
  always wins**. This rule is simple, predictable, and allows for easy overriding of defaults.

**Example:**

```mon
{
    &default_user: {
        theme: "dark",
        log_level: "info",
    },

    admin_settings: {
        ...*default_user,      // 1. Merges in `theme` and `log_level`
        log_level: "verbose",  // 2. This local value for `log_level` wins
        is_admin: true,        // 3. A new key is added
    },
}
```

**Canonical JSON Output:**

```json
{
  "admin_settings": {
    "theme": "dark",
    "log_level": "verbose",
    "is_admin": true
  }
}
```

#### Spreading Arrays

* **Behavior:** When spreading an array, the operation is **concatenation**. The elements from the anchored array are
  inserted into the new array at the position of the spread operator. This provides a clear and intuitive way to build
  up lists from smaller pieces.

**Example:**

```mon
{
    &base_permissions: ["READ", "WRITE"],

    admin_permissions: [
        "LOGIN",              // 1. A local value
        ...*base_permissions, // 2. Inserts "READ" and "WRITE" here
        "DELETE",             // 3. Another local value
    ],
}
```

**Canonical JSON Output:**

```json
{
  "admin_permissions": [
    "LOGIN",
    "READ",
    "WRITE",
    "DELETE"
  ]
}
```

This detailed, step-by-step logic ensures that the behavior of anchors, aliases, and spreads is always predictable and
easy to understand.

---

### Challenge 3: Create a Config Template

Create a file `config.mon`. Inside it, define an anchor `&base_config` for an object with `host` and `port` keys. Then,
create two new objects, `dev_config` and `prod_config`. Use the spread operator to merge the `&base_config` into both,
but override the `host` for each to be different.

---

### Answer to Challenge 3

```mon
{
    &base_config: {
        host: "localhost",
        port: 8080,
    },

    dev_config: {
        ...*base_config,
        host: "dev.server.local",
    },

    prod_config: {
        ...*base_config,
        host: "api.my-app.com",
    },
}
```

Now you can efficiently reuse data. Next, we'll explore how to define custom data structures with the type system.

---

**Next up**: [04 - Defining Types: Structs and Enums](04_defining_types.md)