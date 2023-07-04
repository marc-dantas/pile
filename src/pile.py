from llvmlite import ir, binding
from typing import Callable, TextIO, Generator, NewType
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

    def __init__(self, source: TextIO):
        self.source = source

    @property
    def index(self) -> int:
        return self.source.tell()

    def advance(self) -> int:
        current = self.source.read(1)
        while current and not current.isspace():
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


class NodeKind(Enum):
    Symbol = auto()
    OpPush = auto()
    OpPlus = auto()
    OpDump = auto()


@dataclass
class Node:
    token: Token
    kind: NodeKind


Program = NewType("Program", Generator[Node, None, None])


class Parser:
    
    tokens: Generator[Token, None, None]
    
    def __init__(self, tokens: Generator[Token, None, None]) -> None:
        self.tokens = tokens
    
    def match_kind(self, token: Token) -> NodeKind:
        if token.kind == TokenKind.Symbol:
            table = {
                "+": NodeKind.OpPlus,
                "dump": NodeKind.OpDump,
            }
            return (NodeKind.Symbol if token.value not in table
                    else table[token.value])
        elif token.kind == TokenKind.Int:
            return NodeKind.OpPush
        else:
            assert False, "Unreachable at Parser.match_kind"
    
    def parse(self) -> Program:
        # TODO: Make the parser recognize Symbols in wrong places.
        # TODO: Make the parser be able to parse code blocks (if, while, etc.).
        for token in self.tokens:
            yield Node(token, self.match_kind(token))


class LLVMCompiler:
    
    builder: ir.IRBuilder
    module: ir.Module
    stack: list
    prog: Program
    
    def __init__(self, program: Program) -> None:
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
        self.prog = program
    
    def compile(self) -> ir.Module:
        for node in self.prog:
            if node.kind == NodeKind.OpPush:
                self.push(int(node.token.value))
            elif node.kind == NodeKind.OpPlus:
                self.binop(self.builder.add)
            elif node.kind == NodeKind.OpDump:
                self.dump()
            elif node.kind == NodeKind.Symbol:
                raise NotImplementedError(f"{node.token.value}")
        self.ret(0)
        return self.module
    
    # Op hardcoded functions
    
    def push(self, value: int) -> None:
        self.stack.append(self.builder.alloca(DEFAULT_INT))
        self.builder.store(ir.Constant(DEFAULT_INT, value), self.stack[-1])

    def binop(self, fn: Callable) -> None:
        b = self.builder.load(self.stack.pop())
        a = self.builder.load(self.stack.pop())
        result = fn(a, b)
        self.stack.append(self.builder.alloca(DEFAULT_INT))
        self.builder.store(result, self.stack[-1])
    
    def dump(self) -> None:
        printf = declare_libc_printf(self.module)
        result = self.builder.load(self.stack.pop())
        format_str = static_str(self.module, "%d\n")
        self.builder.call(printf, [self.builder.bitcast(format_str, ir.PointerType(ir.IntType(8))), result])

    def ret(self, code: int) -> None:
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
