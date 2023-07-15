from dataclasses import dataclass
from enum import Enum, auto
from llvmlite import ir, binding
from sys import stderr
from typing import Callable, TextIO
from typing import Dict, Iterable, Tuple

I32: ir.IntType = ir.IntType(32)
BOOL: ir.IntType = ir.IntType(1)
FLOAT: ir.FloatType = ir.FloatType()
DOUBLE: ir.DoubleType = ir.DoubleType()


class TokenKind(Enum):
    Int = auto()
    Float = auto()
    String = auto()
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


def lex_line(line: str) -> Iterable[Tuple[int, str, TokenKind]]:
    col = find_col(line, 0, lambda x: not x.isspace())
    line = line.split("//")[0]
    end = None
    while col < len(line):
        if line[col] == '"':
            # TODO: Report unterminated and unstarted strings
            end = find_col(line, col+1, lambda x: x == '"')
            yield (col, line[col+1:end], TokenKind.String)
            col = find_col(line, end+1, lambda x: not x.isspace())
        else:
            end = find_col(line, col, lambda x: x.isspace())
            token = line[col:end]
            yield (col, line[col:end], classify_token(token))
            col = find_col(line, end, lambda x: not x.isspace())


def lex_file(path: str) -> Iterable[Token]:
    with open(path, "r") as f:
        yield from (
            Token(val, kind, (path, row+1, col))
            for row, x in enumerate(f.readlines())
            for col, val, kind in lex_line(x) 
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
    elif token.startswith('"'):
        return TokenKind.String
    return TokenKind.Word


class NodeKind(Enum):
    Symbol = auto()
    Int = auto()
    Float = auto()
    String = auto()


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
    elif token.kind == TokenKind.String:
        return NodeKind.String
    else:
        raise UnreachableError("match_kind isn't handling all TokenKind variants")


def parse(tokens: Iterable[Token]) -> Program:
    # TODO: Make the parser recognize Symbols in wrong places.
    # TODO: Make the parser be able to parse code blocks (if, while, etc.).
    for token in tokens:
        yield Node(token, match_kind(token))


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


@dataclass
class If(Cond):
    true: ir.Block
    false: ir.Block
    merge: ir.Block
    has_else: bool


@dataclass
class While(Cond):
    true: ir.Block
    false: ir.Block
    merge: ir.Block
    cmp_return: ir.Block


def compile(prog: Program) -> ir.Module:
    ops: Dict[str, Callable] = {
        "+": add, "-": sub,
        "*": mul, ">": gt,
        "<": lt, ">=": ge,
        "<=": le, "!=": ne,
        "=": eq, "dup": dup,
        "drop": drop, "over": over,
        "rot": rot, "swap": swap,
        "dump": dump, "fdump": fdump,
        "if": start_cond, "else": else_cond,
        "while": start_loop, "do": do_loop,
        "end": end_cond,
    }
    for node in prog:
        if node.kind == NodeKind.Int:
            ipush(int(node.token.value))
        elif node.kind == NodeKind.Float:
            fpush(float(node.token.value))
        elif node.kind == NodeKind.String:
            spush(node.token.value)
        elif node.token.value in ops:
            action = ops[node.token.value]
            action()
        else:
            throw(node.token.position,
                  "word error",
                  "unknown operation or defined "
                  f"identifier `{node.token.value}`")
    ret(0)
    return module


def start_cond() -> None:
    cmp = builder.load(stack.pop())
    true = main_fn.append_basic_block("if")
    false = main_fn.append_basic_block("else")
    merge = main_fn.append_basic_block("end")
    builder.cbranch(cmp, true, false)
    conditionals.append(If(true, false, merge, False))
    builder.position_at_end(true)


def start_loop() -> None:
    cmp_return = main_fn.append_basic_block("while")
    true = main_fn.append_basic_block("do")
    merge = main_fn.append_basic_block("end")
    conditionals.append(While(true, merge, merge, cmp_return))
    builder.branch(cmp_return)
    builder.position_at_end(cmp_return)


def do_loop() -> None:
    cmp = builder.load(stack.pop())
    cond = conditionals[-1]
    builder.cbranch(cmp, cond.true, cond.merge)
    builder.position_at_end(cond.true)


def else_cond() -> None:
    cond = conditionals[-1]
    cond.has_else = True
    builder.branch(cond.merge)
    builder.position_at_end(cond.false)


def end_cond() -> None:
    cond = conditionals.pop()
    if isinstance(cond, If):
        builder.branch(cond.merge)
        if not cond.has_else:
            builder.position_at_end(cond.false)
            builder.branch(cond.merge)
        builder.position_at_end(cond.merge)
    elif isinstance(cond, While):
        builder.branch(cond.cmp_return)
        builder.position_at_end(cond.merge)


def ipush(value: int) -> None:
    stack.append(builder.alloca(I32))
    builder.store(ir.Constant(I32, value), stack[-1])


def fpush(value: float) -> None:
    stack.append(builder.alloca(FLOAT))
    builder.store(ir.Constant(FLOAT, value), stack[-1])


def spush(value: str) -> None:
    byte_value = bytearray(bytes(value, "utf-8"))
    name = f".{len(CONSTS)}"
    if name not in CONSTS:
        CONSTS[name] = global_str(byte_value)
    stack.append(builder.alloca(ir.PointerType(ir.IntType(8))))
    string = builder.bitcast(CONSTS[name], ir.PointerType(ir.IntType(8)))
    builder.store(string, stack[-1])


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
    a = builder.load(stack[-1])
    result = (builder.fadd(a, b)
              if a.type in (FLOAT, DOUBLE)
              else builder.add(a, b))
    builder.store(result, stack[-1])


def sub() -> None:
    b = builder.load(stack.pop())
    a = builder.load(stack[-1])
    result = (builder.fsub(a, b)
              if a.type in (FLOAT, DOUBLE)
              else builder.sub(a, b))
    builder.store(result, stack[-1])



def mul() -> None:
    b = builder.load(stack.pop())
    a = builder.load(stack[-1])
    result = (builder.fmul(a, b)
              if a.type in (FLOAT, DOUBLE)
              else builder.mul(a, b))
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
    
    format_str = const_str(bytearray(b"%d\n"))
    
    if "printf" not in FUNCTIONS:
        typ = ir.FunctionType(ir.IntType(32),
                             [ir.PointerType(ir.IntType(8))],
                             var_arg=True)
        FUNCTIONS["printf"] = ir.Function(module, typ, name="printf")
    format_str = builder.bitcast(format_str,
                                 ir.PointerType(ir.IntType(8)))
    builder.call(FUNCTIONS["printf"], [format_str, result])


def fdump() -> None:
    result = builder.load(stack.pop())

    format_str = const_str(bytearray(b"%f\n\0"))
    
    if "printf" not in FUNCTIONS:
        typ = ir.FunctionType(ir.IntType(32),
                             [ir.PointerType(ir.IntType(8))],
                             var_arg=True)
        FUNCTIONS["printf"] = ir.Function(module, typ, name="printf")
    
    result = builder.fpext(result, ir.DoubleType())
    format_str = builder.bitcast(format_str,
                                 ir.PointerType(ir.IntType(8)))
    builder.call(FUNCTIONS["printf"], [format_str, result])


def ret(code: int) -> None:
    builder.ret(ir.Constant(ir.IntType(32), code))


def global_str(value: bytearray, cstr: bool = True) -> ir.GlobalVariable:
    x = value + b'\0' if cstr else value
    char_arr = ir.ArrayType(ir.IntType(8), len(x))
    x = ir.Constant(char_arr, x)
    global_var = ir.GlobalVariable(module,
                                   char_arr,
                                   name=f"{hex(id(value))}")
    global_var.initializer = x
    return global_var


def const_str(value: bytearray, cstr: bool = True) -> ir.AllocaInstr:
    x = value + b'\0' if cstr else value
    char_arr = ir.ArrayType(ir.IntType(8), len(x))
    x = ir.Constant(char_arr, x)
    string = builder.alloca(char_arr)
    builder.store(x, string)
    return string


def indent(file: TextIO,
           text: str,
           prefix: int = None,
           suffix: int = None) -> None:
    if prefix is None:
        prefix = 2
    if suffix is None:
        suffix = 0
    file.write(' '*prefix + text + ' '*suffix + '\n')


def throw(pos: Tuple[str, int, int],
          kind: str, msg: str,
          note: str = None) -> None:
    stderr.write("pile: error at "
                 f"{pos[0]}:{pos[1]}:{pos[2]}:\n")
    indent(stderr, f"| {kind}:")
    for line in break_line_at(25, msg):
        indent(stderr, f"|    {line}")
    if note is not None:
        for line in break_line_at(25, note):
            indent(stderr, f"+ {line}")
    exit(1)


def break_line_at(char_pos: int, value: str) -> Iterable[str]:
    words = value.split()
    current_line = ''
    current_length = 0
    for word in words:
        word_length = len(word) + 1
        if current_length + word_length > char_pos:
            yield current_line
            current_line = ''
            current_length = 0
        current_line += f'{word} '
        current_length += word_length
    yield current_line
