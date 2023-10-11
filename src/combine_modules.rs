use std::collections::HashMap;

use lib_ruby_parser::{
    nodes::{Begin, Send},
    Loc, Node,
};

type KnownModules<'a> = HashMap<String, &'a mut Node>;

pub fn combine_modules<'a>(node: &'a mut Node) -> () {
    match node {
        Node::Begin(begin) => {
            let mut known_modules: KnownModules = HashMap::new();
            let mut valid_indexes: Vec<bool> = Vec::with_capacity(begin.statements.len());
            for node in begin.statements.iter_mut() {
                match node {
                    Node::Class(_) | Node::Module(_) => {
                        let len_before = known_modules.len();
                        combine_modules_internal(node, &mut known_modules);
                        valid_indexes.push(len_before < known_modules.len())
                    }
                    _ => valid_indexes.push(true),
                }
            }
            begin.statements = begin
                .statements
                .iter()
                .enumerate()
                .filter(|(i, _)| valid_indexes[*i])
                .map(|(_, node)| node.to_owned())
                .collect();
        }
        Node::Class(klass) => {
            normalize_children(&mut klass.body);
            if let Some(body) = klass.body.as_mut() {
                combine_modules(body);
            }
        }
        Node::Module(klass) => {
            normalize_children(&mut klass.body);
            if let Some(body) = klass.body.as_mut() {
                combine_modules(body);
            }
        }
        _ => {}
    }
}

fn combine_modules_internal<'a>(node: &'a mut Node, known_modules: &mut KnownModules<'a>) -> () {
    match node {
        Node::Class(klass) => {
            let module_name = const_name_from_node(&klass.name);
            normalize_children(&mut klass.body);
            if known_modules.contains_key(&module_name) {
                let existing_class = known_modules.get_mut(&module_name).expect("");
                combine_bodies(
                    get_body_from_existing_module(existing_class),
                    get_body_from_optional_children(&mut klass.body),
                );
            } else {
                if let Some(body) = klass.body.as_mut() {
                    combine_modules(body);
                }
                known_modules.insert(module_name, node);
            }
        }
        Node::Module(klass) => {
            let module_name = const_name_from_node(&klass.name);
            normalize_children(&mut klass.body);
            if known_modules.contains_key(&module_name) {
                let existing_class = known_modules.get_mut(&module_name).expect("");
                combine_bodies(
                    get_body_from_existing_module(existing_class),
                    get_body_from_optional_children(&mut klass.body),
                );
            } else {
                if let Some(body) = klass.body.as_mut() {
                    combine_modules(body);
                }
                known_modules.insert(module_name, node);
            }
        }
        _ => {}
    }
}

fn get_body_from_existing_module<'a>(node: &'a mut Node) -> &'a mut Begin {
    match node {
        Node::Class(klass) => return get_body_from_optional_children(&mut klass.body),
        Node::Module(module) => return get_body_from_optional_children(&mut module.body),
        _ => panic!(
            "Unexpected node {} while getting body from existing module",
            node.str_type()
        ),
    }
}

fn get_body_from_optional_children<'a>(children: &'a mut Option<Box<Node>>) -> &'a mut Begin {
    match children.as_deref_mut() {
        Some(Node::Begin(begin)) => return begin,
        _ => panic!("Failed to get normalized body from children"),
    }
}

fn statements_contains_modifier(statements: &Vec<Node>) -> bool {
    for node in statements.iter() {
        if let Node::Send(send) = node {
            if send.args.len() > 0 || send.dot_l.is_some() || send.operator_l.is_some() {
                continue;
            }
            match send.method_name.as_str() {
                "private" | "protected" | "module_function" => {
                    return true;
                }
                _ => {}
            }
        }
    }
    return false;
}
fn combine_bodies(existing: &mut Begin, new: &Begin) {
    if statements_contains_modifier(&existing.statements) {
        existing.statements.push(Node::Send(Send {
            recv: None,
            method_name: String::from("public"),
            args: vec![],
            dot_l: None,
            selector_l: None,
            begin_l: None,
            end_l: None,
            operator_l: None,
            expression_l: Loc { begin: 0, end: 1 },
        }));
    }
    existing
        .statements
        .extend(new.statements.to_owned().into_iter());
}

/** Force children to be Some(Begin) node */
fn normalize_children<'a>(children: &'a mut Option<Box<Node>>) -> () {
    if let Some(node) = children {
        match node.as_ref() {
            Node::Begin(_) => {}
            something_else => {
                *children = Some(Box::new(Node::Begin(Begin {
                    statements: vec![something_else.to_owned()],
                    begin_l: None,
                    end_l: None,
                    expression_l: Loc { begin: 0, end: 1 },
                })))
            }
        }
    } else {
        *children = Some(Box::new(Node::Begin(Begin {
            statements: vec![],
            begin_l: None,
            end_l: None,
            expression_l: Loc { begin: 0, end: 1 },
        })))
    }
}

fn const_name_from_node(node: &Node) -> String {
    match node {
        Node::Const(constant) => {
            if let Some(scope) = &constant.scope {
                let mut base = const_name_from_node(&scope);
                base.push_str("::");
                base.push_str(constant.name.as_str());
                return base;
            }
            return constant.name.clone();
        }
        Node::Cbase(_) => {
            return String::from("");
        }
        _ => {
            panic!("Unexpected node {} as Constant Name", node.str_type())
        }
    }
}
