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

Pile is implemented in Rust programming language, for now it's just a simple CLI program that interprets Pile code.

### Using pile
You can run the program using `cargo` following the steps below:

- Windows:
    ```console
    > git clone https://github.com/marc-dantas/pile.git
    > cd .\pile\
    > cargo build
    > cargo run -- [your pile program]
    ```
- Linux/UNIX
    ```console
    $ git clone https://github.com/marc-dantas/pile.git
    $ cd ./pile/
    $ cargo build
    $ cargo run -- [your pile program]
    ```

You can read the file [`basics.pile`](./basics.pile) in the root of this repository to take a look at a compact "reference" file of everything you can do in this language.

> **NOTE**: Documentation isn't done yet.

## Examples

1. **Hello World**
    ```
    # this is a comment
    "Hello World" print
    ```
2. **Circle Area**
    ```
    def pi 3.14159265359 end

    proc circle_area
        dup * pi *
    end

    10 circle_area print
    4 circle_area print
    4.5 circle_area print
    ```
3. **Count to ten** 
    ```
    0 loop
        dup print
        dup 10 = if stop end
        1 +
    end
    ```

You can find more examples in the [`./examples`](./examples) folder at the root of the repository.

---

> Licensed under **GPL 3.0**, see [`LICENCE`](./LICENSE) for more information.

> By [Marcio Dantas](https://github.com/marc-dantas)