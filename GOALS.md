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

## To Be Done
Definitely going to happen someday. Probably not exactly like described but it will happen.

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
- [ ] More refined error messages
  * Idea: Make language errors show the line and column visually to the use
*   Also add useful information like error link to documentation.
    
    An error message that is like this:
    ```
    pile: error at .\test.pile:1:6:
        |    parse error:
        |        syntax error: unexpected token while parsing:
        |        expected valid identifier but got 123
    ```
    could be like this:
    ```
    pile: parse error at .\test.pile:1:6:
    syntax error: unexpected token while parsing:
    
     1  |    proc 123
                  ^
                  expected valid identifier but got 123                 
    ```
    Or any variation of this
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
- [ ] Block Let statement
  * Idea: Create a scope system that works inside a block syntax after the let.
    ```
    10 20 30
    let a b c do # assigns to the last 3 values on the stack
      ....
    end # variables are deleted here
    ```
