from llvmlite import ir, binding
from typing import Callable, TextIO, Generator
from typing import NewType, Dict, List
from enum import Enum, auto
from dataclasses import dataclass
from sys import stderr

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
    OpPush = auto()
    OpPlus = auto()
    OpMinus = auto()
    OpMul = auto()
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
            match_table = {
                "+": NodeKind.OpPlus,
                "-": NodeKind.OpMinus,
                "*": NodeKind.OpMul,
                "dump": NodeKind.OpDump,
            }
            if token.value in match_table:
                return match_table[token.value]
            
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
    functions: Dict[str, ir.Function]
    consts: Dict[str, ir.GlobalVariable]
    
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
        self.functions = {
            "printf": ir.Function(
                self.module,
                ir.FunctionType(ir.IntType(32), [ir.PointerType(ir.IntType(8))], var_arg=True),
                name="printf"
            ),
        }
        self.consts = {
            "digit_fmt": global_str(self.module, "%d\n", "digit_fmt")
        }
    
    def compile(self) -> ir.Module:
        errors = []
        for node in self.prog:
            if node.kind == NodeKind.OpPush:
                self.push(int(node.token.value))
            elif node.kind == NodeKind.OpPlus:
                self.binop(self.builder.add)
            elif node.kind == NodeKind.OpMinus:
                self.binop(self.builder.sub)
            elif node.kind == NodeKind.OpMul:
                self.binop(self.builder.mul)
            elif node.kind == NodeKind.OpDump:
                self.dump()
            else:
                error(f"invalid op or identifier {node.token.value}")
        self.ret(0)
        return self.module
    
    # Op hardcoded functions
    
    def push(self, value: int) -> None:
        self.stack.append(self.builder.alloca(DEFAULT_INT, name="push"))
        self.builder.store(ir.Constant(DEFAULT_INT, value), self.stack[-1])

    def binop(self, fn: Callable, typ: ir.Type = None) -> None:
        b = self.builder.load(self.stack.pop())
        a = self.builder.load(self.stack.pop())
        result = fn(a, b, name="binop")
        if typ is None:
            typ = DEFAULT_INT
        self.stack.append(self.builder.alloca(typ))
        self.builder.store(result, self.stack[-1])
    
    def dump(self) -> None:
        result = self.builder.load(self.stack.pop())
        format_str = self.builder.bitcast(
            self.consts["digit_fmt"],
            ir.PointerType(ir.IntType(8)),
        )
        self.builder.call(
            self.functions["printf"],
            [format_str, result],
            name="dump"
        )

    def ret(self, code: int) -> None:
        self.builder.ret(ir.Constant(ir.IntType(32), code))


def global_str(module: ir.Module,
               value: str,
               name: str,
               cstr: bool = True) -> ir.GlobalVariable:
    
    x = f"{value}\0" if cstr else value
    x = ir.Constant(ir.ArrayType(ir.IntType(8), len(x)), bytearray(x.encode("utf8")))
    v = ir.GlobalVariable(module, x.type, name)
    v.linkage = "private"
    v.global_constant = True
    v.initializer = x
    return v


def error(msg: str) -> None:
    print(f"pile: error:\n  -> {msg}", file=stderr)
    exit(1)
