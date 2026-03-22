// SPDX-License-Identifier: PMPL-1.0-or-later
//! My Language Formatter
//!
//! Formats My Language source code according to style guidelines.

#![forbid(unsafe_code)]
use my_lang::{Program, TopLevel, FnDecl, Stmt, Expr, Type, Pattern, Block};
use std::fmt::Write;

/// Formatting configuration
#[derive(Debug, Clone)]
pub struct FormatConfig {
    /// Number of spaces per indentation level
    pub indent_size: usize,
    /// Maximum line length before wrapping
    pub max_line_length: usize,
    /// Use spaces (true) or tabs (false)
    pub use_spaces: bool,
}

impl Default for FormatConfig {
    fn default() -> Self {
        FormatConfig {
            indent_size: 4,
            max_line_length: 100,
            use_spaces: true,
        }
    }
}

/// Formatter for My Language source code
pub struct Formatter {
    config: FormatConfig,
    output: String,
    indent_level: usize,
}

impl Formatter {
    /// Create a new formatter with default configuration
    pub fn new() -> Self {
        Self::with_config(FormatConfig::default())
    }

    /// Create a new formatter with custom configuration
    pub fn with_config(config: FormatConfig) -> Self {
        Formatter {
            config,
            output: String::new(),
            indent_level: 0,
        }
    }

    /// Format a program
    pub fn format_program(&mut self, program: &Program) -> String {
        self.output.clear();
        self.indent_level = 0;

        for (i, item) in program.items.iter().enumerate() {
            if i > 0 {
                self.output.push('\n');
            }
            self.format_top_level(item);
        }

        self.output.clone()
    }

    fn format_top_level(&mut self, item: &TopLevel) {
        match item {
            TopLevel::Function(f) => self.format_function(f),
            TopLevel::Struct(s) => {
                self.write_indent();
                write!(self.output, "struct {} {{", s.name.name).unwrap();
                self.indent_level += 1;
                for field in &s.fields {
                    self.output.push('\n');
                    self.write_indent();
                    write!(self.output, "{}: ", field.name.name).unwrap();
                    self.format_type(&field.ty);
                    self.output.push(',');
                }
                self.indent_level -= 1;
                self.output.push('\n');
                self.write_indent();
                self.output.push_str("}\n");
            }
            TopLevel::Effect(e) => {
                self.write_indent();
                write!(self.output, "effect {} {{\n", e.name.name).unwrap();
                self.indent_level += 1;
                for op in &e.ops {
                    self.write_indent();
                    write!(self.output, "{}: ", op.name.name).unwrap();
                    self.format_type(&op.ty);
                    self.output.push_str(";\n");
                }
                self.indent_level -= 1;
                self.write_indent();
                self.output.push_str("}\n");
            }
            TopLevel::AiModel(m) => {
                self.write_indent();
                write!(self.output, "ai_model {} {{\n", m.name.name).unwrap();
                self.indent_level += 1;
                for attr in &m.attributes {
                    self.write_indent();
                    write!(self.output, "{:?}\n", attr).unwrap();
                }
                self.indent_level -= 1;
                self.write_indent();
                self.output.push_str("}\n");
            }
            _ => {}
        }
    }

    fn format_function(&mut self, func: &FnDecl) {
        self.write_indent();
        write!(self.output, "fn {}(", func.name.name).unwrap();

        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            write!(self.output, "{}: ", param.name.name).unwrap();
            self.format_type(&param.ty);
        }

        self.output.push(')');

        if let Some(ret_ty) = &func.return_type {
            self.output.push_str(" -> ");
            self.format_type(ret_ty);
        }

