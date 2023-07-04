from pile import *
from sys import argv


assert argv[1:], "No file was provided."

# It's now usable!
mod = LLVMCompiler(
    Parser(
        Lexer(open(argv[1], "r")).lex()
    ).parse()
).compile()

print(mod)