# Pile
[Concatenative](https://en.wikipedia.org/wiki/Concatenative_programming_language), [stack-based](https://en.wikipedia.org/wiki/Stack-oriented_programming), [statically typed](https://en.wikipedia.org/wiki/Type_system#STATIC) and [compiled](https://en.wikipedia.org/wiki/Compiled_language) programming language for computers.


> **WARNING**: this is a project in VERY early stages of development. USE THIS LANGUAGE AT YOUR OWN RISK!
> Keep eyes on the next commits if you want to contribute.

## Development milestones
- [X] Make it (at least) an usable language
- [X] (maybe) JIT Compiler
- [ ] Implement all major operations
- [ ] Control flow
- [ ] Make it [turing complete](https://en.wikipedia.org/wiki/Turing_completeness)
- [ ] Statically typed

## Quick start
This language is not usable as a proper language yet, but you can experiment with some stuff:

Compile to executable and run the program `prog.pl` using clang(linux/osx):
```console
$ cd path/to/pile
$ python3 src/main.py --compile prog.pl
$ ./prog.out
```

Execute the code directly by the JIT compiler:
```console
$ cd path/to/pile
$ python3 src/main.py prog.pl
```

Get the LLVM representation of `prog.pl`:
```console
$ cd path/to/pile
$ python3 src/main.py prog.pl -e
```


---

Powered by [LLVM Compiler Infrastructure](https://llvm.org/) and [llvmlite](https://github.com/numba/llvmlite/). 

> By Marcio Dantas
