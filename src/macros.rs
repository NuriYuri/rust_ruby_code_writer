#[macro_export]
macro_rules! write_body {
    ($body: ident, $writer: ident, $indent: expr) => {
        if is_node_begin_block(&$body) {
            write_code(&$body, $writer, $indent)?;
        } else {
            write_indent($writer, $indent)?;
            write_code(&$body, $writer, $indent)?;
            $writer.write(b"\n")?;
        }
    };
}

#[macro_export]
macro_rules! write_array {
    ($array: ident, $writer: ident, $open: expr, $close: expr, $accessor: ident) => {
        if $array.begin_l.is_some() {
            $writer.write($open)?;
        }
        write_code_with_separator(&$array.$accessor, $writer, b", ")?;
        if $array.end_l.is_some() {
            $writer.write($close)?;
        }
    };
    ($array: ident, $writer: ident, $open: expr, $close: expr) => {
        write_array!($array, $writer, $open, $close, elements)
    };
}

#[macro_export]
macro_rules! write_range {
    ($range: ident, $writer: ident, $operator: expr) => {
        if let Some(left) = &$range.left {
            write_code(&left, $writer, 0)?;
        }
        $writer.write($operator)?;
        if let Some(right) = &$range.right {
            write_code(&right, $writer, 0)?;
        }
    };
}

#[macro_export]
macro_rules! write_def_name_arg_and_body {
    ($def: ident, $writer: ident, $indent: expr ) => {
        $writer.write($def.name.as_bytes())?;
        if let Some(args) = &$def.args {
            $writer.write(b"(")?;
            write_code(&args, $writer, 0)?;
            $writer.write(b")")?;
        }
        if $def.assignment_l.is_some() {
            $writer.write(b" = ")?;
            if let Some(body) = &$def.body {
                $writer.write(b"\n")?;
                write_code(&body, $writer, 0)?;
            }
        } else {
            if let Some(body) = &$def.body {
                $writer.write(b"\n")?;
                match body.as_ref() {
                    Node::Ensure(_) | Node::Rescue(_) => {
                        write_code(&body, $writer, $indent + 1)?;
                    }
                    _ => {
                        write_body!(body, $writer, $indent + 1);
                    }
                }
            }
            write_indent($writer, $indent)?;
            $writer.write(b"end")?;
        }
    };
}

#[macro_export]
macro_rules! write_assign {
    ($asgn: ident, $writer: ident, $indent: expr) => {
        $writer.write($asgn.name.as_bytes())?;
        if let Some(value) = &$asgn.value {
            $writer.write(b" = ")?;
            write_code(&value, $writer, $indent)?;
        }
    };
}

#[macro_export]
macro_rules! write_exe {
    ($exe: ident, $writer: ident, $indent: expr, $keyword_with_bracket: expr) => {
        $writer.write($keyword_with_bracket)?;
        if let Some(body) = &$exe.body {
            if is_node_begin_block(&body) {
                $writer.write(b"\n")?;
                write_code(&body, $writer, $indent + 1)?;
                $writer.write(b"\n")?;
            } else {
                write_code(&body, $writer, $indent + 1)?;
            }
        }
        $writer.write(b" }")?;
    };
}

#[macro_export]
macro_rules! write_until_while {
    ($control: ident, $writer: ident, $indent: expr, $keyword_with_space: expr) => {
        $writer.write($keyword_with_space)?;
        write_code(&$control.cond, $writer, 0)?;
        if let Some(body) = &$control.body {
            $writer.write(b"\n")?;
            write_body!(body, $writer, $indent + 1);
        }
        if $control.end_l.is_some() {
            write_indent($writer, $indent)?;
            $writer.write(b"end")?;
        }
    };
}

#[macro_export]
macro_rules! write_block_control_operator {
    ($control: ident, $writer: ident, $keyword: expr, $keyword_space: expr, $keyword_parent_open: expr) => {
        match $control.args.len() {
            0 => {
                $writer.write($keyword)?;
            }
            1 => {
                $writer.write($keyword_space)?;
                write_code(&$control.args[0], $writer, 0)?;
            }
            _ => {
                $writer.write($keyword_parent_open)?;
                write_code_with_separator(&$control.args, $writer, b", ")?;
                $writer.write(b")")?;
            }
        }
    };
}

#[macro_export]
macro_rules! write_body_with_end {
    ($node: ident, $writer: ident, $indent: expr) => {
        $writer.write(b"\n")?;
        if let Some(body) = &$node.body {
            write_body!(body, $writer, $indent + 1);
        }
        write_indent($writer, $indent)?;
        $writer.write(b"end")?;
    };
}
