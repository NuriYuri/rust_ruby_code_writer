use std::io::{BufWriter, Write};

use lib_ruby_parser::Node;

use crate::{
    write_array, write_assign, write_block_control_operator, write_body, write_body_with_end,
    write_def_name_arg_and_body, write_exe, write_range, write_until_while,
};

pub fn write_code<W: Write>(
    node: &Node,
    writer: &mut BufWriter<W>,
    indent: u32,
) -> Result<(), std::io::Error> {
    match node {
        Node::Alias(alias) => {
            writer.write(b"alias ")?;
            write_code(&alias.to, writer, 0)?;
            writer.write(b" ")?;
            write_code(&alias.from, writer, 0)?;
        }
        Node::And(and) => {
            write_code(&and.lhs, writer, 0)?;
            writer.write(b" && ")?;
            write_code(&and.rhs, writer, 0)?;
        }
        Node::AndAsgn(asgn) => {
            write_code(&asgn.recv, writer, 0)?;
            writer.write(b" &&= ")?;
            write_code(&asgn.value, writer, indent)?;
        }
        Node::Arg(arg) => {
            writer.write(arg.name.as_bytes())?;
        }
        Node::Args(args) => {
            write_code_with_separator(&args.args, writer, b", ")?;
        }
        Node::Array(arr) => {
            write_array!(arr, writer, b"[", b"]");
        }
        Node::ArrayPattern(arr) => {
            write_array!(arr, writer, b"[", b"]");
        }
        Node::ArrayPatternWithTail(arr) => {
            write_array!(arr, writer, b"[", b",]");
        }
        Node::BackRef(back_ref) => {
            writer.write(back_ref.name.as_bytes())?;
        }
        Node::Begin(begin) => {
            if begin.begin_l.is_some() {
                write_indent(writer, indent)?;
                writer.write(b"(")?;
                write_code_with_separator(&begin.statements, writer, b"; ")?;
                writer.write(b")")?;
            } else {
                write_code_without_separator(&begin.statements, writer, indent)?;
            }
        }
        Node::Block(block) => {
            write_code(&block.call, writer, 0)?;
            let do_block = block.begin_l.size() == 2;
            match &block.args {
                Some(args) => {
                    writer.write(if do_block { b" do |" } else { b" { |" })?;
                    write_code(&args, writer, 0)?;
                    writer.write(if do_block { b"|\n" } else { b"| " })?;
                }
                None => {
                    writer.write(if do_block { b" do\n" } else { b" {" })?;
                }
            }
            if let Some(body) = &block.body {
                if do_block {
                    write_body!(body, writer, indent + 1);
                } else {
                    write_code(&body, writer, indent + 1)?;
                }
            }
            if do_block {
                write_indent(writer, indent)?;
                writer.write(b"end")?;
            } else {
                writer.write(b" }")?;
            }
        }
        Node::BlockPass(pass) => {
            writer.write(b"&")?;
            if let Some(value) = pass.value.as_ref() {
                write_code(value, writer, 0)?;
            }
        }
        Node::Blockarg(arg) => {
            writer.write(b"&")?;
            if let Some(name) = arg.name.as_ref() {
                writer.write(name.as_bytes())?;
            }
        }
        Node::Break(control) => {
            write_block_control_operator!(control, writer, b"break", b"break ", b"break(")
        }
        Node::CSend(send) => {
            write_code(&send.recv, writer, 0)?;
            writer.write(b"&.")?;
            match send.operator_l {
                Some(_) => {
                    writer.write(send.method_name[0..(send.method_name.len() - 1)].as_bytes())?;
                    writer.write(b" = ")?;
                    write_code_with_separator(&send.args, writer, b", ")?;
                }
                None => {
                    writer.write(send.method_name.as_bytes())?;
                    if send.begin_l.is_some() {
                        writer.write(b"(")?;
                    }
                    write_code_with_separator(&send.args, writer, b", ")?;
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
                    write_code(&expr, writer, 0)?;
                    writer.write(b"\n")?;
                }
                None => {
                    writer.write(b"case\n")?;
                }
            }
            for body in case.when_bodies.iter() {
                write_indent(writer, indent)?;
                write_code(body, writer, indent)?;
            }
            match &case.else_body {
                Some(else_body) => {
                    write_indent(writer, indent)?;
                    writer.write(b"else\n")?;
                    write_body!(else_body, writer, indent + 1);
                }
                None => {}
            }
            write_indent(writer, indent)?;
            writer.write(b"end")?;
        }
        Node::CaseMatch(case) => {
            writer.write(b"case ")?;
            write_code(&case.expr, writer, 0)?;
            writer.write(b"\n")?;
            write_code_without_separator(&case.in_bodies, writer, indent + 1)?;
            match &case.else_body {
                Some(else_body) => {
                    writer.write(b"else\n")?;
                    write_code(&else_body, writer, indent + 1)?;
                    writer.write(b"\n")?;
                }
                None => {}
            }
            write_indent(writer, indent)?;
            writer.write(b"end")?;
        }
        Node::Casgn(asgn) => {
            if let Some(scope) = &asgn.scope {
                write_code(&scope, writer, 0)?;
            }
            write_assign!(asgn, writer, indent);
        }
        Node::Cbase(_) => {
            writer.write(b"::")?;
        }
        Node::Class(class) => {
            writer.write(b"class ")?;
            write_code(&class.name, writer, 0)?;
            if let Some(super_class) = &class.superclass {
                writer.write(b" < ")?;
                write_code(&super_class, writer, 0)?;
            }
            write_body_with_end!(class, writer, indent);
        }
        Node::Complex(complex) => {
            writer.write(complex.value.as_bytes())?;
        }
        Node::Const(constant) => {
            if let Some(scope) = &constant.scope {
                write_code(&scope, writer, 0)?;
            }
            if constant.double_colon_l.is_some() {
                writer.write(b"::")?;
            }
            writer.write(constant.name.as_bytes())?;
        }
        Node::ConstPattern(constant) => {
            write_code(&constant.const_, writer, 0)?;
            writer.write(b"(")?;
            write_code(&constant.pattern, writer, 0)?;
            writer.write(b")")?;
        }
        Node::Cvar(var) => {
            writer.write(var.name.as_bytes())?;
        }
        Node::Cvasgn(asgn) => {
            write_assign!(asgn, writer, indent);
        }
        Node::Def(def) => {
            writer.write(b"def ")?;
            write_def_name_arg_and_body!(def, writer, indent);
        }
        Node::Defined(defined) => {
            writer.write(b"defined?")?;
            if defined.begin_l.is_some() {
                writer.write(b"(")?;
            }
            write_code(&defined.value, writer, 0)?;
            if defined.end_l.is_some() {
                writer.write(b")")?;
            }
        }
        Node::Defs(def) => {
            writer.write(b"def ")?;
            write_code(&def.definee, writer, 0)?;
            writer.write(b".")?;
            write_def_name_arg_and_body!(def, writer, indent);
        }
        Node::Dstr(_) => {
            writer.write(b"\"unsupported\"")?;
            todo!()
        }
        Node::Dsym(_) => {
            writer.write(b":\"unsupported\"")?;
            todo!()
        }
        Node::EFlipFlop(flip_flop) => {
            write_range!(flip_flop, writer, b"...");
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
                        write_code(&body, writer, indent)?;
                    }
                    _ => {
                        write_body!(body, writer, indent);
                    }
                }
            }
            write_indent(writer, indent - 1)?; // <= Ensure is part of the body of something else
            writer.write(b"ensure\n")?;
            if let Some(ensure) = &ensure.ensure {
                write_body!(ensure, writer, indent);
            }
        }
        Node::Erange(range) => {
            write_range!(range, writer, b"...");
        }
        Node::False(_) => {
            writer.write(b"false")?;
        }
        Node::File(_) => {
            writer.write(b"__FILE__")?;
        }
        Node::FindPattern(pat) => {
            write_array!(pat, writer, b"[", b"]");
        }
        Node::Float(float) => {
            writer.write(float.value.as_bytes())?;
        }
        Node::For(for_kw) => {
            writer.write(b"for ")?;
            write_code(&for_kw.iterator, writer, 0)?;
            writer.write(b" in ")?;
            write_code(&for_kw.iteratee, writer, 0)?;
            write_body_with_end!(for_kw, writer, indent);
        }
        Node::ForwardArg(_) | Node::ForwardedArgs(_) => {
            writer.write(b"...")?;
        }
        Node::Gvar(var) => {
            writer.write(var.name.as_bytes())?;
        }
        Node::Gvasgn(asgn) => {
            write_assign!(asgn, writer, indent);
        }
        Node::Hash(hash) => {
            write_array!(hash, writer, b"{", b"}", pairs);
        }
        Node::HashPattern(pat) => {
            write_array!(pat, writer, b"{", b"}");
        }
        Node::Heredoc(_) => {
            writer.write(b"<<-HERE\nNot Supported\nHERE")?;
            todo!()
        }
        Node::IFlipFlop(flip_flop) => {
            write_range!(flip_flop, writer, b"..");
        }
        Node::If(if_kw) => {
            if let Some(if_true) = &if_kw.if_true {
                writer.write(b"if ")?;
                write_code(&if_kw.cond, writer, 0)?;
                writer.write(b"\n")?;
                write_body!(if_true, writer, indent + 1);
                if let Some(if_false) = &if_kw.if_false {
                    write_indent(writer, indent)?;
                    writer.write(b"else\n")?;
                    write_body!(if_false, writer, indent + 1);
                }
            } else {
                if let Some(if_false) = &if_kw.if_false {
                    writer.write(b"unless ")?;
                    write_code(&if_kw.cond, writer, 0)?;
                    writer.write(b"\n")?;
                    write_body!(if_false, writer, indent + 1);
                }
            }
            write_indent(writer, indent)?;
            writer.write(b"end")?;
        }
        Node::IfGuard(guard) => {
            writer.write(b" if ")?;
            write_code(&guard.cond, writer, 0)?;
        }
        Node::IfMod(guard) => {
            if let Some(code) = &guard.if_true {
                write_code(&code, writer, indent)?;
                writer.write(b" if ")?;
            }
            if let Some(code) = &guard.if_false {
                write_code(&code, writer, indent)?;
                writer.write(b" unless ")?;
            }
            write_code(&guard.cond, writer, 0)?;
        }
        Node::IfTernary(ternary) => {
            write_code(&ternary.cond, writer, 0)?;
            writer.write(b" ? ")?;
            write_code(&ternary.if_true, writer, 0)?;
            writer.write(b" : ")?;
            write_code(&ternary.if_false, writer, 0)?;
        }
        Node::InPattern(pat) => {
            writer.write(b"in ")?;
            write_code(&pat.pattern, writer, 0)?;
            if let Some(guard) = &pat.guard {
                write_code(&guard, writer, 0)?;
            }
            if let Some(body) = &pat.body {
                writer.write(b" then\n")?;
                write_body!(body, writer, indent + 1);
            }
        }
        Node::Index(index) => {
            write_code(&index.recv, writer, 0)?;
            writer.write(b"[")?;
            write_code_with_separator(&index.indexes, writer, b", ")?;
            writer.write(b"]")?;
        }
        Node::IndexAsgn(asgn) => {
            write_code(&asgn.recv, writer, indent)?;
            writer.write(b"[")?;
            write_code_with_separator(&asgn.indexes, writer, b", ")?;
            writer.write(b"]")?;
            if let Some(value) = &asgn.value {
                writer.write(b" = ")?;
                write_code(&value, writer, indent)?;
            }
        }
        Node::Int(value) => {
            writer.write(value.value.as_bytes())?;
        }
        Node::Irange(range) => {
            write_range!(range, writer, b"..");
        }
        Node::Ivar(var) => {
            writer.write(var.name.as_bytes())?;
        }
        Node::Ivasgn(asgn) => {
            write_assign!(asgn, writer, indent);
        }
        Node::KwBegin(kw_begin) => {
            if kw_begin.begin_l.is_some() {
                writer.write(b"begin\n")?;
            }
            //write_code_without_separator(kw_begin.statements, writer, indent + 1)?;
            for body in kw_begin.statements.iter() {
                match body {
                    Node::Ensure(_) | Node::Rescue(_) => {
                        write_code(body, writer, indent + 1)?;
                    }
                    _ => {
                        write_body!(body, writer, indent + 1);
                    }
                }
            }
            if kw_begin.end_l.is_some() {
                write_indent(writer, indent)?;
                writer.write(b"end")?;
            }
        }
        Node::Kwarg(arg) => {
            writer.write(arg.name.as_bytes())?;
            writer.write(b":")?;
        }
        Node::Kwargs(args) => {
            write_code_with_separator(&args.pairs, writer, b", ")?;
        }
        Node::Kwnilarg(_) | Node::MatchNilPattern(_) => {
            writer.write(b"**nil")?;
        }
        Node::Kwoptarg(arg) => {
            writer.write(arg.name.as_bytes())?;
            writer.write(b": ")?;
            write_code(&arg.default, writer, 0)?;
        }
        Node::Kwrestarg(arg) => {
            writer.write(b"**")?;
            if let Some(name) = &arg.name {
                writer.write(name.as_bytes())?;
            }
        }
        Node::Kwsplat(splat) => {
            writer.write(b"**")?;
            write_code(&splat.value, writer, 0)?;
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
            write_assign!(asgn, writer, indent);
        }
        Node::Masgn(asgn) => {
            write_code(&asgn.lhs, writer, 0)?;
            writer.write(b" = ")?;
            write_code(&asgn.rhs, writer, indent)?;
        }
        Node::MatchAlt(match_alt) => {
            write_code(&match_alt.lhs, writer, 0)?;
            writer.write(b" | ")?;
            write_code(&match_alt.rhs, writer, 0)?;
        }
        Node::MatchAs(match_as) => {
            write_code(&match_as.value, writer, 0)?;
            writer.write(b" => ")?;
            write_code(&match_as.as_, writer, 0)?;
        }
        Node::MatchCurrentLine(match_current_line) => {
            write_code(&match_current_line.re, writer, 0)?;
        }
        Node::MatchPattern(pat) => {
            write_code(&pat.value, writer, 0)?;
            writer.write(b" => ")?;
            write_code(&pat.pattern, writer, 0)?;
        }
        Node::MatchPatternP(pat) => {
            write_code(&pat.value, writer, 0)?;
            writer.write(b" in ")?;
            write_code(&pat.pattern, writer, 0)?;
        }
        Node::MatchRest(match_rest) => {
            writer.write(b"*")?;
            if let Some(name) = &match_rest.name {
                write_code(&name, writer, 0)?;
            }
        }
        Node::MatchVar(match_var) => {
            writer.write(match_var.name.as_bytes())?;
        }
        Node::MatchWithLvasgn(asgn) => {
            write_code(&asgn.re, writer, 0)?;
            writer.write(b" =~ ")?;
            write_code(&asgn.value, writer, 0)?;
        }
        Node::Mlhs(mlhs) => {
            if mlhs.begin_l.is_some() {
                writer.write(b"(")?;
            }
            write_code_with_separator(&mlhs.items, writer, b", ")?;
            if mlhs.end_l.is_some() {
                writer.write(b")")?;
            }
        }
        Node::Module(module) => {
            writer.write(b"module ")?;
            write_code(&module.name, writer, 0)?;
            write_body_with_end!(module, writer, indent);
        }
        Node::Next(control) => {
            write_block_control_operator!(control, writer, b"next", b"next ", b"next(")
        }
        Node::Nil(_) => {
            writer.write(b"nil")?;
        }
        Node::NthRef(nthref) => {
            writer.write(b"$")?;
            writer.write(nthref.name.as_bytes())?;
        }
        Node::Numblock(block) => {
            write_code(&block.call, writer, 0)?;
            writer.write(b" { ")?;
            writer.write_fmt(format_args!("_{}", block.numargs))?;
            writer.write(b" }")?;
        }
        Node::OpAsgn(asgn) => {
            write_code(&asgn.recv, writer, 0)?;
            writer.write_fmt(format_args!(" {}", asgn.operator))?;
            writer.write(b"= ")?;
            write_code(&asgn.value, writer, indent)?;
        }
        Node::Optarg(arg) => {
            writer.write(arg.name.as_bytes())?;
            writer.write(b" = ")?;
            write_code(&arg.default, writer, 0)?;
        }
        Node::Or(or) => {
            write_code(&or.lhs, writer, 0)?;
            writer.write(b" || ")?;
            write_code(&or.rhs, writer, 0)?;
        }
        Node::OrAsgn(asgn) => {
            write_code(&asgn.recv, writer, 0)?;
            writer.write(b" ||= ")?;
            write_code(&asgn.value, writer, indent)?;
        }
        Node::Pair(pair) => {
            write_code(&pair.key, writer, 0)?;
            if pair.operator_l.size() >= 2 {
                writer.write(b" => ")?;
            } else {
                writer.write(b": ")?;
            }
            write_code(&pair.value, writer, 0)?;
        }
        Node::Pin(pin) => {
            writer.write(b"^")?;
            write_code(&pin.var, writer, 0)?;
        }
        Node::Postexe(exe) => {
            write_exe!(exe, writer, indent, b"END { ");
        }
        Node::Preexe(exe) => {
            write_exe!(exe, writer, indent, b"BEGIN { ");
        }
        Node::Procarg0(arg) => {
            if arg.begin_l.is_some() {
                writer.write(b"(")?;
            }
            write_code_with_separator(&arg.args, writer, b", ")?;
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
        Node::Regexp(_) => {
            writer.write(b"/unsupported/")?;
            todo!()
        }
        Node::Rescue(rescue) => {
            if let Some(body) = &rescue.body {
                write_body!(body, writer, indent);
            }
            for body in rescue.rescue_bodies.iter() {
                write_indent(writer, indent - 1)?;
                write_code(body, writer, indent - 1)?;
            }
            if let Some(else_body) = &rescue.else_ {
                write_indent(writer, indent - 1)?;
                writer.write(b"else\n")?;
                write_body!(else_body, writer, indent);
            }
        }
        Node::RescueBody(rescue) => {
            if let Some(exc_list) = &rescue.exc_list {
                writer.write(b"rescue ")?;
                write_code(&exc_list, writer, 0)?;
            } else {
                writer.write(b"rescue")?;
            }
            if let Some(exc_var) = &rescue.exc_var {
                writer.write(b" => ")?;
                write_code(&exc_var, writer, 0)?;
            }
            writer.write(b"\n")?;
            if let Some(body) = &rescue.body {
                write_body!(body, writer, indent + 1);
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
                write_code_with_separator(&return_.args, writer, b", ")?;
            } else {
                writer.write(b"return")?;
            }
        }
        Node::SClass(sclass) => {
            writer.write(b"class << ")?;
            write_code(&sclass.expr, writer, 0)?;
            write_body_with_end!(sclass, writer, indent);
        }
        Node::Self_(_) => {
            writer.write(b"self")?;
        }
        Node::Send(send) => {
            if send.method_name.eq("-@") {
                writer.write(b"-")?;
                if let Some(recv) = &send.recv {
                    write_code(&recv, writer, 0)?;
                }
            } else {
                if let Some(recv) = &send.recv {
                    write_code(&recv, writer, 0)?;
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
                        write_code_with_separator(&send.args, writer, b", ")?;
                    }
                } else {
                    writer.write(send.method_name.as_bytes())?;
                    match send.args.len() {
                        0 => {}
                        1 => {
                            if send.dot_l.is_none() {
                                writer.write(b" ")?;
                                write_code(&send.args[0], writer, 0)?;
                            } else {
                                writer.write(b"(")?;
                                write_code(&send.args[0], writer, 0)?;
                                writer.write(b")")?;
                            }
                        }
                        _ => {
                            writer.write(b"(")?;
                            write_code_with_separator(&send.args, writer, b", ")?;
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
                write_code(&value, writer, 0)?;
            }
        }
        Node::Str(str) => {
            if str.begin_l.is_some() {
                writer.write(b"\"")?;
            }
            writer.write(str.value.as_raw())?;
            if str.end_l.is_some() {
                writer.write(b"\"")?;
            }
        }
        Node::Super(super_) => {
            if super_.args.len() > 0 {
                writer.write(b"super(")?;
                write_code_with_separator(&super_.args, writer, b", ")?;
                writer.write(b")")?;
            } else {
                writer.write(b"super()")?;
            }
        }
        Node::Sym(sym) => {
            if sym.begin_l.is_some() {
                writer.write(b":")?;
            }
            writer.write(sym.name.as_raw())?;
            if sym.end_l.is_some() {
                writer.write(b":")?;
            }
        }
        Node::True(_) => {
            writer.write(b"true")?;
        }
        Node::Undef(undef) => {
            writer.write(b"undef ")?;
            write_code_with_separator(&undef.names, writer, b", ")?;
        }
        Node::UnlessGuard(guard) => {
            writer.write(b"unless ")?;
            write_code(&guard.cond, writer, 0)?;
        }
        Node::Until(until) => {
            write_until_while!(until, writer, indent, b"until ");
        }
        Node::UntilPost(until_post) => {
            write_code(&until_post.body, writer, indent)?;
            writer.write(b" until ")?;
            write_code(&until_post.cond, writer, 0)?;
        }
        Node::When(when) => {
            writer.write(b"when ")?;
            write_code_with_separator(&when.patterns, writer, b", ")?;
            if let Some(body) = &when.body {
                writer.write(b"\n")?;
                write_body!(body, writer, indent + 1);
            }
        }
        Node::While(while_) => {
            write_until_while!(while_, writer, indent, b"while ");
        }
        Node::WhilePost(while_post) => {
            write_code(&while_post.body, writer, indent)?;
            writer.write(b" while ")?;
            write_code(&while_post.cond, writer, 0)?;
        }
        Node::XHeredoc(_) => {
            writer.write(b"<<-`HERE`\nUNSUPPORTED\nHERE")?;
            todo!();
        }
        Node::Xstr(_) => {
            writer.write(b"`UNSUPPORTED`")?;
            todo!();
        }
        Node::Yield(control) => {
            write_block_control_operator!(control, writer, b"yield", b"yield ", b"yield(")
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
    separator: &[u8],
) -> Result<(), std::io::Error> {
    if nodes.len() == 0 {
        return Ok(());
    }
    let last_index = nodes.len() - 1;
    for (i, node) in nodes.iter().enumerate() {
        write_code(node, writer, 0)?;
        if i != last_index {
            writer.write(separator)?;
        }
    }

    return Ok(());
}

fn write_code_without_separator<W: Write>(
    nodes: &Vec<Node>,
    writer: &mut BufWriter<W>,
    indent: u32,
) -> Result<(), std::io::Error> {
    for node in nodes.iter() {
        write_indent(writer, indent)?;
        write_code(node, writer, indent)?;
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
