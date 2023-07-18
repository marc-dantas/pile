# Pile Documentation

## Introduction
**This project is designed to be a tool made *only* to teach the concepts of stack-based architectures and reverse polish notation**.

Pile is a concatenative, stack-based and compiled programming language made for educational purposes. It compiles to LLVM Intermediate Representation to be executed. It also features a JIT compiler.

Pile's compiler is written in the Python programming language and it uses the library `llvmlite` for the compilation step.

## How Pile works

Pile's compiler works on a simutated stack to store the values that are manipulated inside it.

> NOTE: Pile works like that because the LLVM IR doesn't use such stack-based design to work. So, the compiler simulates the stack at compile-time.

## Familiarizing with Pile

Here's a simple "hello world" program in Pile:

- file: `examples/hello.pl`
    ```c
    // simple comment ...
    "hello world" dump
    ```

Because it's a stack-based language, Pile uses the Reverse Polish Notation to represent stack operations. So, a simple mathematical operation like `1 + 1` turns into `1 1 +`:

- file: `examples/math.pl`
    ```c
    // math operations
    1 1 + dump // Sums 1 + 1
    3 1 - dump // Subtracts 3 by 1
    2 2 * dump // Multiplies 2 by 2
    1.0 2.0 / dump // Divides 1 by 2 (half)
    ```

## Pile's typing system
Pile is also a statically typed language and it has 4 types:

- `integer`: Represents a signed 32-bit integer value.
- `bool`: Represents a 1-bit integer value (1 or 0).
- `float`: A floating point value.
- `string`: Character pointer (`i8*` or `char*`)

Each operation has it's own typing, and if you disrespect them, you'll get an error:

- file: `examples/type_mismatch.pl`
    ```c
    // type mismatch error!
    69 3.1415 / dump
    //        ^
    // error here
    ```
- stderr:
    ```
    pile: error at examples/type_mismatch.pl:2:10:
      | type mismatch:
      |    `/` operation got mismatched types (float,
      |    integer) but operation expects (float, float)
    ```

All operations in Pile are type-checked during compile-time.

## Operations

Operations in pile are basically every word in a Pile program. Except literals and some control flow words.
> Words are every token in a Pile program

Pile has a very small list of built-in operations:

- `+`: mathematical addition operation
- `-`: mathematical subtraction operation
- `*`: mathematical multiplication operation
- `/`: mathematical division operation
- `>`: "greater than" comparison operation
- `<`: "less than" comparison operation
- `>=`: "greater than or equal" comparison operation
- `<=`: "less than or equal" comparison operation
- `!=`: "not equal" comparison operation
- `=`: "equal" comparison operation
- `|`: bitwise OR operation
- `&`: bitwise AND operation
- `>>`: bitwise shift right operation
- `<<`: bitwise shift left operation  
- `drop`: drops a value from the top of the stack (`a -- `)
- `dup`: duplicates the last element to the top of the stack (`a -- a a`) 
- `over`: copies the 2nd last element on top of the stack (`a b -- a b a`) 
- `rot`: moves the 3rd last element to the top of the stack (`a b c -- b c a`) 
- `swap`: swaps the top two values on top of the stack (`a b -- b a`)
- `dump`: prints the last element (of any type) on top of the stack to stdout

All these operations are demonstrated in `examples/ops.pl`.

# Control flow

In Pile you can control the flow of your program using two statements:

### If conditions

If conditions are very simple in Pile, here's it's syntax:
```lua
<condition> if
    ...
[else
    ...]
end
```

and Here's a simple example of how to use it:
- file: `examples/if.pl`
    ```lua
    10 10 = if
        1 dump
        1 1 = if
            3 dump
        else
            4 dump
        end
    else
        2 dump
    end
    ```

### While loops

While loops are a little bit different. Here's it's syntax:
```lua
while <condition> do
    ...
end
```

and Here's a simple example of how to use it:
- file: `examples/while.pl`
    ```lua
    0 while dup 10 <= do
        dup dump
        1 +
    end drop
    ```
