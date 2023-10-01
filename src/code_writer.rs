use std::io::{BufWriter, Write};

use lib_ruby_parser::{Bytes, Node};

use crate::{
    write_array, write_assign, write_block_control_operator, write_body, write_body_with_end,
    write_def_name_arg_and_body, write_exe, write_range, write_until_while,
};

pub struct CodeWriterContext {
    pub parent_node_type: &'static str,
    pub indent: u32,
}

impl CodeWriterContext {
    pub fn new() -> Self {
        return CodeWriterContext {
            parent_node_type: "none",
            indent: 0,
        };
    }

    fn make_child(&self, node_type: &'static str) -> Self {
        return CodeWriterContext {
            parent_node_type: node_type,
            indent: self.indent,
        };
    }

    fn indent(&self) -> Self {
        return CodeWriterContext {
            parent_node_type: self.parent_node_type,
            indent: self.indent + 1,
        };
    }

    fn outdent(&self) -> Self {
        return CodeWriterContext {
            parent_node_type: self.parent_node_type,
            indent: self.indent - 1,
        };
    }
}

pub fn write_code<W: Write>(
    node: &Node,
    writer: &mut BufWriter<W>,
    context: &CodeWriterContext,
) -> Result<(), std::io::Error> {
    let child_context = context.make_child(node.str_type());
    match node {
        Node::Alias(alias) => {
            writer.write(b"alias ")?;
            write_code(&alias.to, writer, &child_context)?;
            writer.write(b" ")?;
            write_code(&alias.from, writer, &child_context)?;
        }
        Node::And(and) => {
            write_code(&and.lhs, writer, &child_context)?;
            writer.write(b" && ")?;
            write_code(&and.rhs, writer, &child_context)?;
        }
        Node::AndAsgn(asgn) => {
            write_code(&asgn.recv, writer, &child_context)?;
            writer.write(b" &&= ")?;
            write_code(&asgn.value, writer, &child_context)?;
        }
        Node::Arg(arg) => {
            writer.write(arg.name.as_bytes())?;
        }
        Node::Args(args) => {
            write_code_with_separator(&args.args, writer, &child_context, b", ")?;
        }
        Node::Array(arr) => {
            if let Some(begin) = &arr.begin_l {
                if begin.size() == 3 {
                    if arr.elements.iter().any(|node| match node {
                        Node::Dsym(_) => true,
                        _ => false,
                    }) {
                        writer.write(b"%I[")?;
                        write_code_with_separator(&arr.elements, writer, &child_context, b" ")?;
                        writer.write(b"]")?;
                        return Ok(());
                    } else if let Some(Node::Sym(_)) = arr.elements.get(0) {
                        writer.write(b"%i[")?;
                        write_code_with_separator(&arr.elements, writer, &child_context, b" ")?;
                        writer.write(b"]")?;
                        return Ok(());
                    }
                }
            }
            write_array!(arr, writer, b"[", b"]", child_context);
        }
        Node::ArrayPattern(arr) => {
            write_array!(arr, writer, b"[", b"]", child_context);
        }
        Node::ArrayPatternWithTail(arr) => {
            write_array!(arr, writer, b"[", b",]", child_context);
        }
        Node::BackRef(back_ref) => {
            writer.write(back_ref.name.as_bytes())?;
        }
        Node::Begin(begin) => {
            if context.parent_node_type == "part" {
                write_code_with_separator(&begin.statements, writer, &child_context, b";")?;
            } else if begin.begin_l.is_some() {
                writer.write(b"(")?;
                write_code_with_separator(&begin.statements, writer, &child_context, b"; ")?;
                writer.write(b")")?;
            } else {
                write_code_without_separator(&begin.statements, writer, &child_context)?;
            }
        }
        Node::Block(block) => {
            write_code(&block.call, writer, &child_context)?;
            let do_block = block.begin_l.size() == 2;
            match &block.args {
                Some(args) => {
                    writer.write(if do_block { b" do |" } else { b" { |" })?;
                    write_code(&args, writer, &child_context)?;
                    writer.write(if do_block { b"|\n" } else { b"| " })?;
                }
                None => {
                    writer.write(if do_block { b" do\n" } else { b" {" })?;
                }
            }
            if let Some(body) = &block.body {
                if do_block {
                    write_body!(body, writer, &child_context.indent());
                } else {
                    write_code(&body, writer, &child_context.indent())?;
                }
            }
            if do_block {
                write_indent(writer, context.indent)?;
                writer.write(b"end")?;
            } else {
                writer.write(b" }")?;
            }
        }
        Node::BlockPass(pass) => {
            writer.write(b"&")?;
            if let Some(value) = pass.value.as_ref() {
                write_code(value, writer, &child_context)?;
            }
        }
        Node::Blockarg(arg) => {
            writer.write(b"&")?;
            if let Some(name) = arg.name.as_ref() {
                writer.write(name.as_bytes())?;
            }
        }
        Node::Break(control) => {
            write_block_control_operator!(
                control,
                writer,
                b"break",
                b"break ",
                b"break(",
                child_context
            )
        }
        Node::CSend(send) => {
            write_code(&send.recv, writer, &child_context)?;
            writer.write(b"&.")?;
            match send.operator_l {
                Some(_) => {
                    writer.write(send.method_name[0..(send.method_name.len() - 1)].as_bytes())?;
                    writer.write(b" = ")?;
                    write_code_with_separator(&send.args, writer, &child_context, b", ")?;
                }
                None => {
                    writer.write(send.method_name.as_bytes())?;
                    if send.begin_l.is_some() {
                        writer.write(b"(")?;
                    }
                    write_code_with_separator(&send.args, writer, &child_context, b", ")?;
                    if send.end_l.is_some() {
                        writer.write(b")")?;
                    }
                }
            }
        }
        Node::Case(case) => {
            match &case.expr {
                Some(expr) => {
                    writer.write(b"case ")?;
                    write_code(&expr, writer, &child_context)?;
                    writer.write(b"\n")?;
                }
                None => {
                    writer.write(b"case\n")?;
                }
            }
            for body in case.when_bodies.iter() {
                write_indent(writer, context.indent)?;
                write_code(body, writer, &child_context)?;
            }
            match &case.else_body {
                Some(else_body) => {
                    write_indent(writer, context.indent)?;
                    writer.write(b"else\n")?;
                    write_body!(else_body, writer, &child_context.indent());
                }
                None => {}
            }
            write_indent(writer, context.indent)?;
            writer.write(b"end")?;
        }
        Node::CaseMatch(case) => {
            writer.write(b"case ")?;
            write_code(&case.expr, writer, &child_context)?;
            writer.write(b"\n")?;
            write_code_without_separator(&case.in_bodies, writer, &child_context.indent())?;
            match &case.else_body {
                Some(else_body) => {
                    writer.write(b"else\n")?;
                    write_code(&else_body, writer, &child_context.indent())?;
                    writer.write(b"\n")?;
                }
                None => {}
            }
            write_indent(writer, context.indent)?;
            writer.write(b"end")?;
        }
        Node::Casgn(asgn) => {
            if let Some(scope) = &asgn.scope {
                write_code(&scope, writer, &child_context)?;
            }
            write_assign!(asgn, writer, &child_context);
        }
        Node::Cbase(_) => {
            writer.write(b"::")?;
        }
        Node::Class(class) => {
            writer.write(b"class ")?;
            write_code(&class.name, writer, &child_context)?;
            if let Some(super_class) = &class.superclass {
                writer.write(b" < ")?;
                write_code(&super_class, writer, &child_context)?;
            }
            write_body_with_end!(class, writer, child_context);
        }
        Node::Complex(complex) => {
            writer.write(complex.value.as_bytes())?;
        }
        Node::Const(constant) => {
            if let Some(scope) = &constant.scope {
                write_code(&scope, writer, &child_context)?;
            }
            if constant.double_colon_l.is_some() {
                writer.write(b"::")?;
            }
            writer.write(constant.name.as_bytes())?;
        }
        Node::ConstPattern(constant) => {
            write_code(&constant.const_, writer, &child_context)?;
            writer.write(b"(")?;
            write_code(&constant.pattern, writer, &child_context)?;
            writer.write(b")")?;
        }
        Node::Cvar(var) => {
            writer.write(var.name.as_bytes())?;
        }
        Node::Cvasgn(asgn) => {
            write_assign!(asgn, writer, &child_context);
        }
        Node::Def(def) => {
            writer.write(b"def ")?;
            write_def_name_arg_and_body!(def, writer, &child_context);
        }
        Node::Defined(defined) => {
            writer.write(b"defined?")?;
            if defined.begin_l.is_some() {
                writer.write(b"(")?;
            }
            write_code(&defined.value, writer, &child_context)?;
            if defined.end_l.is_some() {
                writer.write(b")")?;
            }
        }
        Node::Defs(def) => {
            writer.write(b"def ")?;
            write_code(&def.definee, writer, &child_context)?;
            writer.write(b".")?;
            write_def_name_arg_and_body!(def, writer, &child_context);
        }
        Node::Dstr(str) => {
            if let Some(begin) = &str.begin_l {
                if begin.size() == 1 {
                    writer.write(b"\"")?;
                    write_parts(writer, &str.parts, &child_context, "\"", "\\\"")?;
                    writer.write(b"\"")?;
                } else {
                    writer.write(b"%Q{")?;
                    write_parts(writer, &str.parts, &child_context, "}", "\\}")?;
                    writer.write(b"}")?;
                }
            } else {
                writer.write(b"\"")?;
                write_parts(writer, &str.parts, &child_context, "\"", "\\\"")?;
                writer.write(b"\"")?;
            }
        }
        Node::Dsym(sym) => {
            if context.parent_node_type == "pair_key" {
                writer.write(b"\"")?;
                write_parts(writer, &sym.parts, &child_context, "\"", "\\\"")?;
                writer.write(b"\"")?;
            } else {
                if sym.begin_l.is_some() {
                    writer.write(b":\"")?;
                }
                write_parts(writer, &sym.parts, &child_context, "\"", "\\\"")?;
                if sym.end_l.is_some() {
                    writer.write(b"\"")?;
                }
            }
        }
        Node::EFlipFlop(flip_flop) => {
            write_range!(flip_flop, writer, b"...", child_context);
        }
        Node::EmptyElse(_) => {
            writer.write(b"else")?;
        }
        Node::Encoding(_) => {
            writer.write(b"__ENCODING__")?;
        }
        Node::Ensure(ensure) => {
            if let Some(body) = &ensure.body {
                match body.as_ref() {
                    Node::Rescue(_) => {
                        write_code(&body, writer, &child_context)?;
                    }
                    _ => {
                        write_body!(body, writer, child_context);
                    }
                }
            }
            write_indent(writer, context.indent - 1)?; // <= Ensure is part of the body of something else
            writer.write(b"ensure\n")?;
            if let Some(ensure) = &ensure.ensure {
                write_body!(ensure, writer, child_context);
            }
        }
        Node::Erange(range) => {
            write_range!(range, writer, b"...", child_context);
        }
        Node::False(_) => {
            writer.write(b"false")?;
        }
        Node::File(_) => {
            writer.write(b"__FILE__")?;
        }
        Node::FindPattern(pat) => {
            write_array!(pat, writer, b"[", b"]", child_context);
        }
        Node::Float(float) => {
            writer.write(float.value.as_bytes())?;
        }
        Node::For(for_kw) => {
            writer.write(b"for ")?;
            write_code(&for_kw.iterator, writer, &child_context)?;
            writer.write(b" in ")?;
            write_code(&for_kw.iteratee, writer, &child_context)?;
            write_body_with_end!(for_kw, writer, child_context);
        }
        Node::ForwardArg(_) | Node::ForwardedArgs(_) => {
            writer.write(b"...")?;
        }
        Node::Gvar(var) => {
            writer.write(var.name.as_bytes())?;
        }
        Node::Gvasgn(asgn) => {
            write_assign!(asgn, writer, child_context);
        }
        Node::Hash(hash) => {
            write_array!(hash, writer, b"{", b"}", pairs, child_context);
        }
        Node::HashPattern(pat) => {
            write_array!(pat, writer, b"{", b"}", child_context);
        }
        Node::Heredoc(doc) => {
            writer.write(b"\"")?;
            write_parts(writer, &doc.parts, &child_context, "\"", "\\\"")?;
            writer.write(b"\"")?;
        }
        Node::IFlipFlop(flip_flop) => {
            write_range!(flip_flop, writer, b"..", context);
        }
        Node::If(if_kw) => {
            let indented_context = child_context.indent();
            if let Some(if_true) = &if_kw.if_true {
                writer.write(b"if ")?;
                write_code(&if_kw.cond, writer, &child_context)?;
                writer.write(b"\n")?;
                write_body!(if_true, writer, indented_context);
                if let Some(if_false) = &if_kw.if_false {
                    write_indent(writer, context.indent)?;
                    writer.write(b"else\n")?;
                    write_body!(if_false, writer, indented_context);
                }
            } else {
                if let Some(if_false) = &if_kw.if_false {
                    writer.write(b"unless ")?;
                    write_code(&if_kw.cond, writer, &child_context)?;
                    writer.write(b"\n")?;
                    write_body!(if_false, writer, indented_context);
                }
            }
            write_indent(writer, context.indent)?;
            writer.write(b"end")?;
        }
        Node::IfGuard(guard) => {
            writer.write(b" if ")?;
            write_code(&guard.cond, writer, &child_context)?;
        }
        Node::IfMod(guard) => {
            if let Some(code) = &guard.if_true {
                write_code(&code, writer, &child_context)?;
                writer.write(b" if ")?;
            }
            if let Some(code) = &guard.if_false {
                write_code(&code, writer, &child_context)?;
                writer.write(b" unless ")?;
            }
            write_code(&guard.cond, writer, &child_context)?;
        }
        Node::IfTernary(ternary) => {
            write_code(&ternary.cond, writer, &child_context)?;
            writer.write(b" ? ")?;
            write_code(&ternary.if_true, writer, &child_context)?;
            writer.write(b" : ")?;
            write_code(&ternary.if_false, writer, &child_context)?;
        }
        Node::InPattern(pat) => {
            writer.write(b"in ")?;
            write_code(&pat.pattern, writer, &child_context)?;
            if let Some(guard) = &pat.guard {
                write_code(&guard, writer, &child_context)?;
            }
            if let Some(body) = &pat.body {
                writer.write(b" then\n")?;
                write_body!(body, writer, child_context.indent());
            }
        }
        Node::Index(index) => {
            write_code(&index.recv, writer, &child_context)?;
            writer.write(b"[")?;
            write_code_with_separator(&index.indexes, writer, &child_context, b", ")?;
            writer.write(b"]")?;
        }
        Node::IndexAsgn(asgn) => {
            write_code(&asgn.recv, writer, &child_context)?;
            writer.write(b"[")?;
            write_code_with_separator(&asgn.indexes, writer, &child_context, b", ")?;
            writer.write(b"]")?;
            if let Some(value) = &asgn.value {
                writer.write(b" = ")?;
                write_code(&value, writer, &child_context)?;
            }
        }
        Node::Int(value) => {
            writer.write(value.value.as_bytes())?;
        }
        Node::Irange(range) => {
            write_range!(range, writer, b"..", child_context);
        }
        Node::Ivar(var) => {
            writer.write(var.name.as_bytes())?;
        }
        Node::Ivasgn(asgn) => {
            write_assign!(asgn, writer, &child_context);
        }
        Node::KwBegin(kw_begin) => {
            if kw_begin.begin_l.is_some() {
                writer.write(b"begin\n")?;
            }
            //write_code_without_separator(kw_begin.statements, writer, indent + 1)?;
            for body in kw_begin.statements.iter() {
                match body {
                    Node::Ensure(_) | Node::Rescue(_) => {
                        write_code(body, writer, &child_context.indent())?;
                    }
                    _ => {
                        write_body!(body, writer, child_context.indent());
                    }
                }
            }
            if kw_begin.end_l.is_some() {
                write_indent(writer, context.indent)?;
                writer.write(b"end")?;
            }
        }
        Node::Kwarg(arg) => {
            writer.write(arg.name.as_bytes())?;
            writer.write(b":")?;
        }
        Node::Kwargs(args) => {
            write_code_with_separator(&args.pairs, writer, &child_context, b", ")?;
        }
        Node::Kwnilarg(_) | Node::MatchNilPattern(_) => {
            writer.write(b"**nil")?;
        }
        Node::Kwoptarg(arg) => {
            writer.write(arg.name.as_bytes())?;
            writer.write(b": ")?;
            write_code(&arg.default, writer, &child_context)?;
        }
        Node::Kwrestarg(arg) => {
            writer.write(b"**")?;
            if let Some(name) = &arg.name {
                writer.write(name.as_bytes())?;
            }
        }
        Node::Kwsplat(splat) => {
            writer.write(b"**")?;
            write_code(&splat.value, writer, &child_context)?;
        }
        Node::Lambda(_) => {
            writer.write(b"->")?;
        }
        Node::Line(_) => {
            writer.write(b"__LINE__")?;
        }
        Node::Lvar(var) => {
            writer.write(var.name.as_bytes())?;
        }
        Node::Lvasgn(asgn) => {
            write_assign!(asgn, writer, &child_context);
        }
        Node::Masgn(asgn) => {
            write_code(&asgn.lhs, writer, &child_context)?;
            writer.write(b" = ")?;
            write_code(&asgn.rhs, writer, &child_context)?;
        }
        Node::MatchAlt(match_alt) => {
            write_code(&match_alt.lhs, writer, &child_context)?;
            writer.write(b" | ")?;
            write_code(&match_alt.rhs, writer, &child_context)?;
        }
        Node::MatchAs(match_as) => {
            write_code(&match_as.value, writer, &child_context)?;
            writer.write(b" => ")?;
            write_code(&match_as.as_, writer, &child_context)?;
        }
        Node::MatchCurrentLine(match_current_line) => {
            write_code(&match_current_line.re, writer, &child_context)?;
        }
        Node::MatchPattern(pat) => {
            write_code(&pat.value, writer, &child_context)?;
            writer.write(b" => ")?;
            write_code(&pat.pattern, writer, &child_context)?;
        }
        Node::MatchPatternP(pat) => {
            write_code(&pat.value, writer, &child_context)?;
            writer.write(b" in ")?;
            write_code(&pat.pattern, writer, &child_context)?;
        }
        Node::MatchRest(match_rest) => {
            writer.write(b"*")?;
            if let Some(name) = &match_rest.name {
                write_code(&name, writer, &child_context)?;
            }
        }
        Node::MatchVar(match_var) => {
            writer.write(match_var.name.as_bytes())?;
        }
        Node::MatchWithLvasgn(asgn) => {
            write_code(&asgn.re, writer, &child_context)?;
            writer.write(b" =~ ")?;
            write_code(&asgn.value, writer, &child_context)?;
        }
        Node::Mlhs(mlhs) => {
            if mlhs.begin_l.is_some() {
                writer.write(b"(")?;
            }
            write_code_with_separator(&mlhs.items, writer, &child_context, b", ")?;
            if mlhs.end_l.is_some() {
                writer.write(b")")?;
            }
        }
        Node::Module(module) => {
            writer.write(b"module ")?;
            write_code(&module.name, writer, &child_context)?;
            write_body_with_end!(module, writer, &child_context);
        }
        Node::Next(control) => {
            write_block_control_operator!(
                control,
                writer,
                b"next",
                b"next ",
                b"next(",
                child_context
            )
        }
        Node::Nil(_) => {
            writer.write(b"nil")?;
        }
        Node::NthRef(nthref) => {
            writer.write(b"$")?;
            writer.write(nthref.name.as_bytes())?;
        }
        Node::Numblock(block) => {
            write_code(&block.call, writer, &child_context)?;
            writer.write(b" { ")?;
            writer.write_fmt(format_args!("_{}", block.numargs))?;
            writer.write(b" }")?;
        }
        Node::OpAsgn(asgn) => {
            write_code(&asgn.recv, writer, &child_context)?;
            writer.write_fmt(format_args!(" {}", asgn.operator))?;
            writer.write(b"= ")?;
            write_code(&asgn.value, writer, &child_context)?;
        }
        Node::Optarg(arg) => {
            writer.write(arg.name.as_bytes())?;
            writer.write(b" = ")?;
            write_code(&arg.default, writer, &child_context)?;
        }
        Node::Or(or) => {
            write_code(&or.lhs, writer, &child_context)?;
            writer.write(b" || ")?;
            write_code(&or.rhs, writer, &child_context)?;
        }
        Node::OrAsgn(asgn) => {
            write_code(&asgn.recv, writer, &child_context)?;
            writer.write(b" ||= ")?;
            write_code(&asgn.value, writer, &child_context)?;
        }
        Node::Pair(pair) => {
            if pair.operator_l.size() >= 2 {
                write_code(&pair.key, writer, &child_context)?;
                writer.write(b" => ")?;
            } else {
                write_code(&pair.key, writer, &context.make_child("pair_key"))?;
                writer.write(b": ")?;
            }
            write_code(&pair.value, writer, &child_context)?;
        }
        Node::Pin(pin) => {
            writer.write(b"^")?;
            write_code(&pin.var, writer, &child_context)?;
        }
        Node::Postexe(exe) => {
            write_exe!(exe, writer, child_context, b"END { ");
        }
        Node::Preexe(exe) => {
            write_exe!(exe, writer, child_context, b"BEGIN { ");
        }
        Node::Procarg0(arg) => {
            if arg.begin_l.is_some() {
                writer.write(b"(")?;
            }
            write_code_with_separator(&arg.args, writer, &child_context, b", ")?;
            if arg.end_l.is_some() {
                writer.write(b")")?;
            }
        }
        Node::Rational(value) => {
            writer.write(value.value.as_bytes())?;
        }
        Node::Redo(_) => {
            writer.write(b"redo")?;
        }
        Node::RegOpt(reg_opt) => {
            if let Some(options) = &reg_opt.options {
                writer.write(options.as_bytes())?;
            }
        }
        Node::Regexp(expr) => {
            if expr.begin_l.size() == 1 {
                writer.write(b"/")?;
                write_parts(writer, &expr.parts, &child_context, "/", "\\/")?;
                writer.write(b"/")?;
            } else {
                writer.write(b"%r{")?;
                write_parts(writer, &expr.parts, &child_context, "}", "\\}")?;
                writer.write(b"}")?;
            }
            if let Some(opt) = &expr.options {
                write_code(opt, writer, &child_context)?;
            }
        }
        Node::Rescue(rescue) => {
            if let Some(body) = &rescue.body {
                write_body!(body, writer, child_context);
            }
            for body in rescue.rescue_bodies.iter() {
                write_indent(writer, context.indent - 1)?;
                write_code(body, writer, &child_context.outdent())?;
            }
            if let Some(else_body) = &rescue.else_ {
                write_indent(writer, context.indent - 1)?;
                writer.write(b"else\n")?;
                write_body!(else_body, writer, child_context);
            }
        }
        Node::RescueBody(rescue) => {
            if let Some(exc_list) = &rescue.exc_list {
                writer.write(b"rescue ")?;
                write_code(&exc_list, writer, &child_context)?;
            } else {
                writer.write(b"rescue")?;
            }
            if let Some(exc_var) = &rescue.exc_var {
                writer.write(b" => ")?;
                write_code(&exc_var, writer, &child_context)?;
            }
            writer.write(b"\n")?;
            if let Some(body) = &rescue.body {
                write_body!(body, writer, child_context.indent());
            }
        }
        Node::Restarg(arg) => {
            writer.write(b"*")?;
            if let Some(name) = &arg.name {
                writer.write(name.as_bytes())?;
            }
        }
        Node::Retry(_) => {
            writer.write(b"retry")?;
        }
        Node::Return(return_) => {
            if return_.args.len() > 0 {
                writer.write(b"return ")?;
                write_code_with_separator(&return_.args, writer, &child_context, b", ")?;
            } else {
                writer.write(b"return")?;
            }
        }
        Node::SClass(sclass) => {
            writer.write(b"class << ")?;
            write_code(&sclass.expr, writer, &child_context)?;
            write_body_with_end!(sclass, writer, &child_context);
        }
        Node::Self_(_) => {
            writer.write(b"self")?;
        }
        Node::Send(send) => {
            if send.method_name.eq("-@") {
                writer.write(b"-")?;
                if let Some(recv) = &send.recv {
                    write_code(&recv, writer, &child_context)?;
                }
            } else {
                if let Some(recv) = &send.recv {
                    write_code(&recv, writer, &child_context)?;
                    if send.dot_l.is_some() {
                        writer.write(b".")?;
                    } else {
                        writer.write(b" ")?;
                    }
                }
                if send.operator_l.is_some() {
                    writer.write(send.method_name[0..(send.method_name.len() - 1)].as_bytes())?;
                    writer.write(b" = ")?;
                    if send.args.len() > 0 {
                        write_code_with_separator(&send.args, writer, &child_context, b", ")?;
                    }
                } else {
                    writer.write(send.method_name.as_bytes())?;
                    match send.args.len() {
                        0 => {}
                        1 => {
                            if send.dot_l.is_none() {
                                writer.write(b" ")?;
                                write_code(&send.args[0], writer, &child_context)?;
                            } else {
                                writer.write(b"(")?;
                                write_code(&send.args[0], writer, &child_context)?;
                                writer.write(b")")?;
                            }
                        }
                        _ => {
                            writer.write(b"(")?;
                            write_code_with_separator(&send.args, writer, &child_context, b", ")?;
                            writer.write(b")")?;
                        }
                    }
                }
            }
        }
        Node::Shadowarg(shadow) => {
            writer.write(b";")?;
            writer.write(shadow.name.as_bytes())?;
        }
        Node::Splat(splat) => {
            writer.write(b"*")?;
            if let Some(value) = &splat.value {
                write_code(&value, writer, &child_context)?;
            }
        }
        Node::Str(str) => {
            writer.write(b"\'")?;
            write_string_with_escape(writer, &str.value, "'", "\'")?;
            writer.write(b"\'")?;
        }
        Node::Super(super_) => {
            if super_.args.len() > 0 {
                writer.write(b"super(")?;
                write_code_with_separator(&super_.args, writer, &child_context, b", ")?;
                writer.write(b")")?;
            } else {
                writer.write(b"super()")?;
            }
        }
        Node::Sym(sym) => {
            if context.parent_node_type == "pair_key" {
                writer.write(sym.name.as_raw())?;
            } else {
                if sym.begin_l.is_some() {
                    writer.write(b":")?;
                }
                writer.write(sym.name.as_raw())?;
                if sym.end_l.is_some() {
                    writer.write(b":")?;
                }
            }
        }
        Node::True(_) => {
            writer.write(b"true")?;
        }
        Node::Undef(undef) => {
            writer.write(b"undef ")?;
            write_code_with_separator(&undef.names, writer, &child_context, b", ")?;
        }
        Node::UnlessGuard(guard) => {
            writer.write(b"unless ")?;
            write_code(&guard.cond, writer, &child_context)?;
        }
        Node::Until(until) => {
            write_until_while!(until, writer, child_context, b"until ");
        }
        Node::UntilPost(until_post) => {
            write_code(&until_post.body, writer, &child_context)?;
            writer.write(b" until ")?;
            write_code(&until_post.cond, writer, &child_context)?;
        }
        Node::When(when) => {
            writer.write(b"when ")?;
            write_code_with_separator(&when.patterns, writer, &child_context, b", ")?;
            if let Some(body) = &when.body {
                writer.write(b"\n")?;
                write_body!(body, writer, child_context.indent());
            }
        }
        Node::While(while_) => {
            write_until_while!(while_, writer, child_context, b"while ");
        }
        Node::WhilePost(while_post) => {
            write_code(&while_post.body, writer, &child_context)?;
            writer.write(b" while ")?;
            write_code(&while_post.cond, writer, &child_context)?;
        }
        Node::XHeredoc(doc) => {
            writer.write(b"`")?;
            write_parts(writer, &doc.parts, &child_context, "`", "\\`")?;
            writer.write(b"`")?;
        }
        Node::Xstr(str) => {
            if str.begin_l.size() == 1 {
                writer.write(b"`")?;
                write_parts(writer, &str.parts, &child_context, "`", "\\`")?;
                writer.write(b"`")?;
            } else {
                writer.write(b"%x{")?;
                write_parts(writer, &str.parts, &child_context, "}", "\\}")?;
                writer.write(b"}")?;
            }
        }
        Node::Yield(control) => {
            write_block_control_operator!(
                control,
                writer,
                b"yield",
                b"yield ",
                b"yield(",
                child_context
            )
        }
        Node::ZSuper(_) => {
            writer.write(b"super")?;
        }
    }

    return Ok(());
}

