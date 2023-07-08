from dataclasses import dataclass
from enum import Enum, auto
from llvmlite import ir, binding
from sys import stderr
from typing import Callable
from typing import Dict, Iterable, Tuple

I32: ir.IntType = ir.IntType(32)
BOOL: ir.IntType = ir.IntType(1)
FLOAT: ir.FloatType = ir.FloatType()
DOUBLE: ir.DoubleType = ir.DoubleType()


class TokenKind(Enum):
    Int = auto()
    Float = auto()
    Word = auto()


@dataclass
class Token:
    value: str
    kind: TokenKind
    position: Tuple[str, int, int]


# I stole this lexer from Tsoding Daily:
# https://youtube.com/clip/Ugkx0EDLNMP4aS5yHNxqmHehqQ6iYG3OCOSC


def find_col(string: str, col: int, pred: Callable) -> int:
    while col < len(string) and not pred(string[col]):
        col += 1
    return col


def lex_line(line: str) -> Iterable[Tuple[int, str]]:
    col = find_col(line, 0, lambda x: not x.isspace())
    while col < len(line):
        end = find_col(line, col, lambda x: x.isspace())
        yield (col, line[col:end])
        col = find_col(line, end, lambda x: not x.isspace())


def lex_file(path: str) -> Iterable[Token]:
    with open(path, "r") as f:
        yield from (
            Token(val, classify_token(val), (path, row+1, col))
            for row, x in enumerate(f.readlines())
            for col, val in lex_line(x) 
        )


def is_cls(cls: type, text: str) -> bool:
    try:
        cls(text)
    except ValueError:
        return False
    else:
        return True


def classify_token(token: str) -> TokenKind:
    if is_cls(int, token):
        return TokenKind.Int
    elif is_cls(float, token):
        return TokenKind.Float
    return TokenKind.Word


class NodeKind(Enum):
    Symbol = auto()
    Int = auto()
    Float = auto()


@dataclass
class Node:
    token: Token
    kind: NodeKind


Program = Iterable[Node]
class UnreachableError(Exception): ...


def match_kind(token: Token) -> NodeKind:
    if token.kind == TokenKind.Word:
        return NodeKind.Symbol
    elif token.kind == TokenKind.Int:
        return NodeKind.Int
    elif token.kind == TokenKind.Float:
        return NodeKind.Float
    else:
        raise UnreachableError("match_kind isn't handling all TokenKind variants")


def parse(tokens: Iterable[Token]) -> Program:
    # TODO: Make the parser recognize Symbols in wrong places.
    # TODO: Make the parser be able to parse code blocks (if, while, etc.).
    for token in tokens:
        yield Node(token, match_kind(token))


def global_str(module: ir.Module,
               value: str,
               name: str,
               cstr: bool = True) -> ir.GlobalVariable:
    
    x = f"{value}\0" if cstr else value
    const = ir.Constant(ir.ArrayType(ir.IntType(8), len(x)), bytearray(x.encode("utf8")))
    v = ir.GlobalVariable(module, const.type, name)
    v.linkage = "private"
    v.global_constant = True
    v.initializer = const
    return v


def error(msg: str, pos: Tuple[str, int, int]) -> None:
    print(f"pile: error at {pos[0]}:{pos[1]}:{pos[2]}:", file=stderr)
    print(f"  -> {msg}", file=stderr)
    exit(1)


binding.initialize()
binding.initialize_native_target()
binding.initialize_native_asmprinter()
module = ir.Module(name="pile")
module.triple = binding.get_default_triple()
main_fn = ir.Function(
    module,
    ir.FunctionType(ir.IntType(32), []),
    name="main"
)
entry = main_fn.append_basic_block(name="entry")
builder = ir.IRBuilder(entry)
merge_block = entry
stack: list = []
conditionals: list = []
FUNCTIONS: Dict[str, ir.Function] = {}
CONSTS: Dict[str, ir.GlobalVariable] = {}


@dataclass
class Cond:
    true: ir.Block
    false: ir.Block
    merge: ir.Block
    has_else: bool


def compile(prog: Program) -> ir.Module:
    ops: Dict[str, Callable] = {
        "+": lambda: add(),
        "-": lambda: sub(),
        "*": lambda: mul(),
        ">": lambda: gt(),
        "<": lambda: lt(),
        ">=": lambda: ge(),
        "<=": lambda: le(),
        "!=": lambda: ne(),
        "=": lambda: eq(),
        "dup": lambda: dup(),
        "drop": lambda: drop(),
        "over": lambda: over(),
        "rot": lambda: rot(),
        "swap": lambda: swap(),
        "dump": lambda: dump(),
        "fdump": lambda: fdump(),
    }
    for node in prog:
        if node.kind == NodeKind.Int:
            ipush(int(node.token.value))
        elif node.kind == NodeKind.Float:
            fpush(float(node.token.value))
        elif node.token.value in ops:
            action = ops[node.token.value]
            action()
        elif node.token.value == "if":
            comparison = builder.load(stack.pop())
            true = main_fn.append_basic_block("if")
            false = main_fn.append_basic_block("else")
            merge = main_fn.append_basic_block("merge")
            builder.cbranch(comparison, true, false)
            conditionals.append(Cond(true, false, merge, False))
            builder.position_at_end(true)
        elif node.token.value == "else":
            cond = conditionals[-1]
            cond.has_else = True
            builder.branch(cond.merge)
            builder.position_at_end(cond.false)
        elif node.token.value == "end": 
            cond = conditionals.pop()
            builder.branch(cond.merge)
            if not cond.has_else:
                builder.position_at_end(cond.false)
                builder.branch(cond.merge)
            builder.position_at_end(cond.merge)
        else:
            error(f"invalid op or identifier `{node.token.value}`",
                  node.token.position)
    ret(0)
    return module


