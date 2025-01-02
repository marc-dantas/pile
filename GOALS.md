# Pile's Development Goals and Ideas for the Language

## Already Done
- [X] Introduce more mathematical operations
  * Final: Operations added: Modulo and Exponentiation.

## To Be Done
Definitely going to happen someday. Probably not exactly like described but it will happen.

- [ ] Introduce `nil` datatype
  * Idea: Use `nil` to represent empty or undefined values for error handling and logic flow.
- [ ] Introduce `bool` datatype
  * Idea: Make `if` statements only accept boolean values: `true` or `false`. 
- [ ] More builtins
  * Idea: Include type conversion and additional I/O builtins.
- [ ] String formatting
  * Idea: Introduce a `format` builtin to format strings dynamically:
    ```pile
    10 34 "this {} is {} formatted" format println # output: this 34 is 10 formatted
    ```
- [ ] Import system
  * Idea: Enable importing other `.pile` files into the main program:
    ```pile
    import "test.pile" end
    ...
    ```
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
  * Idea 2: Use `?` operator to check if something is not nil and use if directly:
    ```!?
    "10" tonumber # could return nil if can't convert 
    ? if # Checking if it is nil

    end
    ```
- [ ] Error stack trace
  * Idea: Make multiple errors appear at the same execution shot.
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
