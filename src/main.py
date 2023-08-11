#!/usr/bin/python3
from argparse import *
from ctypes import CFUNCTYPE, c_int
from os import remove
from sys import platform
from os.path import splitext, exists
from pile import *
import subprocess

# VERSION = (0, 0, 0)
# STR_VERSION = '.'.join(str(i) for i in VERSION)


def parse_args() -> Namespace:
    p = ArgumentParser("pile",
                       description="Pile Programming Language",
                       epilog="Copyright © 2023 Marcio Dantas. "
                       "This software is under MIT License",
                       usage="pile [OPTIONS] filename")
    p.add_argument("filename")
    p.add_argument(
        "-e", "--emit-llvm",
        action="store_true",
        help="prints the compiled LLVM representation of given file"
    )
    p.add_argument("-o", "--output", help="sets the output file to be written on.")
    p.add_argument(
        "-c", "--compile",
        help="compiles to an executable (using clang) instead "
        "of running by the JIT compiler.",
        action="store_true"
    )
    return p.parse_args()


def pile2llvm(path: str) -> ir.Module:
    prog = parse(lex_file(path))
    return compile_program(prog)


def dump_tokens(path: str) -> None:
    for i in lex_file(path):
        print(i)


def err(msg: str) -> None:
    print(f"pile: error: {msg}", file=stderr)
    exit(1)


def compile_to_executable(filename: str, output: str) -> None:
    if output is None:
        output = filename
    llvm_path = f"{splitext(output)[0]}.ll"
    with open(llvm_path, "w") as llvm_f:
        llvm_f.write(str(pile2llvm(filename)))
    subprocess.call(["clang", llvm_path, "-o", splitext(output)[0]])
    remove(llvm_path)


def compile_mcjit(mod: ir.Module) -> binding.ExecutionEngine:
    target = binding.Target.from_default_triple()
    target = target.create_target_machine()
    module = binding.parse_assembly(str(mod))
    engine = binding.create_mcjit_compiler(module, target)
    engine.finalize_object()
    return engine


def main() -> None:
    args = parse_args()
    if not exists(args.filename):
        err(f"no such file \"{args.filename}\"")
    
    if args.emit_llvm:
        if args.output is None:
            print(pile2llvm(args.filename))
        else:
            with open(args.output, "w") as f:
                f.write(str(pile2llvm(args.filename)))
    elif args.compile:
        if platform == "win32":
            err("the ability to compile a program to an executable is not supported on Windows")
        compile_to_executable(args.filename, args.output)
    else:
        engine = compile_mcjit(pile2llvm(args.filename))
        main_addr = engine.get_function_address("main")
        CFUNCTYPE(c_int)(main_addr)()


if __name__ == "__main__":
    main()