def ipush(value: int) -> None:
    stack.append(builder.alloca(I32))
    builder.store(ir.Constant(I32, value), stack[-1])


def fpush(value: float) -> None:
    stack.append(builder.alloca(FLOAT))
    builder.store(ir.Constant(FLOAT, value), stack[-1])


def dup() -> None:
    a = builder.load(stack[-1])
    stack.append(builder.alloca(a.type))
    builder.store(a, stack[-1])


def drop() -> None:
    # Just ignore value. I didn't find
    # any way to undo the alloca intruction :(
    stack.pop()


def over() -> None:
    a = builder.load(stack[-2])
    stack.append(builder.alloca(a.type))
    builder.store(a, stack[-1])


def swap() -> None:
    a = builder.load(stack.pop(-2))
    stack.append(builder.alloca(a.type))
    builder.store(a, stack[-1])


def rot() -> None:
    a = builder.load(stack.pop(-3))
    stack.append(builder.alloca(a.type))
    builder.store(a, stack[-1])


def add() -> None:
    b = builder.load(stack.pop())
    a = builder.load(stack.pop())
    result = (builder.fadd(a, b)
              if a.type in (FLOAT, DOUBLE)
              else builder.add(a, b))
    stack.append(builder.alloca(a.type))
    builder.store(result, stack[-1])


def sub() -> None:
    b = builder.load(stack.pop())
    a = builder.load(stack.pop())
    result = (builder.fsub(a, b)
              if a.type in (FLOAT, DOUBLE)
              else builder.sub(a, b))
    stack.append(builder.alloca(a.type))
    builder.store(result, stack[-1])


def mul() -> None:
    b = builder.load(stack.pop())
    a = builder.load(stack.pop())
    result = (builder.fmul(a, b)
              if a.type in (FLOAT, DOUBLE)
              else builder.mul(a, b))
    stack.append(builder.alloca(a.type))
    builder.store(result, stack[-1])


def gt() -> None:
    b = builder.load(stack.pop())
    a = builder.load(stack.pop())
    result = (builder.fcmp_ordered(">", a, b)
              if a.type in (FLOAT, DOUBLE)
              else builder.icmp_signed(">", a, b))
    stack.append(builder.alloca(BOOL))
    builder.store(result, stack[-1])


def lt() -> None:
    b = builder.load(stack.pop())
    a = builder.load(stack.pop())
    result = (builder.fcmp_ordered("<", a, b)
              if a.type in (FLOAT, DOUBLE)
              else builder.icmp_signed("<", a, b))
    stack.append(builder.alloca(BOOL))
    builder.store(result, stack[-1])


def ge() -> None:
    b = builder.load(stack.pop())
    a = builder.load(stack.pop())
    result = (builder.fcmp_ordered(">=", a, b)
              if a.type in (FLOAT, DOUBLE)
              else builder.icmp_signed(">=", a, b))
    stack.append(builder.alloca(BOOL))
    builder.store(result, stack[-1])


def le() -> None:
    b = builder.load(stack.pop())
    a = builder.load(stack.pop())
    result = (builder.fcmp_ordered("<=", a, b)
              if a.type in (FLOAT, DOUBLE)
              else builder.icmp_signed("<=", a, b))
    stack.append(builder.alloca(BOOL))
    builder.store(result, stack[-1])


def ne() -> None:
    b = builder.load(stack.pop())
    a = builder.load(stack.pop())
    result = (builder.fcmp_ordered("!=", a, b)
              if a.type in (FLOAT, DOUBLE)
              else builder.icmp_signed("!=", a, b))
    stack.append(builder.alloca(BOOL))
    builder.store(result, stack[-1])


def eq() -> None:
    b = builder.load(stack.pop())
    a = builder.load(stack.pop())
    result = (builder.fcmp_ordered("==", a, b)
              if a.type in (FLOAT, DOUBLE)
              else builder.icmp_signed("==", a, b))
    stack.append(builder.alloca(BOOL))
    builder.store(result, stack[-1])


def dump() -> None:
    result = builder.load(stack.pop())

    if "digit_fmt" not in CONSTS:
        CONSTS["digit_fmt"] = global_str(module, "%d\n", "digit_fmt")
    
    if "printf" not in FUNCTIONS:
        typ = ir.FunctionType(ir.IntType(32),
                             [ir.PointerType(ir.IntType(8))],
                             var_arg=True)
        FUNCTIONS["printf"] = ir.Function(module, typ, name="printf")
    
    format_str = builder.bitcast(CONSTS["digit_fmt"],
                                 ir.PointerType(ir.IntType(8)))
    builder.call(FUNCTIONS["printf"], [format_str, result])


def fdump() -> None:
    result = builder.load(stack.pop())

    if "float_fmt" not in CONSTS:
        CONSTS["float_fmt"] = global_str(module, "%f\n", "float_fmt")
    
    if "printf" not in FUNCTIONS:
        typ = ir.FunctionType(ir.IntType(32),
                             [ir.PointerType(ir.IntType(8))],
                             var_arg=True)
        FUNCTIONS["printf"] = ir.Function(module, typ, name="printf")
    
    result = builder.fpext(result, ir.DoubleType())
    format_str = builder.bitcast(CONSTS["float_fmt"],
                                 ir.PointerType(ir.IntType(8)))
    builder.call(FUNCTIONS["printf"], [format_str, result])


def ret(code: int) -> None:
    builder.ret(ir.Constant(ir.IntType(32), code))
