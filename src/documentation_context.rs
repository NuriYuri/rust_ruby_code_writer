use std::io::{BufWriter, Write};

use lib_ruby_parser::source::{Comment, DecodedInput};

pub struct DocumentationContext {
    comments: Vec<Comment>,
    input: DecodedInput,
    exclude_method_body: bool,
}

impl DocumentationContext {
    pub fn new(comments: Vec<Comment>, input: DecodedInput, exclude_method_body: bool) -> Self {
        return DocumentationContext {
            comments,
            input,
            exclude_method_body,
        };
    }

    pub fn write_documentation<W: Write>(
        &self,
        writer: &mut BufWriter<W>,
        indent: u32,
        node_expression_begin: usize,
    ) -> Result<(), std::io::Error> {
        let indent_offset = (indent * 2) as usize;
        let last_comment_expression_end: usize = node_expression_begin - indent_offset;
        let last_comment = self
            .comments
            .iter()
            .find(|c| c.location.end == last_comment_expression_end);
        if let Some(last_comment) = last_comment {
            let mut first_comment = last_comment;
            let mut last_comment_expression_end = first_comment.location.begin - indent_offset;
            while let Some(previous_comment) = self
                .comments
                .iter()
                .find(|c| c.location.end == last_comment_expression_end)
            {
                first_comment = previous_comment;
                last_comment_expression_end = first_comment.location.begin - indent_offset;
            }
            let first_index = self
                .comments
                .iter()
                .position(|c| c == first_comment)
                .unwrap();
            let last_index = self
                .comments
                .iter()
                .position(|c| c == last_comment)
                .unwrap();
            for index in first_index..=last_index {
                let comment = self.comments.get(index).unwrap();
                writer.write(&self.input.bytes[comment.location.begin..comment.location.end])?;
                if indent > 0 {
                    writer.write(b" ".repeat(indent_offset)[..].as_ref())?;
                }
            }
        }

        return Ok(());
    }

    pub fn method_body_excluded(&self) -> bool {
        self.exclude_method_body
    }
}
