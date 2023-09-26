use std::collections::HashMap;

use lib_ruby_parser::{
    nodes::{Arg, Args, Blockarg, Kwarg, Kwoptarg, Kwrestarg, Optarg, Restarg, Shadowarg},
    Node,
};

use crate::{
    create_arg, create_some_name_arg, edit_node_array_attr, edit_node_attr, edit_node_opt_attr,
};

pub fn edit_methods(node: &mut Box<Node>) -> () {
    let mut arg_name_to_new_arg: ArgsMap = HashMap::new();
    edit_node(&mut arg_name_to_new_arg, node.as_mut());
}

const ARGS_LIST: [&str; 26] = [
    "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s",
    "t", "u", "v", "w", "x", "y", "z",
];

type ArgsMap = HashMap<String, &'static str>;

fn edit_method(args: &mut Args, body: &mut Box<Node>) {
    let mut arg_name_to_new_arg: ArgsMap = HashMap::new();
    edit_args(&mut arg_name_to_new_arg, args);
    edit_node(&mut arg_name_to_new_arg, body.as_mut());
}

fn edit_args(arg_name_to_new_arg: &mut ArgsMap, args: &mut Args) {
    args.args = args
        .args
        .iter()
        .map(|node| match node {
            Node::Arg(arg) => Node::Arg(create_arg!(Arg, arg_name_to_new_arg, arg)),
            Node::Kwarg(arg) => Node::Kwarg(create_arg!(Kwarg, arg_name_to_new_arg, arg)),
            Node::Shadowarg(arg) => {
                Node::Shadowarg(create_arg!(Shadowarg, arg_name_to_new_arg, arg))
            }
            Node::Optarg(arg) => {
                let mut new_arg = create_arg!(Optarg, arg_name_to_new_arg, arg);
                edit_node(arg_name_to_new_arg, new_arg.default.as_mut());
                return Node::Optarg(new_arg);
            }
            Node::Kwoptarg(arg) => {
                let mut new_arg = create_arg!(Kwoptarg, arg_name_to_new_arg, arg);
                edit_node(arg_name_to_new_arg, new_arg.default.as_mut());
                return Node::Kwoptarg(new_arg);
            }
            Node::Restarg(arg) => {
                create_some_name_arg!(Restarg, arg_name_to_new_arg, arg, node);
            }
            Node::Kwrestarg(arg) => {
                create_some_name_arg!(Kwrestarg, arg_name_to_new_arg, arg, node);
            }
            Node::Blockarg(arg) => {
                create_some_name_arg!(Blockarg, arg_name_to_new_arg, arg, node);
            }
            node => node.to_owned(),
        })
        .collect();
}

fn resolve_or_assign_arg<'a>(arg_name_to_new_arg: &mut ArgsMap, arg: &'a String) -> &'a str {
    if arg_name_to_new_arg.contains_key(arg) {
        let &value = arg_name_to_new_arg.get(arg).unwrap();
        if value.len() == 0 {
            return arg.as_str();
        }
        return value;
    }
    let new_key = if arg_name_to_new_arg.len() < 26 {
        ARGS_LIST[arg_name_to_new_arg.len()]
    } else {
        &""
    };
    arg_name_to_new_arg.insert(arg.clone(), new_key);
    return new_key;
}

