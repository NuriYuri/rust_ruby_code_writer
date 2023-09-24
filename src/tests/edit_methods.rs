use std::collections::HashMap;

use lib_ruby_parser::{
    nodes::{Arg, Args, Kwarg, Kwoptarg, Kwrestarg, Optarg, Restarg},
    Node,
};

pub fn edit_methods(node: &mut Box<Node>) -> () {
    match node.as_mut() {
        Node::Def(def) => {
            if let Some(Node::Args(args)) = def.args.as_deref_mut() {
                if let Some(body) = def.body.as_mut() {
                    edit_method(args, body);
                }
            }
        }
        _ => {}
    }
}

const ARGS_LIST: [&str; 26] = [
    "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s",
    "t", "u", "v", "w", "x", "y", "z",
];

type ArgsMap = HashMap<String, &'static str>;

fn edit_method(args: &mut Args, body: &mut Box<Node>) {
    let mut arg_name_to_new_arg: ArgsMap = HashMap::new();
    args.args = args
        .args
        .iter()
        .map(|node| match node {
            Node::Arg(arg) => Node::Arg(Arg {
                name: String::from(resolve_or_assign_arg(&mut arg_name_to_new_arg, &arg.name)),
                ..arg.clone()
            }),
            Node::Kwarg(arg) => Node::Kwarg(Kwarg {
                name: String::from(resolve_or_assign_arg(&mut arg_name_to_new_arg, &arg.name)),
                ..arg.clone()
            }),
            Node::Optarg(arg) => Node::Optarg(Optarg {
                name: String::from(resolve_or_assign_arg(&mut arg_name_to_new_arg, &arg.name)),
                ..arg.clone()
            }),
            Node::Kwoptarg(arg) => Node::Kwoptarg(Kwoptarg {
                name: String::from(resolve_or_assign_arg(&mut arg_name_to_new_arg, &arg.name)),
                ..arg.clone()
            }),
            Node::Restarg(arg) => {
                if let Some(name) = arg.name.as_ref() {
                    return Node::Restarg(Restarg {
                        name: Some(String::from(resolve_or_assign_arg(
                            &mut arg_name_to_new_arg,
                            &name,
                        ))),
                        ..arg.clone()
                    });
                }
                return node.to_owned();
            }
            Node::Kwrestarg(arg) => {
                if let Some(name) = arg.name.as_ref() {
                    return Node::Kwrestarg(Kwrestarg {
                        name: Some(String::from(resolve_or_assign_arg(
                            &mut arg_name_to_new_arg,
                            &name,
                        ))),
                        ..arg.clone()
                    });
                }
                return node.to_owned();
            }
            node => node.to_owned(),
        })
        .collect()
}

fn resolve_or_assign_arg(arg_name_to_new_arg: &mut ArgsMap, arg: &String) -> &'static str {
    if arg_name_to_new_arg.contains_key(arg) {
        return arg_name_to_new_arg.get(arg).as_deref().unwrap();
    }
    let new_key = if arg_name_to_new_arg.len() < 26 {
        ARGS_LIST[arg_name_to_new_arg.len()]
    } else {
        ARGS_LIST[25]
    };
    arg_name_to_new_arg.insert(arg.clone(), new_key);
    return new_key;
}
