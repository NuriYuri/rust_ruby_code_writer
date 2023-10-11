#[macro_export]
macro_rules! write_body {
    ($body: ident, $writer: ident, $context: expr) => {
        if is_node_begin_block(&$body) {
            write_code(&$body, $writer, &$context)?;
        } else {
            write_indent($writer, $context.indent)?;
            write_code(&$body, $writer, &$context)?;
            $writer.write(b"\n")?;
        }
    };
}

#[macro_export]
macro_rules! write_array {
    ($array: ident, $writer: ident, $open: expr, $close: expr, $accessor: ident, $context: expr) => {
        if $array.begin_l.is_some() {
            $writer.write($open)?;
        }
        write_code_with_separator(&$array.$accessor, $writer, &$context, b", ")?;
        if $array.end_l.is_some() {
            $writer.write($close)?;
        }
    };
    ($array: ident, $writer: ident, $open: expr, $close: expr, $context: expr) => {
        write_array!($array, $writer, $open, $close, elements, $context)
    };
}

#[macro_export]
macro_rules! write_range {
    ($range: ident, $writer: ident, $operator: expr, $context: expr) => {
        if let Some(left) = &$range.left {
            write_code(&left, $writer, &$context)?;
        }
        $writer.write($operator)?;
        if let Some(right) = &$range.right {
            write_code(&right, $writer, &$context)?;
        }
    };
}

#[macro_export]
macro_rules! write_def_name_arg_and_body {
    ($def: ident, $writer: ident, $context: expr) => {
        $writer.write($def.name.as_bytes())?;
        if let Some(args) = &$def.args {
            $writer.write(b"(")?;
            write_code(&args, $writer, &$context)?;
            $writer.write(b")")?;
        }
        if $def.assignment_l.is_some() {
            $writer.write(b" = ")?;
            if let Some(body) = &$def.body {
                $writer.write(b"\n")?;
                write_code(&body, $writer, &$context)?;
            }
        } else {
            if let Some(body) = &$def.body {
                $writer.write(b"\n")?;
                match body.as_ref() {
                    Node::Ensure(_) | Node::Rescue(_) => {
                        write_code(&body, $writer, &$context.indent())?;
                    }
                    _ => {
                        write_body!(body, $writer, $context.indent());
                    }
                }
            } else {
                $writer.write(b"\n")?;
            }
            write_indent($writer, $context.indent)?;
            $writer.write(b"end")?;
        }
    };
}

#[macro_export]
macro_rules! write_assign {
    ($asgn: ident, $writer: ident, $context: expr) => {
        $writer.write($asgn.name.as_bytes())?;
        if let Some(value) = &$asgn.value {
            $writer.write(b" = ")?;
            write_code(&value, $writer, &$context)?;
        }
    };
}

#[macro_export]
macro_rules! write_exe {
    ($exe: ident, $writer: ident, $context: expr, $keyword_with_bracket: expr) => {
        $writer.write($keyword_with_bracket)?;
        if let Some(body) = &$exe.body {
            if is_node_begin_block(&body) {
                $writer.write(b"\n")?;
                write_code(&body, $writer, &$context.indent())?;
                $writer.write(b"\n")?;
            } else {
                write_code(&body, $writer, &$context.indent())?;
            }
        }
        $writer.write(b" }")?;
    };
}

#[macro_export]
macro_rules! write_until_while {
    ($control: ident, $writer: ident, $context: expr, $keyword_with_space: expr) => {
        if $control.end_l.is_some() {
            $writer.write($keyword_with_space)?;
            write_code(&$control.cond, $writer, &$context)?;
            if let Some(body) = &$control.body {
                $writer.write(b"\n")?;
                write_body!(body, $writer, $context.indent());
            }
            write_indent($writer, $context.indent)?;
            $writer.write(b"end")?;
        } else {
            if let Some(body) = &$control.body {
                write_code(body, $writer, &$context)?;
                $writer.write(b" ")?;
            }
            $writer.write($keyword_with_space)?;
            write_code(&$control.cond, $writer, &$context)?;
        }
    };
}

#[macro_export]
macro_rules! write_block_control_operator {
    ($control: ident, $writer: ident, $keyword: expr, $keyword_parent_open: expr, $context: expr) => {
        match $control.args.len() {
            0 => {
                $writer.write($keyword)?;
            }
            _ => {
                $writer.write($keyword_parent_open)?;
                write_code_with_separator(&$control.args, $writer, &$context, b", ")?;
                $writer.write(b")")?;
            }
        }
    };
}

#[macro_export]
macro_rules! write_body_with_end {
    ($node: ident, $writer: ident, $context: expr) => {
        $writer.write(b"\n")?;
        if let Some(body) = &$node.body {
            write_body!(body, $writer, $context.indent());
        }
        write_indent($writer, $context.indent)?;
        $writer.write(b"end")?;
    };
}

#[macro_export]
macro_rules! write_documentation {
    ($node: ident, $writer: ident, $context: expr) => {
        if let Some(documentation_context) = &$context.documentation_context {
            documentation_context.write_documentation($writer, $context.indent, $node.expression_l.begin)?;
        }
    };
}