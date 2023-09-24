use std::{
    env, fs,
    io::{BufWriter, Write},
};

use lib_ruby_parser::{nodes::Int, Loc, Node, Parser, ParserOptions};
use tests::{
    explore_constants::{explore_constants, make_constant_hash_map},
    insert_test_in_module::mutate_module,
};

use crate::code_writer::write_code;
mod code_writer;
mod macros;
mod tests;

fn main() -> Result<(), std::io::Error> {
    let ruby_filename = env::args().nth(1).expect("Ruby Filename is expected");
    let instruction = env::args()
        .nth(2)
        .expect("instruction is expected (write, explore_constants)");
    let ruby_file_content = fs::read_to_string(ruby_filename.clone())
        .expect(format!("Failed to read ruby file: {}", ruby_filename).as_str());
    let options = ParserOptions {
        record_tokens: false,
        ..Default::default()
    };
    let parser = Parser::new(ruby_file_content, options);
    let result = parser.do_parse();
    let mut node = result.ast.expect("Failed to read AST from ParserResult");

    match instruction.as_str() {
        "write" => {
            let mut writer = BufWriter::new(std::io::stdout());
            mutate_module(&mut node);
            write_code(node.as_ref(), &mut writer, 0)?;
            writer.flush()?;
        }
        "explore_constants" => {
            let mut constants = make_constant_hash_map();
            explore_constants(&mut constants, &node, &|send| {
                if send.recv.is_none() {
                    if send.method_name.eq("gen") && send.args.len() == 2 {
                        if let Node::Int(arg1) = &send.args[0] {
                            if let Node::Int(arg2) = &send.args[1] {
                                return Some(Box::new(Node::Int(Int {
                                    value: (384
                                        + (arg1.value.parse::<i32>().unwrap())
                                        + ((arg2.value.parse::<i32>().unwrap()) * 8))
                                        .to_string(),
                                    operator_l: None,
                                    expression_l: Loc { begin: 0, end: 0 },
                                })));
                            }
                        }
                    }
                }
                return None;
            });
            println!("{:?}", constants);
        }
        _ => {
            println!("Unknown instruction, use write or explore_constants")
        }
    }
    return Ok(());
}
