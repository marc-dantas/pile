# Pile's builtins

Builtins are more complex operations that come with the interpreter. 
They are often system-related or specific operations for certain values. 

Pile's builtins are simply words what work just like a normal operation, refer to it in the program and you will execute it.

## Builtins

> In this list, it will be used the Forth stack notation to describe the stack state needed to perform the operation being documented. Please consult this notation at [forth-standard.org](https://forth-standard.org/standard/notation) at section **2.2.2 Stack notation**.

### System

| Builtin    | Forth stack notation | Description                                                              |
|------------|----------------------|--------------------------------------------------------------------------|
| `exit`     | `( c -- )`           | Pops an integer code C and exits the program with code C.                |
| `read`     | `( f -- xs )`        | Pops a file object F from the stack and read until the end.              |
| `readline` | `( f -- xs )`        | Pops a file object F from the stack and reads one line.                  |
| `write`    | `( f xs -- )`        | Pops a file object F and content XS from the stack and writes XS into F. |
| `open`     | `( p m -- f )`       | Pops a string path P and a mode M from the stack and opens the file F.   |

## Language
| Builtin    | Forth stack notation | Description                                                                 |
|------------|----------------------|-----------------------------------------------------------------------------|
| `len`      | `( s -- #s )`        | Pops a string or array S from the stack and pushes its length.              |
| `ord`      | `( s -- c )`         | Pops a string S from the stack and pushes the corresponding character code. |
| `chr`      | `( c -- s )`         | Pops a character code C from the stack and pushes the corresponding string. |

## Typing
| Builtin    | Forth stack notation | Description                                                                        |
|------------|----------------------|------------------------------------------------------------------------------------|
| `typeof`   | `( a -- t )`         | Pops any value A from the stack and pushes its type name as a string.              |
| `toint`    | `( a -- b )`         | Pops a value A from the stack and pushes the corresponding convertion to `int`.    |
| `tofloat`  | `( a -- b )`         | Pops a value A from the stack and pushes the corresponding convertion to `float`.  |
| `tostring` | `( a -- b )`         | Pops a value A from the stack and pushes the corresponding convertion to `string`. |
| `tobool`   | `( a -- b )`         | Pops a value A from the stack and pushes the corresponding convertion to `bool`.   |

---

> [**Next**](./controlflow.md)
