# Vali Assembly Language Commands

## Input and Output

### Input
`inw dst`

Reads a word from the keyboard and stores it in *dst*. Similarly for `inb`, `inf`.

### Output
`outw src`

Print *src* out on screen as a signed integer. Similarly for `outb` and `outf`.

## Memory and Arithmatic

These calls manipulate the data in the memory.

### Move
`movw src dst`

Sets the value of *dst* equal to the value of *src* (i.e., dst=src). Similar for `movb` and `movf`.

### Add
`addw src dst`

Adds the contents of `src` to the contents of `dst` (i.e., dst=dst + src). Similarly for `addb` and `addf`.

### Subtract
`subw` src dst`

Subtracts the contents of `src` from the contents of `dst` (i.e., dst=dst - src). Similarly for `subb` and `subf`.

### Multiplication
`mulw src dst`

Multiplies the contents of `dst` by the contents of `src` (i.e., dst = dst * src). Similarly for `mulb` and `mulf`.

### Division
`divw src dst`

Divides the contents of dst by the contents of src (i.e., dst = dst / src). Similarly for `divb` and `divf`. For words and bytes performs integer division. For floats performs floating point division.

## Conversion

### Float to Integer
`cvtfw src dst`

Converts a floating point value in `src` to an integer (word) value in `dst`. Rounds to the nearest integer.

### Integer to Float
`cvtwf src dst`

Converts an integer (word) value in src to a floating point value in `dst`.

## Control Flow

### Compare
`cmpw op1 op2`

Compares `op1` and `op2` and sets the conditional codes appropriately. Similarly for `cmpb` and `cmpf`. The condition codes can be:

- Equal (eq)
- Not Equal (neq)
- Greater Than (gtr)
- Less Than (lss)
- Greater Than or Equal To (geq)
- Less Than or Equal To (leq)

### Branch If
`beq dst`

Branch to destination `dst` if the result of the last comparison was "equal". Similarly for `bneq`, `bgtr`, `blss`, `bgeq`, `bleq`.

### Unconditional Branch
`jmp dst`

Does and inconditional branch (regardless of condition codes) to `dst`.

### Call Procedure
`call num dst`

Calls procedure specified by `dst` assuming `num` parameters have been pushed onto the run time stack.

### Return
`ret`

### End Execution
`end`
