# rsbf

rsbf is a Rust based Brainfuck intepreter.

## Usage

It reads a file and inteprets it as a Brainfuck program. Inputs (`,`) are read from stdin, and outputs (`.`) are written to stdout. By default, the output uses the ascii codec, which means a cell with integer 65 is outputed as `A`. Another codec `Int` can be chosen, with which a cell of 65 is outputed as `65`.

```
cargo run ~/test.bf
```

## Spec

Cell is typed unsigned 8-bit integer. An oveflow is treated as two's complement wrap.
Length of the tape is 30,000. Moving pointer out of region [0, 30000) would trigger exception and quits the program.


