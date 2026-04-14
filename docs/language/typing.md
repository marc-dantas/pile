# Pile's typing system

Pile is a strong, dynamically-typed programming language. That means the types are more present than in a weak-typing language (like Lua) but not as strict as in statically-typed programming language (like C).

Operations done to values that don't have a valid type will result in a Runtime error.

Example:
```
1 "1" + // error
5 1.7 - // error
```

## Datatypes
In Pile, the datatypes are:
| Type     | Description                                                                       |
|----------|-----------------------------------------------------------------------------------|
| `string` | String, corresponds to the quotes (e.g. `"hello"`)                          |
| `int`    | Integer number                                                                    |
| `float`  | Floating-point number                                                             |
| `array`  | List of values of arbitrary types                                                 |
| `data`   | Arbitrary datatype in which its information comes from outside Pile's interpreter |
| `nil`    | Null type expressed with the keyword `nil`                                        |

The creation of compound data types (struct-like or objects) is still in development.

---

> [**Next**](./operations.md)
