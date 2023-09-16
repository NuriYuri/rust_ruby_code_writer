use std::{
    env, fs,
    io::{BufWriter, Write},
};

use lib_ruby_parser::{Parser, ParserOptions};

use crate::code_writer::write_code;
mod code_writer;
mod macros;

fn main() -> Result<(), std::io::Error> {
    let ruby_filename = env::args().nth(1).expect("Ruby Filename is expected");
    let ruby_file_content = fs::read_to_string(ruby_filename.clone())
        .expect(format!("Failed to read ruby file: {}", ruby_filename).as_str());
    let options = ParserOptions {
        record_tokens: false,
        ..Default::default()
    };
    let parser = Parser::new(ruby_file_content, options);
    let result = parser.do_parse();
    let mut writer = BufWriter::new(std::io::stdout());
    write_code(
        result.ast.expect("Failed to read AST from ParserResult"),
        &mut writer,
        0,
    )?;
    writer.flush()?;
    return Ok(());
}
