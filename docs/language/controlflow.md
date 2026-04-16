# Pile's control-flow

Pile features some kinds of control-flow structures to allow the user to
branch execution and create meaningful programs.

## Conditional branching
In Pile, the only conditional branching structure is `if`.

The `if` keyword, by itself, is nothing more than a special operation.
It pops a value from the stack and checks if it is _truthy_.
If so, it executes the block following it, otherwhise, it falls through or executes the `else` branch block.

Use `if` the following way:
```
<condition> if
 	<true>
[else
	<false>]
end 
``` 

## Unconditional branching
The `loop` structure provides a way to create unconditional branching in Pile.

It is basically an infinite loop. You can stop it with `break` keyword and you can go to the next iteration with `continue`.

Use `loop` the following way:
```
loop
	<block>
end
```

## Iterative branching
Iterative branching is achieved in Pile using the `for` structure.

`for` iterates through a sequence (a `string` or `array` value) on top of
the stack and assigns the currently iterating value to a variable name,
provided after the `for` keyword.

After that, executes the block of code and repeats it until the sequence is over.

Use `for` the following way:
```
<sequence> for <variable>
	<block>
end
```

---

> [**Next**](./procedures.md)
