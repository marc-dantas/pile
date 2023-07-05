#!/usr/bin/python3
from pile import *
from argparse import *
from os.path import splitext
from os import remove
from ctypes import CFUNCTYPE
import subprocess


def parse_args() -> None:
    p = ArgumentParser("pile",
                       description="Concatenative, stack-based, statically"
                       " typed and compiled programming language for computers. ")
    p.add_argument("filename")
    p.add_argument(
        "-e", "--emit-llvm",
        action="store_true",
        help="prints the compiled LLVM representation of given file"
    )
    p.add_argument("-o", "--output", help="sets the output file to be written on.")
    p.add_argument(
        "-c", "--compile",
        help="compiles to an executable instead"
             "of running by the JIT compiler.",
        action="store_true")
    return p.parse_args()


def compile(path: str) -> ir.Module:
    with open(path, "r") as f:
        prog = Parser(Lexer(f).lex()).parse()
        return LLVMCompiler(prog).compile()


def err(msg: str) -> None:
    print(f"pile: {msg}", file=stderr)
    exit(1)


def compile_to_executable(filename: str, output: str) -> None:
    if output is None:
            output = f"{splitext(filename)[0]}"
    llvm_path = f"{splitext(output)[0]}.ll"
    with open(llvm_path, "w") as llvm_f:
        llvm_f.write(str(compile(filename)))
    subprocess.call(["clang", llvm_path, "-o", f"{splitext(llvm_path)[0]}.out"])
    remove(llvm_path)


def compile_mcjit(mod: ir.Module) -> Callable:
    target = binding.Target.from_default_triple()
    target = target.create_target_machine()
    module = binding.parse_assembly(str(mod))
    engine = binding.create_mcjit_compiler(module, target)
    engine.finalize_object()
    fnptr = engine.get_function_address("main")
    main = CFUNCTYPE(None)(fnptr)
    main()


def main() -> None:
    args = parse_args()
    if args.emit_llvm:
        if args.output is None:
            print(compile(args.filename))
        else:
            with open(args.output, "w") as f:
                f.write(str(compile(args.filename)))
    elif args.compile:
        compile_to_executable(args.filename, args.output)
    else:
        compile_mcjit(compile(args.filename))


if __name__ == "__main__":
    main()
