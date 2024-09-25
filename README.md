<h1 align="center">pile</h1>
<p align="center">Educational stack-based and concatenative computer programming language.</p>

## Introduction to pile (please read if you are new)
**Pile is an educational programming language designed to teach programming logic, stack-based concepts and logic for computer science students and software developers**.
Pile allows users to write stack-based algorithms in an intuitive way thanks to reverse Polish notation.

The language is based on stack-based data structures and **reverse Polish notation** (a.k.a. Infix notation).
Reverse Polish notation is a way to write mathematical and algorithmic operations in a way that the operands are expressed **before** the operation itself. Here are some examples of mathematical expressions:

| **Infix notation (normal)** | **Reverse Polish notation** |
| --------------------------- | --------------------------- |
| `4 + 4`                     | `4 4 +`                     |
| `2 - 2 + 1`                 | `2 2 - 1 +`                 |
| `(6 + 1) * 2`               | `6 1 + 2 *`                 |
| `6 + 1 * 2`                 | `2 1 * 6 +`                 |

Pile uses this notation as it's syntax base, since it's way more intuitive to express stack-based algorithms.
RPN also simplifies expression evaluation, eliminating the need for parentheses and operator precedence.

## Getting started
Pile doesn't really exit yet, I just wrote some random code and called it a day.
But I have some nice ideas!

You can read the file [`basics.pile`](./basics.pile) in the root of this repository to take a look at some ideas that I have for this language.

> **NOTE**: Documentation isn't done yet.

For now I don't recommend you to run any code that's in this repository.
For now I'm just keeping the code here for history and documentation purposes.

---

> Licensed under **GPL 3.0**, see [`LICENCE`](./LICENSE) for more information.

> By [Marcio Dantas](https://github.com/marc-dantas)