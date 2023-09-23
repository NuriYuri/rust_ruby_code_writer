use lib_ruby_parser::{
    nodes::{Begin, Str},
    Bytes, Loc, Node,
};

pub fn mutate_module(node: &mut Box<Node>) {
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