        self.output.push_str(" {\n");
        self.indent_level += 1;
        self.format_block(&func.body);
        self.indent_level -= 1;
        self.write_indent();
        self.output.push_str("}\n");
    }

    fn format_block(&mut self, block: &Block) {
        for stmt in &block.stmts {
            self.format_stmt(stmt);
        }
    }

    fn format_stmt(&mut self, stmt: &Stmt) {
        self.write_indent();

        match stmt {
            Stmt::Let { name, ty, value, .. } => {
                write!(self.output, "let {}", name.name).unwrap();
                if let Some(t) = ty {
                    self.output.push_str(": ");
                    self.format_type(t);
                }
                self.output.push_str(" = ");
                self.format_expr(value, false);
                self.output.push_str(";\n");
            }
            Stmt::Expr(expr) => {
                self.format_expr(expr, false);
                self.output.push_str(";\n");
            }
            Stmt::Return { value, .. } => {
                self.output.push_str("return");
                if let Some(v) = value {
                    self.output.push(' ');
                    self.format_expr(v, false);
                }
                self.output.push_str(";\n");
            }
            Stmt::If { condition, then_block, else_block, .. } => {
                self.output.push_str("if ");
                self.format_expr(condition, false);
                self.output.push_str(" {\n");
                self.indent_level += 1;
                self.format_block(then_block);
                self.indent_level -= 1;
                self.write_indent();
                self.output.push('}');

                if let Some(else_b) = else_block {
                    self.output.push_str(" else {\n");
                    self.indent_level += 1;
                    self.format_block(else_b);
                    self.indent_level -= 1;
                    self.write_indent();
                    self.output.push('}');
                }
                self.output.push('\n');
            }
            Stmt::Go { block, .. } => {
                self.output.push_str("go {\n");
                self.indent_level += 1;
                self.format_block(block);
                self.indent_level -= 1;
                self.write_indent();
                self.output.push_str("}\n");
            }
            Stmt::Await { value, .. } => {
                self.output.push_str("await ");
                self.format_expr(value, false);
                self.output.push_str(";\n");
            }
            Stmt::Try { value, .. } => {
                self.output.push_str("try ");
                self.format_expr(value, false);
                self.output.push_str(";\n");
            }
            Stmt::Comptime { block, .. } => {
                self.output.push_str("comptime {\n");
                self.indent_level += 1;
                self.format_block(block);
                self.indent_level -= 1;
                self.write_indent();
                self.output.push_str("}\n");
            }
            Stmt::Ai(ai_stmt) => {
                write!(self.output, "ai {:?} ", ai_stmt.keyword).unwrap();
                match &ai_stmt.body {
                    my_lang::AiStmtBody::Block(b) => {
                        self.output.push_str("{\n");
                        self.indent_level += 1;
                        self.format_block(b);
                        self.indent_level -= 1;
                        self.write_indent();
                        self.output.push_str("}\n");
                    }
                    my_lang::AiStmtBody::Expr(e) => {
                        self.format_expr(e, false);
                        self.output.push_str(";\n");
                    }
                }
            }
            Stmt::Belief { name, ty, confidence, .. } => {
                write!(self.output, "belief {}: ", name.name).unwrap();
                self.format_type(ty);
                write!(self.output, " where confidence({});\n", confidence).unwrap();
            }
        }
    }

    fn format_expr(&mut self, expr: &Expr, in_binary: bool) {
        match expr {
            Expr::Literal(lit) => {
                write!(self.output, "{:?}", lit).unwrap();
            }
            Expr::Ident(ident) => {
                write!(self.output, "{}", ident.name).unwrap();
            }
            Expr::Call { callee, args, .. } => {
                self.format_expr(callee, false);
                self.output.push('(');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.format_expr(arg, false);
                }
                self.output.push(')');
            }
            Expr::Field { object, field, .. } => {
                self.format_expr(object, false);
                write!(self.output, ".{}", field.name).unwrap();
            }
            Expr::Binary { left, op, right, .. } => {
                if in_binary {
                    self.output.push('(');
                }
                self.format_expr(left, true);
                write!(self.output, " {:?} ", op).unwrap();
                self.format_expr(right, true);
                if in_binary {
                    self.output.push(')');
                }
            }
            Expr::Unary { op, operand, .. } => {
                write!(self.output, "{:?}", op).unwrap();
                self.format_expr(operand, true);
            }
            Expr::Block(block) => {
                self.output.push_str("{\n");
                self.indent_level += 1;
                self.format_block(block);
                self.indent_level -= 1;
                self.write_indent();
                self.output.push('}');
            }
            Expr::Lambda { params, body, .. } => {
                self.output.push('|');
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    write!(self.output, "{}: ", param.name.name).unwrap();
                    self.format_type(&param.ty);
                }
                self.output.push_str("| ");
                match body {
                    my_lang::LambdaBody::Expr(e) => {
                        self.output.push_str("=> ");
                        self.format_expr(e, false);
                    }
                    my_lang::LambdaBody::Block(b) => {
                        self.output.push_str("{\n");
                        self.indent_level += 1;
                        self.format_block(b);
                        self.indent_level -= 1;
                        self.write_indent();
                        self.output.push('}');
                    }
                }
            }
            Expr::Match { scrutinee, arms, .. } => {
                self.output.push_str("match ");
                self.format_expr(scrutinee, false);
                self.output.push_str(" {\n");
                self.indent_level += 1;
                for arm in arms {
                    self.write_indent();
                    self.format_pattern(&arm.pattern);
                    self.output.push_str(" => ");
                    self.format_expr(&arm.body, false);
                    self.output.push_str(",\n");
                }
                self.indent_level -= 1;
                self.write_indent();
                self.output.push('}');
            }
            Expr::Array { elements, .. } => {
                self.output.push('[');
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.format_expr(elem, false);
                }
                self.output.push(']');
            }
            Expr::Try { operand, .. } => {
                self.output.push_str("try ");
                self.format_expr(operand, false);
            }
            Expr::Restrict { operand, .. } => {
                self.output.push_str("restrict ");
                self.format_expr(operand, false);
            }
            Expr::Ai(ai_expr) => {
                write!(self.output, "ai {:?}", ai_expr).unwrap();
            }
            Expr::Record { fields, .. } => {
                self.output.push_str("{ ");
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    write!(self.output, "{}: ", field.name.name).unwrap();
                    self.format_expr(&field.value, false);
                }
                self.output.push_str(" }");
            }
        }
    }

    fn format_type(&mut self, ty: &Type) {
        match ty {
            Type::Primitive(p) => write!(self.output, "{:?}", p).unwrap(),
            Type::Named(name) => write!(self.output, "{}", name.name).unwrap(),
            Type::Array { element, .. } => {
                self.output.push('[');
                self.format_type(element);
                self.output.push(']');
            }
            Type::Function { param, result, .. } => {
                self.output.push('(');
                self.format_type(param);
                self.output.push_str(") -> ");
                self.format_type(result);
            }
            Type::Ai { inner, .. } => {
                self.output.push_str("AI<");
                self.format_type(inner);
                self.output.push('>');
            }
            Type::Effect { inner, .. } => {
                self.format_type(inner);
                self.output.push_str("!");
            }
            _ => write!(self.output, "{:?}", ty).unwrap(),
        }
    }

    fn format_pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Literal(lit) => write!(self.output, "{:?}", lit).unwrap(),
            Pattern::Ident(ident) => write!(self.output, "{}", ident.name).unwrap(),
            Pattern::Wildcard(_) => self.output.push('_'),
            Pattern::Constructor { name, args, .. } => {
                write!(self.output, "{}(", name.name).unwrap();
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.format_pattern(arg);
                }
                self.output.push(')');
            }
        }
    }

    fn write_indent(&mut self) {
        let indent_str = if self.config.use_spaces {
            " ".repeat(self.config.indent_size * self.indent_level)
        } else {
            "\t".repeat(self.indent_level)
        };
        self.output.push_str(&indent_str);
    }
}

impl Default for Formatter {
    fn default() -> Self {
        Self::new()
    }
}

/// Format a My Language program with default configuration
pub fn format(program: &Program) -> String {
    let mut formatter = Formatter::new();
    formatter.format_program(program)
}

/// Format a My Language program with custom configuration
pub fn format_with_config(program: &Program, config: FormatConfig) -> String {
    let mut formatter = Formatter::with_config(config);
    formatter.format_program(program)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_simple_function() {
        let source = "fn add(x:Int,y:Int)->Int{return x+y;}";
        let program = my_lang::parse(source).unwrap();
        let formatted = format(&program);

        assert!(formatted.contains("fn add(x: Int, y: Int) -> Int"));
        assert!(formatted.contains("    return"));
    }
}
