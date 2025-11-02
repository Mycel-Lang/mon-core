# 02 - Making Lists: Arrays

An **Array** is an ordered list of items. You create an array using square brackets `[]`, with each item in the list separated by a comma.

Arrays are perfect for collections of things, like a list of tags, a group of users, or a sequence of steps.

### Simple Lists

You can create a simple list of primitive values like strings or numbers.

```mon
{
    // A list of strings
    tags: ["tutorial", "mon", "beginner"],

    // A list of numbers
    winning_numbers: [12, 45, 77, 81],
}
```

### Lists of Objects

More powerfully, you can create a list where each item is an object. This is a fundamental pattern for structuring complex data.

```mon
{
    // A list of users, where each user is an object
    users: [
        {
            name: "Alice",
            role: "admin",
        },
        {
            name: "Bob",
            role: "viewer",
        },
    ],
}
```

### Challenge 2: Your Favorite Things

Create a file named `favorites.mon`. Inside the root object, add a key named `favorite_movies`. The value should be an array where each item is an object representing a movie. Each movie object should have a `title` (String) and a `release_year` (Number).

---

### Answer to Challenge 2

```mon
{
    favorite_movies: [
        {
            title: "The Matrix",
            release_year: 1999,
        },
        {
            title: "Inception",
            release_year: 2010,
        },
    ],
}
```

Excellent! You can now structure both single items and lists of items. Next, we'll learn how to stop repeating ourselves by reusing data within a file.

---

**Next up**: [03 - Reusing Data: Anchors and Spreads](03_reusing_data.md)
