use std::{collections::HashMap, ops::Deref, rc::Rc};

use lib_ruby_parser::{
    nodes::{Casgn, Send},
    Node,
};

#[derive(Debug)]
pub enum ConstantHashValue {
    EndValue(Node),
    ModuleValue(ConstantHashMap),
}

type ConstantHashMap = HashMap<Rc<str>, ConstantHashValue>;
type SendHandler = dyn Fn(&Send) -> Option<Box<Node>>;

// impl std::fmt::Debug for ConstantHashMap {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let mut debug_struct = f.debug_struct("HashMap");
//         for (key, val) in self.iter() {
//             debug_struct.field(key, val);
//         }
//         debug_struct.finish()
//     }
// }

pub fn make_constant_hash_map() -> ConstantHashMap {
    return HashMap::new();
}

pub fn explore_constants(
    top_level_or_current_module: &mut ConstantHashMap,
    node: &Node,
    send_handler: &SendHandler,
) -> () {
    match node {
        Node::Module(module) => {
            explore_module_or_class(
                top_level_or_current_module,
                &module.name,
                &module.body,
                send_handler,
            );
        }
        Node::Class(class) => {
            explore_module_or_class(
                top_level_or_current_module,
                &class.name,
                &class.body,
                send_handler,
            );
        }
        _ => {
            explore_body(top_level_or_current_module, node, send_handler);
        }
    }
}

fn explore_module_or_class(
    current_module: &mut ConstantHashMap,
    name: &Node,
    body: &Option<Box<Node>>,
    send_handler: &SendHandler,
) -> () {
    let explored_module_name = constant_to_rc_str(name);
    if !current_module.contains_key(&explored_module_name) {
        current_module.insert(
            explored_module_name.clone(),
            ConstantHashValue::ModuleValue(make_constant_hash_map()),
        );
    }
    if let Some(body) = body {
        if let Some(ConstantHashValue::ModuleValue(map)) =
            current_module.get_mut(&explored_module_name)
        {
            explore_body(map, &body, send_handler);
        };
    };
    // Remove empty module after exploration
    if let Some(ConstantHashValue::ModuleValue(map)) = current_module.get(&explored_module_name) {
        if map.is_empty() {
            current_module.remove(&explored_module_name);
        }
    }
}

fn explore_body(map: &mut ConstantHashMap, body: &Node, send_handler: &SendHandler) -> () {
    match body {
        Node::Casgn(asgn) => {
            handle_casgn(map, asgn, send_handler);
        }
        Node::Module(_) | Node::Class(_) => {
            explore_constants(map, body, send_handler);
        }
        Node::Begin(begin) => {
            for node in begin.statements.iter() {
                explore_body(map, node, send_handler);
            }
        }
        _ => {}
    }
}

fn handle_casgn(map: &mut ConstantHashMap, asgn: &Casgn, send_handler: &SendHandler) -> () {
    if let Some(value) = &asgn.value {
        match value.as_ref() {
            Node::Int(_)
            //| Node::Str(_)
            | Node::Sym(_)
            | Node::Float(_)
            | Node::Nil(_)
            | Node::True(_)
            | Node::False(_) => {
                map.insert(
                    constant_to_rc_str(&Node::Casgn(asgn.clone())),
                    ConstantHashValue::EndValue(value.as_ref().clone()),
                );
            }
            Node::Send(send) => {
                let new_asgn = Casgn { value: send_handler(send), ..asgn.clone() };
                handle_casgn(map, &new_asgn, send_handler);
            }
            _ => {}
        }
    }
}

fn constant_to_rc_str(constant_node: &Node) -> Rc<str> {
    match constant_node {
        Node::Const(constant) => {
            return get_constant_name(&constant.scope, &constant.name);
        }
        Node::Casgn(asgn) => {
            return get_constant_name(&asgn.scope, &asgn.name);
        }
        _ => {
            return "".into();
        }
    }
}

fn get_constant_name(scope: &Option<Box<Node>>, constant_name: &String) -> Rc<str> {
    if let Some(scope) = &scope {
        let mut name = String::from(constant_to_rc_str(&scope).deref());
        name.push_str("::");
        name.push_str(constant_name.as_str());
        return name.as_str().into();
    }
    return constant_name.as_str().into();
}
