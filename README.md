<h1 align="center">pile</h1>
<p align="center">Educational stack-based and concatenative programming language.</p>

---

## Introduction to Pile
**Pile is an educational programming language designed to teach programming logic, stack-based concepts, and computer science fundamentals.**  
It provides an intuitive way to write stack-based algorithms, using **reverse Polish notation (RPN)**, where operands appear before the operation itself. Here are some RPN examples:

| **Infix notation (standard)** | **Reverse Polish notation** | **Evaluated result** |
| ----------------------------- | --------------------------- | -------------------- |
| `4 + 4`                       | `4 4 +`                     | `8`                  |
| `2 - 2 + 1`                   | `2 2 - 1 +`                 | `1`                  |
| `(6 + 1) * 2`                 | `6 1 + 2 *`                 | `14`                 |
| `6 + 1 * 2`                   | `2 1 * 6 +`                 | `8`                  |

Using RPN simplifies expression evaluation, eliminating the need for parentheses and operator precedence, which is ideal for stack-based algorithms.

## Getting Started

Pile is implemented in Rust as a CLI program that interprets Pile code.

### Using Pile

Clone the repository and build the project:
- **Windows**
    ```console
    > git clone https://github.com/marc-dantas/pile.git
    > cd .\pile\
    > cargo build
    > cargo run -- [your pile program]
    ```
- **Linux/UNIX**
    ```console
    $ git clone https://github.com/marc-dantas/pile.git
    $ cd ./pile/
    $ cargo build
    $ cargo run -- [your pile program]
    ```

For a quick reference, read [`basics.pile`](./basics.pile) file, which includes some examples and a compact overview of the language.

> **NOTE**: Full documentation is coming soon.

## Examples

1. **Hello World**
    ```
    # this is a comment
    "Hello World" dump
    ```
2. **Circle Area**
    ```
    def PI 3.14159265359 end

    proc circle_area
        dup * PI *
    end

    10 circle_area dump
    4 circle_area dump
    4.5 circle_area dump
    ```
3. **Count to Ten**
    ```
    0 loop
        dup dump
        dup 10 = if stop end
        1 +
    end
    ```
4. **Fibonacci sequence**
    ```
    proc fib
        0 1
        loop
            dup N >= if
                dup dump
                over over +
            else stop end
        end
    end

    def N 400 end
    fib
    ```

For additional examples, explore the [`./examples`](./examples) folder.

---

> Licensed under **GPL 3.0**. See [`LICENSE`](./LICENSE) for details.

> Developed by [Marcio Dantas](https://github.com/marc-dantas)
