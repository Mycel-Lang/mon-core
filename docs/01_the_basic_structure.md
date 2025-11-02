# 01 - The Basic Structure: A Labeled Container

Every MON file is a single "labeled container" called an **Object**. This is the most fundamental rule. An object starts with a `{` and ends with a `}`. Inside this container, you organize your facts into `key: value` pairs.

### How to Write a `key: value` Pair

1.  **Key:** The label for your fact. For simple, one-word keys, you can write them directly. For keys with spaces or special characters, enclose them in double quotes `""`.
2.  **Colon (`:`):** Separates the key from the value.
3.  **Value:** The actual piece of information.
4.  **Comma (`,`):** Separates one pair from the next. A comma after the last pair is optional but recommended!

```mon
// A MON file must start with { and end with }
{
    // A simple key-value pair with a string value
    service_name: "My Awesome App",

    // You can add notes with comments like this
    port: 8080, // A number value

    "is-enabled": on, // A boolean value (yes/no)
}
```

### Types of Values (Primitives)

*   **String**: Text, enclosed in double quotes (e.g., `"Hello, World!"`).
*   **Number**: Numbers, with or without decimals (e.g., `100`, `-42.5`).
*   **Boolean**: A "yes/no" value. Use `on`/`true` for yes and `off`/`false` for no.
*   **Null**: Represents "nothing" or an empty value. Use `null`.

### Challenge 1: Create Your Profile

Create a file named `profile.mon`. Inside it, create a single object that contains at least three facts about yourself, such as your name, age, and a boolean indicating if you like to code.

---

### Answer to Challenge 1

```mon
{
    name: "Alex",
    age: 30,
    likes_coding: on,
}
```

Great! You've mastered the basic structure. Next, we'll learn how to manage lists of items.

---

**Next up**: [02 - Making Lists: Arrays](02_making_lists_arrays.md)
