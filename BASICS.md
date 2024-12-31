# Pile Basics

**Welcome to Pile programming language**

This file is meant to be read by more experienced programmers.

If you aren't into stack-based programming or any concept like that or you're just new to all this,
it's recommended to read the actual documentation of Pile.

> **NOTE**: This file is not meant to be any kind of reference or documentation. For now, this file is just a simple way of understanding the basics of the language in a few minutes without further explanation.

> **WARNING**: The content of this file is not revised. You can find outdated, unimplemented content or grammar mistakes. This file is provisory.   

## Introduction to Pile

Pile relies on a stack-based architecture to manipulate data and create algorithms.

It also uses the concept of LIFO (Last-in, First-out) that may confuse the beginners at the start because of the reversed order of operands when working with some kinds of operations. More of this will be mentioned.

A stack in a stack-oriented paradigm is essentially an "infinite" array where you can push, duplicate, or remove items.

### Rules:

- A Pile program operates on **one global stack**.
- This stack cannot be divided or replaced.
- All operations interact with this same stack.


## Simple operations

### Basic Operations:

```pile
1 1 +        # Adds 1 and 1 (Result: 2)
1 1 + 1 -    # Adds 1 and 1, then subtracts 1 (Result: 0)
2 10 / 1 +   # Divides 10 by 2, then adds 1 (Result: 6)
4 4 * 2 -    # Multiplies 4 by 4, then subtracts 2 (Result: 14)
```

### Comparisons:

```pile
10 10 =    # Checks if 10 equals 10 (true)
10 11 !=   # Checks if 11 is not equal to 10 (true)
10 12 <=   # Checks if 12 is less than or equal to 10 (false)
10 12 >=   # Checks if 12 is greater than or equal to 10 (true)
10 12 >    # Checks if 12 is greater than 10 (true)
10 12 <    # Checks if 12 is less than 10 (false)
```

### Bitwise Operations:

```pile
1     !     # Bitwise NOT (Result: 0)
1 0   |     # Bitwise OR (Result: 1)
1 0   &     # Bitwise AND (Result: 0)
1 0   >>    # Bitwise SHIFT LEFT (Result: 0)
1 0   <<    # Bitwise SHIFT RIGHT (Result: 0)
```

## Stack Manipulation

### Operations:

```pile
420     drop  # Deletes (drops) the last item on the stack
45      dup   # Duplicates (copies) the last item on the stack
45 5    swap  # Swaps the last pair of items on the stack (45 5 to 5 45)
45 5    over  # Copies the second last item and pushes it onto the stack (45 5 to 45 5 45)
45 5 12 rot   # Copies the third last item and pushes it onto the stack (45 5 12 to 5 12 45)
```

### Literals

A literal value is any value that you can write (hardcode) into your program. Pile has (for now, it will be updated soon) 2 datatypes:
- `string`: Array of unicode characters. Syntax: `"[your text]"`
- `number`: Base-10 number. Can be any combination of any length of the decimal digits. 

- Examples:
  - Numbers: `5`, `7`, `10`, `120`
  - Strings: `"hello world"`, `"foo bar baz"`

**Any** literal value written in Pile is **always** interpreted as a "push" operation onto the stack.

## Control Flow

### If Expressions:

```pile
10 10 = if
    "true case" dump
else  # Optional
    "false case" dump
end
```

### Loop Expressions:

```pile
loop
    "this will dump forever" dump
    # Use `stop` to break the loop
end
```

## Procedures

### Overview

- A procedure in Pile is a reusable block of code that is executed when it's called.
- In Pile, procedures do not have arguments or return values.
- The stack is used to pass and store data simultaneously.

### Examples

```pile
proc dump_hello
    "hello" dump
end
dump_hello # Output: hello
```

```
proc add_1
    1 +  # Assumes a value is already on the stack
end
add_1      # Error!
1 add_1    # Output: 2
```

## Definitions

### Overview

- A definition in Pile is a constant value computed at the time of its creation bound to a name.
- A definition pushes its value onto the stack when used.

### Examples

```pile
def SUN 1 end
def MON 1 end
def TUE 1 end
def WED 1 end
def THU 1 end
def FRI 1 end
def SAT 1 end
def TOTAL
    SUN
    MON +
    TUE +
    WED +
    THU +
    FRI +
    SAT +
end
TOTAL dump
```

```
def AUTHOR "marc-dantas" end
AUTHOR dump # Output: marc-dantas
```


**More about Pile programming language can be found in the official documentation.**

---

> Licensed under [GPL-3.0](./LICENSE)

> By Marcio Dantas