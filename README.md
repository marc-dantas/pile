# Pile
[Concatenative](https://en.wikipedia.org/wiki/Concatenative_programming_language), [stack-based](https://en.wikipedia.org/wiki/Stack-oriented_programming), [statically typed](https://en.wikipedia.org/wiki/Type_system#STATIC) and [compiled](https://en.wikipedia.org/wiki/Compiled_language) programming language for computers.


> **WARNING**: this is a project in VERY early stages of development. USE THIS LANGUAGE AT YOUR OWN RISK!
> It isn't even usable yet. Keep eyes on the next commits if you want to contribute.

## Development milestones
- [ ] Make it (at least) an usable language
- [ ] Implement all major operations
- [ ] Control flow
- [ ] Make it [turing complete](https://en.wikipedia.org/wiki/Turing_completeness)
- [ ] Statically typed

## Quick start
This language is not usable as a language yet, but you can experiment with some stuff:

Compile and run the hardcoded program at `src/main.py`:
```console
$ cd path/to/pile
$ python3 src/main.py > output.ll
$ clang output.ll -o output
$ ./output
```

If you want to **see** the LLVM IR output:
```console
$ cd path/to/pile
$ python3 src/main.py
```

---

Powered by [LLVM Compiler Infrastructure](https://llvm.org/) and [llvmlite](https://github.com/numba/llvmlite/). 

> By Marcio Dantas
