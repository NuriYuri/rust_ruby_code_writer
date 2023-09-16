#[macro_export]
macro_rules! write_body {
    ($body: ident, $writer: ident, $indent: expr) => {
        if is_node_begin_block(&$body) {
            write_code($body, $writer, $indent)?;
        } else {
            write_indent($writer, $indent)?;
            write_code($body, $writer, $indent)?;
            $writer.write(b"\n")?;
        }
    };
}