fn edit_node(arg_name_to_new_arg: &mut ArgsMap, node: &mut Node) -> () {
    match node {
        Node::And(node) => {
            edit_node_attr!(arg_name_to_new_arg, node, lhs);
            edit_node_attr!(arg_name_to_new_arg, node, rhs);
        }
        Node::AndAsgn(node) => {
            edit_node_attr!(arg_name_to_new_arg, node, recv);
            edit_node_attr!(arg_name_to_new_arg, node, value);
        }
        Node::Array(node) => {
            edit_node_array_attr!(arg_name_to_new_arg, node, elements);
        }
        Node::ArrayPattern(node) => {
            edit_node_array_attr!(arg_name_to_new_arg, node, elements);
        }
        Node::ArrayPatternWithTail(node) => {
            edit_node_array_attr!(arg_name_to_new_arg, node, elements);
        }
        Node::Begin(node) => {
            edit_node_array_attr!(arg_name_to_new_arg, node, statements);
        }
        Node::Block(node) => {
            edit_node_attr!(arg_name_to_new_arg, node, call);
            let mut new_args_map = arg_name_to_new_arg.clone();
            if let Some(Node::Args(args)) = node.args.as_deref_mut() {
                edit_args(&mut new_args_map, args);
            } else {
                edit_node_opt_attr!(&mut new_args_map, node, args);
            }
            edit_node_opt_attr!(&mut new_args_map, node, body);
        }
        Node::BlockPass(node) => {
            edit_node_opt_attr!(arg_name_to_new_arg, node, value);
        }
        Node::Break(node) => {
            edit_node_array_attr!(arg_name_to_new_arg, node, args);
        }
        Node::Case(node) => {
            edit_node_opt_attr!(arg_name_to_new_arg, node, expr);
            edit_node_array_attr!(arg_name_to_new_arg, node, when_bodies);
            edit_node_opt_attr!(arg_name_to_new_arg, node, else_body);
        }
        Node::CaseMatch(node) => {
            edit_node_attr!(arg_name_to_new_arg, node, expr);
            edit_node_array_attr!(arg_name_to_new_arg, node, in_bodies);
            edit_node_opt_attr!(arg_name_to_new_arg, node, else_body);
        }
        Node::Class(node) => {
            edit_node_opt_attr!(arg_name_to_new_arg, node, superclass);
            edit_node_opt_attr!(arg_name_to_new_arg, node, body);
        }
        Node::ConstPattern(node) => {
            edit_node_attr!(arg_name_to_new_arg, node, pattern);
        }
        Node::CSend(node) => {
            edit_node_attr!(arg_name_to_new_arg, node, recv);
            edit_node_array_attr!(arg_name_to_new_arg, node, args);
        }
        Node::Cvasgn(node) => {
            edit_node_opt_attr!(arg_name_to_new_arg, node, value);
        }
        Node::Def(node) => {
            if let Some(Node::Args(args)) = node.args.as_deref_mut() {
                if let Some(body) = node.body.as_mut() {
                    edit_method(args, body);
                }
            } else {
                let mut args_map: ArgsMap = HashMap::new();
                edit_node_opt_attr!(&mut args_map, node, body);
            }
        }
        Node::Defs(node) => {
            edit_node_attr!(arg_name_to_new_arg, node, definee);
            let mut args_map: ArgsMap = HashMap::new();
            if let Some(Node::Args(args)) = node.args.as_deref_mut() {
                edit_args(&mut args_map, args);
            } else {
                edit_node_opt_attr!(&mut args_map, node, args);
            }
            edit_node_opt_attr!(&mut args_map, node, body);
        }
        Node::Dstr(node) => {
            edit_node_array_attr!(arg_name_to_new_arg, node, parts);
        }
        Node::Dsym(node) => {
            edit_node_array_attr!(arg_name_to_new_arg, node, parts);
        }
        Node::EFlipFlop(node) => {
            edit_node_opt_attr!(arg_name_to_new_arg, node, right);
            edit_node_opt_attr!(arg_name_to_new_arg, node, left);
        }
        Node::Ensure(node) => {
            edit_node_opt_attr!(arg_name_to_new_arg, node, body);
            edit_node_opt_attr!(arg_name_to_new_arg, node, ensure);
        }
        Node::Erange(node) => {
            edit_node_opt_attr!(arg_name_to_new_arg, node, right);
            edit_node_opt_attr!(arg_name_to_new_arg, node, left);
        }
        Node::FindPattern(node) => {
            edit_node_array_attr!(arg_name_to_new_arg, node, elements);
        }
        Node::For(node) => {
            edit_node_attr!(arg_name_to_new_arg, node, iterator);
            edit_node_attr!(arg_name_to_new_arg, node, iteratee);
            edit_node_opt_attr!(arg_name_to_new_arg, node, body);
        }
        Node::Gvasgn(node) => {
            edit_node_opt_attr!(arg_name_to_new_arg, node, value);
        }
        Node::Hash(node) => {
            edit_node_array_attr!(arg_name_to_new_arg, node, pairs);
        }
        Node::HashPattern(node) => {
            edit_node_array_attr!(arg_name_to_new_arg, node, elements);
        }
        Node::Heredoc(node) => {
            edit_node_array_attr!(arg_name_to_new_arg, node, parts);
        }
        Node::If(node) => {
            edit_node_attr!(arg_name_to_new_arg, node, cond);
            edit_node_opt_attr!(arg_name_to_new_arg, node, if_true);
            edit_node_opt_attr!(arg_name_to_new_arg, node, if_false);
        }
        Node::IfGuard(node) => {
            edit_node_attr!(arg_name_to_new_arg, node, cond);
        }
        Node::IFlipFlop(node) => {
            edit_node_opt_attr!(arg_name_to_new_arg, node, right);
            edit_node_opt_attr!(arg_name_to_new_arg, node, left);
        }
        Node::IfMod(node) => {
            edit_node_attr!(arg_name_to_new_arg, node, cond);
            edit_node_opt_attr!(arg_name_to_new_arg, node, if_true);
            edit_node_opt_attr!(arg_name_to_new_arg, node, if_false);
        }
        Node::IfTernary(node) => {
            edit_node_attr!(arg_name_to_new_arg, node, cond);
            edit_node_attr!(arg_name_to_new_arg, node, if_true);
            edit_node_attr!(arg_name_to_new_arg, node, if_false);
        }
        Node::Index(node) => {
            edit_node_attr!(arg_name_to_new_arg, node, recv);
            edit_node_array_attr!(arg_name_to_new_arg, node, indexes);
        }
        Node::IndexAsgn(node) => {
            edit_node_attr!(arg_name_to_new_arg, node, recv);
            edit_node_array_attr!(arg_name_to_new_arg, node, indexes);
            edit_node_opt_attr!(arg_name_to_new_arg, node, value);
        }
        Node::InPattern(node) => {
            edit_node_attr!(arg_name_to_new_arg, node, pattern);
            edit_node_opt_attr!(arg_name_to_new_arg, node, guard);
            edit_node_opt_attr!(arg_name_to_new_arg, node, body);
        }
        Node::Irange(node) => {
            edit_node_opt_attr!(arg_name_to_new_arg, node, right);
            edit_node_opt_attr!(arg_name_to_new_arg, node, left);
        }
        Node::Ivasgn(node) => {
            edit_node_opt_attr!(arg_name_to_new_arg, node, value);
        }
        Node::Kwargs(node) => {
            edit_node_array_attr!(arg_name_to_new_arg, node, pairs);
        }
        Node::KwBegin(node) => {
            edit_node_array_attr!(arg_name_to_new_arg, node, statements);
        }
        Node::Kwsplat(node) => {
            edit_node_attr!(arg_name_to_new_arg, node, value);
        }
        Node::Lvar(node) => {
            node.name = String::from(resolve_or_assign_arg(arg_name_to_new_arg, &node.name));
        }
        Node::Lvasgn(node) => {
            node.name = String::from(resolve_or_assign_arg(arg_name_to_new_arg, &node.name)); // TODO: end_of_use optimization
            edit_node_opt_attr!(arg_name_to_new_arg, node, value);
        }
        Node::Masgn(node) => {
            edit_node_attr!(arg_name_to_new_arg, node, lhs);
            edit_node_attr!(arg_name_to_new_arg, node, rhs);
        }
        Node::MatchAlt(node) => {
            edit_node_attr!(arg_name_to_new_arg, node, lhs);
            edit_node_attr!(arg_name_to_new_arg, node, rhs);
        }
        Node::MatchAs(node) => {
            edit_node_attr!(arg_name_to_new_arg, node, value);
            edit_node_attr!(arg_name_to_new_arg, node, as_);
        }
        Node::MatchCurrentLine(node) => {
            edit_node_attr!(arg_name_to_new_arg, node, re);
        }
        Node::MatchPattern(node) => {
            edit_node_attr!(arg_name_to_new_arg, node, value);
            edit_node_attr!(arg_name_to_new_arg, node, pattern);
        }
        Node::MatchPatternP(node) => {
            edit_node_attr!(arg_name_to_new_arg, node, value);
            edit_node_attr!(arg_name_to_new_arg, node, pattern);
        }
        Node::MatchRest(node) => {
            edit_node_opt_attr!(arg_name_to_new_arg, node, name);
        }
        Node::MatchVar(node) => {
            arg_name_to_new_arg.insert(node.name.clone(), "");
        }
        Node::MatchWithLvasgn(node) => {
            edit_node_attr!(arg_name_to_new_arg, node, re);
            edit_node_attr!(arg_name_to_new_arg, node, value);
        }
        Node::Mlhs(node) => {
            edit_node_array_attr!(arg_name_to_new_arg, node, items);
        }
        Node::Module(node) => {
            edit_node_opt_attr!(arg_name_to_new_arg, node, body);
        }
        Node::Next(node) => {
            edit_node_array_attr!(arg_name_to_new_arg, node, args);
        }
        Node::OpAsgn(node) => {
            edit_node_attr!(arg_name_to_new_arg, node, recv);
            edit_node_attr!(arg_name_to_new_arg, node, value);
        }
        Node::Or(node) => {
            edit_node_attr!(arg_name_to_new_arg, node, lhs);
            edit_node_attr!(arg_name_to_new_arg, node, rhs);
        }
        Node::OrAsgn(node) => {
            edit_node_attr!(arg_name_to_new_arg, node, recv);
            edit_node_attr!(arg_name_to_new_arg, node, value);
        }
        Node::Pair(node) => {
            edit_node_attr!(arg_name_to_new_arg, node, key);
            edit_node_attr!(arg_name_to_new_arg, node, value);
        }
        Node::Pin(node) => {
            edit_node_attr!(arg_name_to_new_arg, node, var);
        }
        Node::Procarg0(node) => {
            edit_node_array_attr!(arg_name_to_new_arg, node, args);
        }
        Node::Regexp(node) => {
            edit_node_array_attr!(arg_name_to_new_arg, node, parts);
        }
        Node::Rescue(node) => {
            edit_node_opt_attr!(arg_name_to_new_arg, node, body);
            edit_node_array_attr!(arg_name_to_new_arg, node, rescue_bodies);
            edit_node_opt_attr!(arg_name_to_new_arg, node, else_);
        }
        Node::Return(node) => {
            edit_node_array_attr!(arg_name_to_new_arg, node, args);
        }
        Node::Send(node) => {
            edit_node_opt_attr!(arg_name_to_new_arg, node, recv);
            edit_node_array_attr!(arg_name_to_new_arg, node, args);
        }
        Node::Splat(node) => {
            edit_node_opt_attr!(arg_name_to_new_arg, node, value);
        }
        Node::Super(node) => {
            edit_node_array_attr!(arg_name_to_new_arg, node, args);
        }
        Node::UnlessGuard(node) => {
            edit_node_attr!(arg_name_to_new_arg, node, cond);
        }
        Node::Until(node) => {
            edit_node_attr!(arg_name_to_new_arg, node, cond);
            edit_node_opt_attr!(arg_name_to_new_arg, node, body);
        }
        Node::UntilPost(node) => {
            edit_node_attr!(arg_name_to_new_arg, node, cond);
            edit_node_attr!(arg_name_to_new_arg, node, body);
        }
        Node::When(node) => {
            edit_node_array_attr!(arg_name_to_new_arg, node, patterns);
            edit_node_opt_attr!(arg_name_to_new_arg, node, body);
        }
        Node::While(node) => {
            edit_node_attr!(arg_name_to_new_arg, node, cond);
            edit_node_opt_attr!(arg_name_to_new_arg, node, body);
        }
        Node::WhilePost(node) => {
            edit_node_attr!(arg_name_to_new_arg, node, cond);
            edit_node_attr!(arg_name_to_new_arg, node, body);
        }
        Node::XHeredoc(node) => {
            edit_node_array_attr!(arg_name_to_new_arg, node, parts);
        }
        Node::Xstr(node) => {
            edit_node_array_attr!(arg_name_to_new_arg, node, parts);
        }
        Node::Yield(node) => {
            edit_node_array_attr!(arg_name_to_new_arg, node, args);
        }
        _ => {}
    }
}
