use clap::{arg_enum, value_t_or_exit, Arg};
use std::error::Error;
use std::fs::File;
use std::io::Read;

arg_enum! {
    #[derive(Debug)]
    pub enum OutputCodec {
        Ascii,
        Int
    }
}

// Reads a BF program from stdin, executes, outputs as ascii. Don't support input.
// TODO: an arg of filename, a flag of output codec (ascii? int?), a flag of mem size, stdin -> ".", stdout -> ","
fn main() -> Result<(), Box<dyn Error>> {
    let matches = clap::App::new("rsbf, a Rust-based Brainfuck intepreter")
        .about("Reads BF program from a file, takes input from stdin and outputs to stdout.")
        .arg(
            Arg::with_name("output_codec")
                .short("c")
                .long("codec")
                .possible_values(&OutputCodec::variants())
                .case_insensitive(true)
                .default_value("Ascii")
                .help("Output format. Case insensitive."),
        )
        .arg(
            Arg::with_name("filename")
                .help("a bf file to intepret")
                .required(true),
        )
        .get_matches();

    let mut sink = std::io::stdout();
    let mut writer: Box<dyn rsbf::io::Writer> =
        match value_t_or_exit!(matches, "output_codec", OutputCodec) {
            OutputCodec::Ascii => Box::new(rsbf::io::AsciiWriter { sink: &mut sink }),
            OutputCodec::Int => Box::new(rsbf::io::IntWriter { sink: &mut sink }),
        };
    let mut reader = rsbf::io::AsciiReader {
        source: &mut std::io::stdin(),
    };
    let filename = matches.value_of("filename").unwrap();
    let mut file = File::open(filename)?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;
    let mut rt = rsbf::runtime::Runtime::new(buffer.as_str(), &mut reader, &mut *writer)?;
    rt.run()
}
