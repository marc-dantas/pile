# Pile's Development Goals and Ideas for the Language

## Already Done
- [X] Introduce more mathematical operations
  * Final: Operations added: Modulo and Exponentiation.
- [X] Introduce `bool` datatype
  * Final: Type `true`or `false` for the respective variants of the type.
           `if` statements only accept boolean values. 
- [X] Introduce `nil` datatype
  * Final: Use `nil` to represent empty or undefined values for error handling and logic flow.
- [X] Let statement
  * Final: Define global variables using keyword `let`:
    ```
    10 let x # assigns the name to the last value on top of the stack
    ```
- [X] Block Let statement
  * Final: Define local and scoped variables using the construction `as..let`:
    ```
    10 20 30
    as a b c let # assigns to the last 3 values on the stack
      ...
    end # variables are deleted here
    ```
- [X] Return in procedures
  * Auto-descriptive
- [X] More refined error messages
  * Final: errors now show lines
- [X] Import system
  * Final: You can import other program's namespaces into the main program:
    ```pile
    import "myprettyusefulprogram.pile"
    ...
    ```

## To Be Done
Definitely going to happen someday. Probably not exactly like described but it will happen.

- [ ] While loops
  * Auto-descriptive
- [ ] More builtins
  * Idea: Include type conversion and additional I/O builtins.
- [ ] String escaping
  * Idea: use the backslash syntax to allow escaped characters.
- [ ] String formatting
  * Idea: Introduce a `format` builtin to format strings dynamically:
    ```pile
    10 34 "this {} is {} formatted" format println # output: this 34 is 10 formatted
    ```
- [ ] Improve performance
    * Idea: Create a virtual machine for Pile.
- [ ] First stable release implementation

## Just ideas
> **NOTE**: Probably the majority of these ideas below aren't going to be actually implemented. **Nothing is confirmed**.

- [ ] Pattern matching?
  * Idea: Implement a `try` (or `match`?) mechanism for pattern matching mechanic to check `nil` values:
    ```
    "10" tonumber # could return nil if can't convert 
    try # Only runs this block if the last item on the stack is not nil
      ...
    end
    ```
- [ ] CLI Enhancements
  * Idea: Implement a REPL mode for testing code directly from the terminal.
- [ ] Code debugging
  * Idea: Introduce a debugging tool to help beginners track logical problems in the stack-based algorithm:
    ```console
    $ pile program.pile --debug
    $ # or
    $ pile debug program.pile # ?
    ```
- [ ] Testing features
  * Idea: Include `assert` statement and simple error handling:
    ```pile
    assert 2 3 + 5 = end
    ```
- [ ] For loop
  * Idea: Create a structure that iterates through the stack and assigns the value in iteration to a variable:
    ```
    1 2 3 4
    4 for x # each value is assigned to x for each iteration
        x println
    end
    # Output:
    # 4
    # 3
    # 2
    # 1
    ```