fn write_code_with_separator<W: Write>(
    nodes: &Vec<Node>,
    writer: &mut BufWriter<W>,
    context: &CodeWriterContext,
    separator: &[u8],
) -> Result<(), std::io::Error> {
    if nodes.len() == 0 {
        return Ok(());
    }
    let last_index = nodes.len() - 1;
    for (i, node) in nodes.iter().enumerate() {
        write_code(node, writer, context)?;
        if i != last_index {
            writer.write(separator)?;
        }
    }

    return Ok(());
}

fn write_code_without_separator<W: Write>(
    nodes: &Vec<Node>,
    writer: &mut BufWriter<W>,
    context: &CodeWriterContext,
) -> Result<(), std::io::Error> {
    for node in nodes.iter() {
        write_indent(writer, context.indent)?;
        write_code(node, writer, context)?;
        writer.write(b"\n")?;
    }
    return Ok(());
}

fn write_indent<W: Write>(writer: &mut BufWriter<W>, indent: u32) -> Result<(), std::io::Error> {
    if indent > 0 {
        writer.write(b" ".repeat((2 * indent).try_into().unwrap())[..].as_ref())?;
    }
    return Ok(());
}

fn is_node_begin_block(node: &Node) -> bool {
    if let Node::Begin(begin) = node {
        return begin.begin_l.is_none();
    }
    return false;
}

