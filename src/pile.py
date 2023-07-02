from io import BufferedReader
from typing import Callable
from llvmlite import ir, binding
from os import SEEK_END


# NOTE: For now it's unused, because we don't have a proper parser for this
# I know this lexer is shit. It's here just to save this very good code.
class Lexer:
    
    source: BufferedReader
    
    def __init__(self, input: BufferedReader):
        self.source = input
    
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
        x = self.source.seek(0, SEEK_END)
        self.source.seek(0, ptr)
        return x
    
    def lex(self):
        buflen = self.tell()
        while self.index < buflen:
            start = self.index
            end = self.advance()
            if self.index < buflen:
                end -= 1
            if start != end:
                self.source.seek(start)
                yield self.__source.read(end - start)


# Pretty good. Isn't it?
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
        self.stack.append(self.builder.alloca(ir.IntType(32)))
        self.builder.store(ir.Constant(ir.IntType(32), value), self.stack[-1])

    def binop(self, fn: Callable):
        b = self.builder.load(self.stack.pop())
        a = self.builder.load(self.stack.pop())
        result = fn(a, b)
        self.stack.append(self.builder.alloca(ir.IntType(32)))
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
