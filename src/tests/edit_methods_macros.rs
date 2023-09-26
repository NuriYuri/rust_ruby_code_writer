#[macro_export]
macro_rules! edit_node_attr {
    ($args_map: expr, $node: expr, $attr: ident) => {
        edit_node($args_map, &mut $node.$attr);
    };
}

#[macro_export]
macro_rules! edit_node_opt_attr {
    ($args_map: expr, $node: expr, $attr: ident) => {
        if let Some($attr) = &mut $node.$attr {
            edit_node($args_map, $attr);
        }
    };
}

#[macro_export]
macro_rules! edit_node_array_attr {
    ($args_map: expr, $node: expr, $attr: ident) => {
        for element in $node.$attr.iter_mut() {
            edit_node($args_map, element);
        }
    };
}

#[macro_export]
macro_rules! create_arg {
    ($arg_type: ident, $args_map: expr, $arg: expr) => {
        $arg_type {
            name: String::from(resolve_or_assign_arg($args_map, &$arg.name)),
            ..$arg.clone()
        }
    };
}

#[macro_export]
macro_rules! create_some_name_arg {
    ($arg_type: ident, $args_map: expr, $arg: expr, $node: expr) => {
        if let Some(name) = $arg.name.as_ref() {
            return Node::$arg_type($arg_type {
                name: Some(String::from(resolve_or_assign_arg($args_map, &name))),
                ..$arg.clone()
            });
        }
        return $node.to_owned();
    };
}
