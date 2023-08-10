# Pile
[Concatenative](https://en.wikipedia.org/wiki/Concatenative_programming_language), [stack-based](https://en.wikipedia.org/wiki/Stack-oriented_programming), [statically typed](https://en.wikipedia.org/wiki/Type_system#STATIC) and [compiled](https://en.wikipedia.org/wiki/Compiled_language) programming language for computers.

![Python](https://img.shields.io/badge/Python-3.8+-3670A0?style=plastic&logo=python&logoColor=white)
![PyPy](https://img.shields.io/badge/PyPy-3.8+-3670A0?style=plastic&logo=pypy&logoColor=white)


> NOTE: This language is not designed to be used in production.
> This project has educational purposes to teach stack-based architectures to students.

---

## Development milestones

- [X] Make it (at least) a usable language
- [X] JIT Compiler
- [X] Implement all major operations
- [X] Control flow
- [X] Strings
- [X] Statically typed (implement a proper type checker)
- [X] Documentation
- [ ] Make it [turing complete](https://en.wikipedia.org/wiki/Turing_completeness)

## Quick start
> **WARNING**: This is a project in VERY early stages of development. USE THIS LANGUAGE AT YOUR OWN RISK!
> Keep eyes on the next commits if you want to contribute.
> **DISCLAIMER**: On Windows systems, this project is more limited. I recommend using Unix/Linux!

#### Dependencies

To use this software, you need to install the dependencies by doing:
```console
$ cd path/to/pile
$ python3 -m pip install -r requirements.txt
```

#### How to use

This language is not a proper language yet, but you already do some stuff with it:

Execute the code directly by the JIT compiler:
```console
$ # To run main.py this way, do `chmod +x src/main.py`
$ src/main.py examples/hello.pl
```

Compile to executable and run the program `prog.pl` (using clang):
```console
$ python3 src/main.py examples/hello.pl -c
... clang stuff ...
$ ./examples/hello
```


Get the LLVM representation of `prog.pl`:
```console
$ python3 src/main.py examples/hello.pl -e
```

# Documentation
You can read the official documentation [here](./docs/index.md)

---

Powered by [LLVM Compiler Infrastructure](https://llvm.org/) and [llvmlite](https://github.com/numba/llvmlite/). 

> By Marcio Dantas
