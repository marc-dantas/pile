# Stack-Oriented Programming

[Stack-oriented programming](https://en.wikipedia.org/wiki/Stack-oriented_programming) is a paradigm built around a [stack](https://en.wikipedia.org/wiki/Stack_(abstract_data_type)) as the primary means of data passing between operations. Values are pushed onto the stack; operators pop their arguments and push results back. Programs are written in [Reverse Polish Notation](https://en.wikipedia.org/wiki/Reverse_Polish_notation), making execution order explicit and eliminating the need for parentheses or precedence rules.

This model shows up more often than people expect:

- **[RPN Calculators](https://en.wikipedia.org/wiki/HP_calculators)** — HP's calculator line (HP-35, HP-48) exposed the stack directly to the user.
- **[Forth](https://en.wikipedia.org/wiki/Forth_(programming_language))** — the archetypal stack-based language, still used in embedded systems and firmware.
- **[PostScript](https://en.wikipedia.org/wiki/PostScript)** — Adobe's page description language, which every PostScript printer interprets. PDF inherits much of the same model.
- **[JVM](https://en.wikipedia.org/wiki/Java_virtual_machine)** — Java bytecode is a stack machine. Every Java, Kotlin, and Scala program runs on one.
- **[WebAssembly](https://en.wikipedia.org/wiki/WebAssembly)** — formally specified as a stack machine.
- **[Factor](https://en.wikipedia.org/wiki/Factor_(programming_language))** — a modern concatenative language that extends the Forth model with a type system and functional idioms.

The rest of this documentation references Pile programming language, a stack-based scripting programming language.

> [**Next**](./structure.md)
