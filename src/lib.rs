type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
// TODO: make this a generic one and a CLI arg.
type Cell = u8;

mod error {
    use std::fmt;

    #[derive(Debug, Clone)]
    pub struct BfError {
        pub msg: String,
    }

    impl fmt::Display for BfError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.msg)
        }
    }
    impl std::error::Error for BfError {}
}

pub mod io {
    use crate::Result;
    use std::io::{Read, Write};

    pub trait Writer {
        fn write(&mut self, cell: crate::Cell) -> Result<()>;
    }

    pub struct AsciiWriter<'a> {
        pub sink: &'a mut dyn Write,
    }
    impl Writer for AsciiWriter<'_> {
        fn write(&mut self, cell: crate::Cell) -> Result<()> {
            use std::convert::TryFrom;
            let c = u8::try_from(cell)? as char;
            write!(self.sink, "{}", c)?;
            Ok({})
        }
    }
    pub struct IntWriter<'a> {
        pub sink: &'a mut dyn Write,
    }
    impl Writer for IntWriter<'_> {
        fn write(&mut self, cell: crate::Cell) -> Result<()> {
            let s = cell.to_string();
            write!(self.sink, "{}", s)?;
            Ok({})
        }
    }
    pub trait Reader {
        fn read(&mut self) -> Result<crate::Cell>;
    }

    pub struct AsciiReader<'a> {
        pub source: &'a mut dyn Read,
    }
    impl Reader for AsciiReader<'_> {
        fn read(&mut self) -> Result<crate::Cell> {
            let mut buf = [0; 1];
            self.source.read(&mut buf)?;
            Ok(buf[0] as crate::Cell)
        }
    }
}

pub mod runtime {
    use crate::error::BfError;
    use crate::io::{Reader, Writer};
    use crate::Result;

    pub struct Runtime<'a> {
        mem: Vec<crate::Cell>,
        code: Vec<Inst>,
        ptr: usize, // runtime checked to be in range [0, mem.length)
        pc: usize,
        writer: &'a mut dyn Writer,
        reader: &'a mut dyn Reader,
    }

    #[derive(Debug)]
    enum Inst {
        Right,
        Left,
        Inc,
        Dec,
        Out,
        In,
        // Enter contains the address of its corresponding Exit.
        Enter(usize),
        // And vice versa.
        Exit(usize),
    }
    fn parse(s: &str) -> Result<Vec<Inst>> {
        let mut code = Vec::<Inst>::with_capacity(s.len());
        let mut stack: Vec<usize> = vec![];
        for c in s.chars() {
            match c {
                '>' => code.push(Inst::Right),
                '<' => code.push(Inst::Left),
                '+' => code.push(Inst::Inc),
                '-' => code.push(Inst::Dec),
                '.' => code.push(Inst::Out),
                ',' => code.push(Inst::In),
                '[' => {
                    stack.push(code.len());
                    code.push(Inst::Enter(0));
                }
                ']' => {
                    let exit_addr = code.len();
                    let enter_addr = stack.pop().ok_or(BfError {
                        msg: "encounted ']' without corresponding '['.".to_owned(),
                    })?;
                    if let Inst::Enter(ref mut addr) = code[enter_addr] {
                        *addr = exit_addr;
                    }
                    code.push(Inst::Exit(enter_addr));
                }
                _ => (),
            }
        }
        if stack.is_empty() {
            // println!("parsed: {:?}", code);
            Ok(code)
        } else {
            Err(Box::new(BfError {
                msg: "parse finished with unbalanced '['.".to_owned(),
            }))
        }
    }
    impl Runtime<'_> {
        pub fn new<'w>(
            s: &str,
            reader: &'w mut dyn Reader,
            writer: &'w mut dyn Writer,
        ) -> Result<Runtime<'w>> {
            Ok(Runtime {
                mem: vec![0; 30000], // 30k mem
                code: parse(s)?,
                ptr: 0,
                pc: 0,
                reader: reader,
                writer: writer,
            })
        }
        pub fn run(&mut self) -> Result<()> {
            while self.pc < self.code.len() {
                self.step()?
            }
            return Ok({});
        }
        fn step(&mut self) -> Result<()> {
            let inst = &self.code[self.pc];
            // println!(
            //     "STEP: pc {}, inst {:?}, mem[ptr] = {}",
            //     self.pc, inst, self.mem[self.ptr]
            // );
            match inst {
                Inst::Right => self.move_ptr(1)?,
                Inst::Left => self.move_ptr(-1)?,
                Inst::Inc => self.mem[self.ptr] += 1,
                Inst::Dec => self.mem[self.ptr] -= 1,
                Inst::Out => self.writer.write(self.mem[self.ptr])?,
                Inst::In => {
                    let cell = self.reader.read()?;
                    self.mem[self.ptr] = cell
                }
                Inst::Enter(exit_addr) => {
                    if self.mem[self.ptr] == 0 {
                        self.pc = *exit_addr
                    }
                }
                Inst::Exit(enter_addr) => {
                    if self.mem[self.ptr] != 0 {
                        self.pc = *enter_addr
                    }
                }
            };
            self.pc += 1;
            Ok({})
        }
        // Move ptr rightward (toward positive infinity) with n steps.
        // If the result exceeds range [0, mem.length), ptr is not updated, and return error.
        fn move_ptr(&mut self, n: isize) -> Result<()> {
            let x = ((self.ptr as isize) + n) as usize;
            if (0..self.mem.len()).contains(&x) {
                self.ptr = x;
                Ok({})
            } else {
                Err(Box::new(BfError {
                    msg: "ptr moved out of range".to_owned(),
                }))
            }
        }
    }
}
