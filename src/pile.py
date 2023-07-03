from llvmlite import ir, binding
from typing import Callable, TextIO, Generator, Tuple
from enum import Enum, auto
from dataclasses import dataclass

DEFAULT_INT: ir.IntType = ir.IntType(32)


class TokenKind(Enum):
    Int = auto()
    Symbol = auto()


@dataclass
class Token:
    value: str
    kind: TokenKind


class Lexer:
    source: TextIO

    def __init__(self, input_file: TextIO):
        self.source = input_file

    @property
    def index(self) -> int:
        return self.source.tell()

    def advance(self) -> int:
        current = self.source.read(1)
        while current and current.isspace():
            current = self.source.read(1)
        return self.index

    def tell(self) -> int:
        ptr = self.source.tell()
        x = self.source.seek(0, 2)
        self.source.seek(0, ptr)
        return x

    def lex(self) -> Generator[Token, None, None]:
        buflen = self.tell()
        while self.index < buflen:
            start = self.index
            end = self.advance()
            if self.index < buflen:
                end -= 1
            if start != end:
                self.source.seek(start)
                value = self.source.read(end - start)
                kind = self.classify_token(value)
                yield Token(value, kind)

    def classify_token(self, token: str) -> TokenKind:
        # TODO: We'll have more types, I promise.
        return TokenKind.Int if token.isdigit() else TokenKind.Symbol


class LLVMCompiler:
    
    builder: ir.IRBuilder
    module: ir.Module
    stack: list
    
    def __init__(self, _: object) -> None:
        binding.initialize()
        binding.initialize_native_target()
        binding.initialize_native_asmprinter()
        self.module = ir.Module(name="pile")
        self.module.triple = binding.get_default_triple()
        main = ir.Function(
            self.module,
            ir.FunctionType(ir.IntType(32), []),
            name="main"
        )
        self.builder = ir.IRBuilder(main.append_basic_block(name="entry"))
        self.stack = []
    
    # Op hardcoded functions
    
    def push(self, value):
        self.stack.append(self.builder.alloca(DEFAULT_INT))
        self.builder.store(ir.Constant(DEFAULT_INT, value), self.stack[-1])

    def binop(self, fn: Callable):
        b = self.builder.load(self.stack.pop())
        a = self.builder.load(self.stack.pop())
        result = fn(a, b)
        self.stack.append(self.builder.alloca(DEFAULT_INT))
        self.builder.store(result, self.stack[-1])
    
    def dump(self):
        printf = declare_libc_printf(self.module)
        result = self.builder.load(self.stack.pop())
        format_str = static_str(self.module, "%d\n")
        self.builder.call(printf, [self.builder.bitcast(format_str, ir.PointerType(ir.IntType(8))), result])

    def ret(self, code: int):
        self.builder.ret(ir.Constant(ir.IntType(32), code))


# useful functions...

def declare_libc_printf(module: ir.Module) -> ir.Function:
    printf = ir.FunctionType(ir.IntType(32), [ir.PointerType(ir.IntType(8))], var_arg=True)
    return ir.Function(module, printf, name="printf")


def static_str(module: ir.Module, value: str, cstr: bool = True) -> ir.GlobalVariable:
    str = f"{value}\0" if cstr else value
    format_const = ir.Constant(ir.ArrayType(ir.IntType(8), len(str)), bytearray(str.encode("utf8")))
    string = ir.GlobalVariable(module, format_const.type, name=f"{hex(id(value))}")
    string.linkage = "internal"
    string.global_constant = True
    string.initializer = format_const
    return string