fn write_string_with_escape<W: Write>(
    writer: &mut BufWriter<W>,
    string: &Bytes,
    escape: &str,
    escape_to: &str,
) -> Result<(), std::io::Error> {
    writer.write(
        string
            .to_string_lossy()
            .replace(escape, escape_to)
            .replace("#", "\\#")
            .as_bytes(),
    )?;
    return Ok(());
}

fn write_parts<W: Write>(
    writer: &mut BufWriter<W>,
    parts: &Vec<Node>,
    context: &CodeWriterContext,
    part_escape: &str,
    part_escape_to: &str,
) -> Result<(), std::io::Error> {
    for part in parts.iter() {
        match part {
            Node::Str(node) => {
                write_string_with_escape(writer, &node.value, part_escape, part_escape_to)?;
            }
            Node::Begin(_) => {
                writer.write(b"#{")?;
                write_code(part, writer, &context.make_child("part"))?;
                writer.write(b"}")?;
            }
            Node::Ivar(node) => {
                writer.write(b"#")?;
                writer.write(node.name.as_bytes())?;
            }
            Node::Gvar(node) => {
                writer.write(b"#")?;
                writer.write(node.name.as_bytes())?;
            }
            Node::Cvar(node) => {
                writer.write(b"#")?;
                writer.write(node.name.as_bytes())?;
            }
            _ => {
                writer.write_fmt(format_args!("<Unhandled node {}>", part.str_type()))?;
            }
        }
    }
    return Ok(());
}
