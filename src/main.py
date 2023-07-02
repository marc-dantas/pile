from pile import *

# It's empty because we can't read and process any kind of pile source code yet :(
prog = LLVMCompiler(object())

# hardcoded simple stack-based program

prog.push(10)
prog.push(20)
prog.binop(prog.builder.add)
prog.dump()

prog.ret(0)

# Print compiled result (LLVM IR)
print(prog.module)
