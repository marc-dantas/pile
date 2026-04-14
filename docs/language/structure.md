# Pile's lexical and syntactical structure

This is not a specification, just a context to understand how Pile's interpreter processes the source-code.

## Tokenization
Pile's interpreter first processing step is Tokenization, groups the sequence of characters in the source code that mean a **Token**, basic lexical unit of a language.

In Pile, there are 4 types of **tokens**:
- Word
- Integer
- Floating-point
- String

Pretty auto-descriptive.

## Syntax
Pile's parser is extremely simple. A recursive descent parser that generates the AST (Abstract Syntax Tree) for the compiler to generate the sequence of virtual machine instructions corresponding to the program.

All structures in Pile are expressions. Most statements are called statements for the mere fact of having a specific syntax to be followed, like Conditionals, Loops and Arrays.

The only declarative (non-expression) statements are the ones that declare something: Procedures, Definitions and Variables.

---

> [**Next**](./typing.md)
