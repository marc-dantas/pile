# Pile's operations

> In this list, it will be used the Forth stack notation to describe the stack state needed to perform the operation being documented. Please consult this notation at [forth-standard.org](https://forth-standard.org/standard/notation) at section **2.2.2 Stack notation**.

## Math

| Name      | Operation | Forth stack notation | Description                                                   |
|-----------|-----------|----------------------|---------------------------------------------------------------|
| Add       | `+`       | `( a b -- a+b )`     | Pops A and B from the stack and pushes their sum.             |
| Subtract  | `-`       | `( a b -- a-b )`     | Pops A and B from the stack and pushes their difference.      |
| Multiply  | `*`       | `( a b -- a*b )`     | Pops A and B from the stack and pushes their product.         |
| Divide    | `/`       | `( a b -- a/b )`     | Pops A and B from the stack and pushes their quotient.        |
| Modulo    | `%`       | `( a b -- a%b )`     | Pops A and B from the stack and pushes their division modulo. |


## Relational
| Name          | Operation | Forth stack notation | Description                                                                            |
|---------------|-----------|----------------------|----------------------------------------------------------------------------------------|
| Less          | `<`       | `( a b -- a<b )`     | Pops A and B from the stack and pushes the result of comparison less-than.             |
| Greater       | `>`       | `( a b -- a>b )`     | Pops A and B from the stack and pushes the result of comparison greater-than.          |
| Less-equal    | `<=`      | `( a b -- a<=b )`    | Pops A and B from the stack and pushes the result of comparison less-than or equal.    |
| Greater-equal | `>=`      | `( a b -- a>=b )`    | Pops A and B from the stack and pushes the result of comparison greater-than or equal. |
| Equal         | `=`       | `( a b -- a==b )`    | Pops A and B from the stack and pushes the result of comparison equal.                 |
| Not Equal     | `!=`      | `( a b -- a!=b )`    | Pops A and B from the stack and pushes the result of comparison not equal.             |

## Bitwise


| Name        | Operation | Forth stack notation | Description                                                                     |
|-------------|-----------|----------------------|---------------------------------------------------------------------------------|
| B-Or        | `|`       | `( a b -- a|b )`     | Pops A and B from the stack and pushes the result of Bitwise Or to them.        |
| B-And       | `&`       | `( a b -- a&b )`     | Pops A and B from the stack and pushes the result of Bitwise And to them.       |
| Shift Left  | `<<`      | `( a b -- a<<b )`    | Pops A and B from the stack and pushes the result of A shifted left by B bits.  |
| Shift Right | `>>`      | `( a b -- a>>b )`    | Pops A and B from the stack and pushes the result of A shifted right by B bits. |
| B-Not       | `~`       | `( a -- ~a )`        | Pops A from the stack and pushes the result of Bitiwise Not to it.              |

> **NOTE**: The B-Or, B-And and B-Not operations also work with the `bool` type in Pile, so these are also logic operators.

## Sequence operations

| Name           | Operation | Forth stack notation | Description                                                                             |
|----------------|-----------|----------------------|-----------------------------------------------------------------------------------------|
| Read at index  | `@`       | `( s i -- s[i] )`    | Pushes the read value at the index I of the sequence S, being it an array or string.    |
| Write at index | `!`       | `( s i x -- )`       | Replaces the value at I in the sequence S, being it an array or string, by the value X. |

## Misc

| Name   | Operation | Forth stack notation | Description                                         |
|--------|-----------|----------------------|-----------------------------------------------------|
| Is Nil | `?`       | `( x -- x==nil )`    | Pops X from the stack and pushes if it is `nil`.    |
| Trace  | `trace`   | `( x -- x )`         | Debug-prints the value that is on top of the stack. |

---

> [**Next**](./builtins.md)
