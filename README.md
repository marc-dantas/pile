<p align="center">
    <img width="300" src="./logo/readme_logo.png" alt="pile"></img>
</p>
<h3 align="center">Educational stack-based and concatenative programming language.</h3>

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
> **WARNING**: This language is not finished yet, there's no warranty of this software in any way. **Use it at your own risk**!.

Pile is implemented in Rust as a CLI program that interprets Pile code.

### Using Pile

Clone the repository and build the project:
- **Windows**
    ```console
    > git clone https://github.com/marc-dantas/pile.git
    > cd .\pile\
    > cargo build --release
    > .\target\release\pile.exe [your pile program]
    ```
- **Linux/UNIX**
    ```console
    $ git clone https://github.com/marc-dantas/pile.git
    $ cd ./pile/
    $ cargo build --release
    $ ./target/release/pile [your pile program]
    ```

## Documentation

***(Still in development)***

Pile's full documentation and website is being developed at [marc-dantas/pile-online](https://github.com/marc-dantas/pile-online).

For a quick understanding of the language, try reading [`BASICS.md`](./BASICS.md) file, which includes some examples and a compact overview of the language.

## Examples

1. **Hello World**
    ```
    # this is a comment
    "Hello World" println
    ```
2. **Circle Area**
    ```
    def PI 3.14159265359 end

    proc circle_area
        dup * PI *
    end

    10 circle_area println
    4 circle_area println
    4.5 circle_area println
    ```
3. **Count to Ten**
    ```
    0 loop
        dup println
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
                dup println
                over over +
            else stop end
        end
    end

    def N 400 end
    fib
    ```
5. **Ask my name**
   ```
   "What is your name? " print
   readln
   "Your name is " print print "." println
   ```

For additional examples, explore the [`./examples`](./examples) folder.

## Goals and Ideas
You can read the [`GOALS.md`](./GOALS.md) file to find out what I want to implement in the language in the future and some ideas.

---

> Licensed under **GPL 3.0**. See [`LICENSE`](./LICENSE) for details.

> Developed by [Marcio Dantas](https://github.com/marc-dantas)
