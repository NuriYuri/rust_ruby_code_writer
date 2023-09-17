use std::{
    env, fs,
    io::{BufWriter, Write},
    vec,
};

use lib_ruby_parser::{
    nodes::{Begin, Str},
    Bytes, Loc, Node, Parser, ParserOptions,
};

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
    let mut node = result.ast.expect("Failed to read AST from ParserResult");
    mutate_module(&mut node);
    write_code(node.as_ref(), &mut writer, 0)?;
    writer.flush()?;
    return Ok(());
}

fn mutate_module(node: &mut Box<Node>) {
    match node.as_mut() {
        Node::Module(module) => {
            if let Some(body) = module.body.as_mut() {
                match body.as_mut() {
                    Node::Begin(begin) => push_string_to_begin(begin),
                    Node::Class(_) => {
                        let mut new_body = make_begin(body.as_ref().clone());
                        push_string_to_begin(&mut new_body);
                        module.body = Some(Box::new(Node::Begin(new_body)));
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

fn push_string_to_begin(begin: &mut Begin) {
    begin.statements.push(Node::Str(Str {
        value: Bytes::new("test".as_bytes().to_vec()),
        begin_l: Some(Loc { begin: 0, end: 1 }),
        end_l: Some(Loc { begin: 0, end: 1 }),
        expression_l: Loc { begin: 0, end: 1 },
    }))
}

fn make_begin(node: Node) -> Begin {
    return Begin {
        statements: vec![node],
        begin_l: None,
        end_l: None,
        expression_l: Loc { begin: 0, end: 1 },
    };
}
