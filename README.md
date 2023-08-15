```
 _____ _ _      
|  __ (_) |     
| |__) || | ___ 
|  ___/ | |/ _ \
| |   | | |  __/
|_|   |_|_|\___|
```

Pile is an [Esoteric](https://en.wikipedia.org/wiki/Esoteric_programming_language), [Concatenative](https://en.wikipedia.org/wiki/Concatenative_programming_language), [stack-based](https://en.wikipedia.org/wiki/Stack-oriented_programming), [statically typed](https://en.wikipedia.org/wiki/Type_system#STATIC) and [compiled](https://en.wikipedia.org/wiki/Compiled_language) programming language designed **only** for educational purposes.

---

## Quick start

### Dependencies

To use this software, you need to install the dependencies by running:
(Python version 3.8 or higher is required, get it [here](https://www.python.org/downloads/))

```console
$ cd path/to/pile
$ python3 -m pip install -r requirements.txt
```

### Usage

#### Execute the hello world program

- Unix/Linux
    ```console
    $ chmod +x src/main.py
    $ src/main.py examples/hello.pl
    ```
- Windows
    ```console
    > python .\src\main.py .\examples\hello.pl
    ```

#### Compile to executable and run (only on Unix/Linux systems)
```console
$ python3 src/main.py examples/hello.pl -c
$ ./examples/hello
```

#### Show all tokens of the program
- Unix/Linux
    ```console
    $ python3 src/main.py examples/hello.pl -t
    ```
- Windows
    ```console
    > python .\src\main.py .\examples\hello.pl -t
    ```
  

#### Get the LLVM Intermediate Representation
- Unix/Linux
    ```console
    $ python3 src/main.py examples/hello.pl -e
    ```
- Windows
    ```console
    > python .\src\main.py .\examples\hello.pl -e
    ```

# Docs
You can read the official wiki page [here](https://esolangs.org/wiki/Pile) and learn more about how to use Pile and how it works.

---

Powered by [LLVM Compiler Infrastructure](https://llvm.org/) and [llvmlite](https://github.com/numba/llvmlite/). 

> By Marcio Dantas
