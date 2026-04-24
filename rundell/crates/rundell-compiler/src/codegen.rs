//! Rundell → Rust code generator.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use rundell_parser::ast::*;
use rundell_parser::parse;

use crate::prelude::PRELUDE;

/// Rust keywords that must be escaped as raw identifiers or renamed.
const RUST_KEYWORDS: &[&str] = &[
    "as", "async", "await", "become", "box", "break", "const", "continue",
    "crate", "do", "dyn", "else", "enum", "extern", "false", "final", "fn",
    "for", "if", "impl", "in", "let", "loop", "macro", "match", "mod",
    "move", "mut", "override", "priv", "pub", "ref", "return", "self",
    "static", "struct", "super", "trait", "true", "try", "type", "typeof",
    "unsafe", "unsized", "use", "virtual", "where", "while", "yield",
];

fn safe_ident(name: &str) -> String {
    if RUST_KEYWORDS.contains(&name) {
        format!("_v_{name}")
    } else {
        name.replace('-', "_")
    }
}

/// Code generator — walks the Rundell AST and emits Rust source.
pub struct CodeGen {
    /// All global variable names (access via `ctx.name`).
    globals: HashSet<String>,
    /// All function definitions collected in a pre-pass.
    functions: HashMap<String, FunctionDefStmt>,
    /// Variables local to the currently-generating function.
    current_locals: HashSet<String>,
    /// Generated output buffer.
    output: String,
    /// Current indentation level (4 spaces each).
    indent: usize,
    /// Directory of the source file (for import resolution).
    source_dir: PathBuf,
}

impl CodeGen {
    pub fn new(source_dir: PathBuf) -> Self {
        CodeGen {
            globals: HashSet::new(),
            functions: HashMap::new(),
            current_locals: HashSet::new(),
            output: String::new(),
            indent: 0,
            source_dir,
        }
    }

    // ── Output helpers ────────────────────────────────────────

    fn emit(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn emit_line(&mut self, s: &str) {
        let pad = "    ".repeat(self.indent);
        self.output.push_str(&format!("{pad}{s}\n"));
    }

    fn emit_blank(&mut self) {
        self.output.push('\n');
    }

    // ── Variable resolution ───────────────────────────────────

    fn is_global(&self, name: &str) -> bool {
        self.globals.contains(name)
    }

    fn is_local(&self, name: &str) -> bool {
        self.current_locals.contains(name)
    }

    /// Generate a Rust expression that reads a variable (cloning it).
    fn var_get(&self, name: &str) -> String {
        let id = safe_ident(name);
        if self.is_local(name) || !self.is_global(name) {
            format!("{id}.clone()")
        } else {
            format!("ctx.{id}.clone()")
        }
    }

    /// Generate the lhs of an assignment to a variable.
    fn var_lhs(&self, name: &str) -> String {
        let id = safe_ident(name);
        if self.is_local(name) || !self.is_global(name) {
            id
        } else {
            format!("ctx.{id}")
        }
    }

    /// Generate a mutable borrow of a variable.
    fn var_mut_ref(&self, name: &str) -> String {
        let id = safe_ident(name);
        if self.is_local(name) || !self.is_global(name) {
            format!("&mut {id}")
        } else {
            format!("&mut ctx.{id}")
        }
    }

    // ── Pre-passes ────────────────────────────────────────────

    /// Collect global variable names and function definitions from top-level stmts.
    fn collect_globals_and_fns(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            match stmt {
                Stmt::Define(d) if d.global => {
                    self.globals.insert(d.name.clone());
                }
                Stmt::FunctionDef(fd) => {
                    self.functions.insert(fd.name.clone(), fd.clone());
                }
                _ => {}
            }
        }
    }

    /// Collect all variable names declared anywhere in a list of statements
    /// (hoisted to function scope for Rundell-compatible scoping).
    fn collect_local_defs(stmts: &[Stmt]) -> HashSet<String> {
        let mut names = HashSet::new();
        Self::collect_from_stmts(stmts, &mut names);
        names
    }

    fn collect_from_stmts(stmts: &[Stmt], names: &mut HashSet<String>) {
        for stmt in stmts {
            match stmt {
                Stmt::Define(d) if !d.global => { names.insert(d.name.clone()); }
                Stmt::ForLoop(fl) => {
                    names.insert(fl.var.clone());
                    Self::collect_from_stmts(&fl.body, names);
                }
                Stmt::ForEach(fe) => {
                    names.insert(fe.var.clone());
                    Self::collect_from_stmts(&fe.body, names);
                }
                Stmt::If(i) => {
                    Self::collect_from_stmts(&i.then_body, names);
                    for (_, body) in &i.else_ifs { Self::collect_from_stmts(body, names); }
                    if let Some(b) = &i.else_body { Self::collect_from_stmts(b, names); }
                }
                Stmt::WhileLoop(wl) => { Self::collect_from_stmts(&wl.body, names); }
                Stmt::Switch(sw) => {
                    for c in &sw.cases { Self::collect_from_stmts(&c.body, names); }
                }
                Stmt::TryCatch(tc) => {
                    Self::collect_from_stmts(&tc.try_body, names);
                    for c in &tc.catches { Self::collect_from_stmts(&c.body, names); }
                    if let Some(b) = &tc.finally_body { Self::collect_from_stmts(b, names); }
                }
                _ => {}
            }
        }
    }

    // ── Public entry point ────────────────────────────────────

    /// Generate a complete Rust source file for the given program.
    pub fn generate(&mut self, stmts: &[Stmt]) -> String {
        // Inline imports first.
        let stmts = self.inline_imports(stmts);

        // Pre-pass: collect globals and functions.
        self.collect_globals_and_fns(&stmts);

        // Emit prelude.
        self.emit(PRELUDE);
        self.emit_blank();

        // Emit Ctx struct.
        self.emit_ctx_struct();

        // Emit user functions.
        let fns: Vec<FunctionDefStmt> = self.functions.values().cloned().collect();
        for fd in fns {
            self.emit_function(&fd);
            self.emit_blank();
        }

        // Emit main().
        self.emit_main(&stmts);

        self.output.clone()
    }

    // ── Import inlining ───────────────────────────────────────

    fn inline_imports(&self, stmts: &[Stmt]) -> Vec<Stmt> {
        let mut result = Vec::new();
        for stmt in stmts {
            if let Stmt::Import(path) = stmt {
                let mut full = self.source_dir.clone();
                full.push(format!("{path}.run"));
                match std::fs::read_to_string(&full) {
                    Ok(src) => match parse(&src) {
                        Ok(imported) => {
                            for s in imported {
                                match &s {
                                    Stmt::Define(d) if d.global => result.push(s),
                                    Stmt::FunctionDef(_) => result.push(s),
                                    _ => {}
                                }
                            }
                        }
                        Err(_) => {
                            // Silently skip parse errors in imports
                        }
                    },
                    Err(_) => {
                        // Silently skip missing imports
                    }
                }
            } else {
                result.push(stmt.clone());
            }
        }
        result
    }

    // ── Ctx struct ────────────────────────────────────────────

    fn emit_ctx_struct(&mut self) {
        self.emit_line("// ── Global context ─────────────────────────────────");
        if self.globals.is_empty() {
            self.emit_line("struct Ctx {}");
        } else {
            self.emit_line("struct Ctx {");
            self.indent += 1;
            let mut sorted: Vec<String> = self.globals.iter().cloned().collect();
            sorted.sort();
            for name in &sorted {
                self.emit_line(&format!("{}: RVal,", safe_ident(name)));
            }
            self.indent -= 1;
            self.emit_line("}");
        }
        self.emit_blank();
    }

    // ── Function emission ─────────────────────────────────────

    fn emit_function(&mut self, fd: &FunctionDefStmt) {
        // Build parameter list.
        let params: String = fd.params.iter()
            .map(|p| format!("{}: RVal", safe_ident(&p.name)))
            .collect::<Vec<_>>()
            .join(", ");

        let sig = if params.is_empty() {
            format!("fn usr_{}(mut ctx: &mut Ctx) -> RVal", safe_ident(&fd.name))
        } else {
            format!("fn usr_{}(mut ctx: &mut Ctx, {params}) -> RVal", safe_ident(&fd.name))
        };

        self.emit_line(&format!("// fn {}()", fd.name));
        self.emit_line(&sig);
        self.emit_line("{");
        self.indent += 1;

        // Set up locals: all params are local, plus all declared vars in body.
        let mut locals: HashSet<String> = fd.params.iter().map(|p| p.name.clone()).collect();
        let body_locals = Self::collect_local_defs(&fd.body);
        locals.extend(body_locals.clone());
        let old_locals = std::mem::replace(&mut self.current_locals, locals);

        // Hoist body-declared variables (not params).
        for name in &body_locals {
            let id = safe_ident(name);
            self.emit_line(&format!("let mut {id} = RVal::Null;"));
        }
        if !body_locals.is_empty() { self.emit_blank(); }

        // Emit body.
        for stmt in &fd.body {
            self.emit_stmt(stmt);
        }

        // Default return.
        self.emit_line("RVal::Null");
        self.indent -= 1;
        self.emit_line("}");

        self.current_locals = old_locals;
    }

    // ── Main function ─────────────────────────────────────────

    fn emit_main(&mut self, stmts: &[Stmt]) {
        self.emit_line("// ── Entry point ────────────────────────────────────");
        self.emit_line("fn main() {");
        self.indent += 1;

        // Initialise Ctx.
        if self.globals.is_empty() {
            self.emit_line("let mut ctx = Ctx {};");
        } else {
            self.emit_line("let mut ctx = Ctx {");
            self.indent += 1;
            let mut sorted: Vec<String> = self.globals.iter().cloned().collect();
            sorted.sort();
            for name in &sorted {
                self.emit_line(&format!("{}: RVal::Null,", safe_ident(name)));
            }
            self.indent -= 1;
            self.emit_line("};");
        }
        self.emit_blank();

        // Hoist local vars from top-level.
        let top_locals: HashSet<String> = stmts.iter().filter_map(|s| {
            if let Stmt::Define(d) = s { if !d.global { return Some(d.name.clone()); } }
            None
        }).collect();
        let fe_for_locals = Self::collect_local_defs(stmts);
        let mut all_locals = top_locals;
        all_locals.extend(fe_for_locals);
        // Remove globals from locals
        all_locals.retain(|n| !self.globals.contains(n));

        let old_locals = std::mem::replace(&mut self.current_locals, all_locals.clone());
        for name in &all_locals {
            let id = safe_ident(name);
            self.emit_line(&format!("let mut {id} = RVal::Null;"));
        }
        if !all_locals.is_empty() { self.emit_blank(); }

        // Emit top-level statements, skipping function definitions (already emitted).
        for stmt in stmts {
            if matches!(stmt, Stmt::FunctionDef(_)) { continue; }
            self.emit_stmt(stmt);
        }

        self.indent -= 1;
        self.emit_line("}");
        self.current_locals = old_locals;
    }

    // ── Statement emission ────────────────────────────────────

    fn emit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Define(d)         => self.emit_define(d),
            Stmt::Set(s)            => self.emit_set(s),
            Stmt::Print(expr)       => self.emit_print(expr),
            Stmt::Debug(path, expr) => self.emit_debug(path, expr),
            Stmt::Receive(r)        => self.emit_receive(r),
            Stmt::If(i)             => self.emit_if(i),
            Stmt::Switch(sw)        => self.emit_switch(sw),
            Stmt::ForLoop(fl)       => self.emit_for(fl),
            Stmt::WhileLoop(wl)     => self.emit_while(wl),
            Stmt::ForEach(fe)       => self.emit_foreach(fe),
            Stmt::FunctionDef(_)    => { /* already emitted */ }
            Stmt::Return(expr)      => self.emit_return(expr),
            Stmt::TryCatch(tc)      => self.emit_try(tc),
            Stmt::Remove(expr)      => self.emit_remove(expr),
            Stmt::Append(col, val)  => self.emit_append_stmt(col, val),
            Stmt::ExprStmt(expr)    => {
                let e = self.gen_expr(expr);
                self.emit_line(&format!("let _ = {e};"));
            }
            Stmt::Import(_) => { /* already inlined */ }
            // GUI / query / event-timer statements are not supported in compiled mode.
            Stmt::FormDef(_) | Stmt::EventTimerDef(_) | Stmt::DefineControl(_, _) => {
                self.emit_line("panic!(\"GUI features are not supported in compiled mode\");");
            }
            Stmt::CredentialsDef(_) | Stmt::QueryDef(_) | Stmt::Attempt(_) => {
                self.emit_line("panic!(\"REST query features are not supported in compiled mode\");");
            }
        }
    }

    fn emit_define(&mut self, d: &DefineStmt) {
        let lhs = self.var_lhs(&d.name);
        if let Some(init) = &d.init {
            let e = self.gen_expr(init);
            self.emit_line(&format!("{lhs} = {e};"));
        }
        // If no initializer, variable was already hoisted to Null — nothing to emit.
    }

    fn emit_set(&mut self, s: &SetStmt) {
        match &s.target {
            SetTarget::Identifier(name) => {
                match &s.op {
                    SetOp::Assign(expr) => {
                        let lhs = self.var_lhs(name);
                        let rhs = self.gen_expr(expr);
                        self.emit_line(&format!("{lhs} = {rhs};"));
                    }
                    SetOp::Increment => {
                        let lhs = self.var_lhs(name);
                        let get = self.var_get(name);
                        self.emit_line(&format!("{lhs} = rval_add({get}, RVal::Int(1));"));
                    }
                    SetOp::Decrement => {
                        let lhs = self.var_lhs(name);
                        let get = self.var_get(name);
                        self.emit_line(&format!("{lhs} = rval_sub({get}, RVal::Int(1));"));
                    }
                }
            }
            SetTarget::Index(col_expr, key_expr) => {
                if let SetOp::Assign(val_expr) = &s.op {
                    // Find root identifier for mutation.
                    if let Some(root) = extract_root_ident(col_expr) {
                        let key = self.gen_expr(key_expr);
                        let val = self.gen_expr(val_expr);
                        let path = collect_path_after_root(col_expr);
                        if path.is_empty() {
                            let mr = self.var_mut_ref(&root);
                            self.emit_line(&format!("rval_set_index({mr}, {key}, {val});"));
                        } else {
                            // Nested: rebuild the root json, navigate, then set.
                            let get = self.var_get(&root);
                            let lhs = self.var_lhs(&root);
                            self.emit_line("{");
                            self.indent += 1;
                            self.emit_line(&format!("let mut _rnd_root = {get};"));
                            for seg in &path {
                                let seg_e = self.gen_expr(seg);
                                self.emit_line(&format!("let mut _rnd_root = rval_index(_rnd_root.clone(), {seg_e});"));
                            }
                            self.emit_line(&format!("rval_set_index(&mut _rnd_root, {key}, {val});"));
                            // This is a simplified approximation — for deeply nested sets
                            // we just emit a comment noting the limitation.
                            self.emit_line("// Note: deeply nested set may not persist to root in compiled mode");
                            self.emit_line(&format!("{lhs} = _rnd_root;"));
                            self.indent -= 1;
                            self.emit_line("}");
                        }
                    } else {
                        self.emit_line("panic!(\"compiled mode: complex index assignment not supported\");");
                    }
                }
            }
            SetTarget::ObjectPath(_) => {
                self.emit_line("panic!(\"GUI object-path assignment not supported in compiled mode\");");
            }
        }
    }

    fn emit_print(&mut self, expr: &Expr) {
        let e = self.gen_expr(expr);
        self.emit_line(&format!("print!(\"{{}}\", {e}.to_display());"));
        self.emit_line("let _ = std::io::stdout().flush();");
    }

    fn emit_debug(&mut self, path_expr: &Option<Expr>, msg_expr: &Expr) {
        let msg = self.gen_expr(msg_expr);
        match path_expr {
            None => {
                self.emit_line(&format!("rnd_debug_stdout(&{msg}.to_display());"));
            }
            Some(pe) => {
                let path = self.gen_expr(pe);
                self.emit_line(&format!("rnd_debug_file(&{path}.to_display(), &{msg}.to_display());"));
            }
        }
    }

    fn emit_receive(&mut self, r: &ReceiveStmt) {
        if let Some(prompt) = &r.prompt {
            let p = self.gen_expr(prompt);
            self.emit_line(&format!("print!(\"{{}}\", {p}.to_display());"));
            self.emit_line("let _ = std::io::stdout().flush();");
        }
        let lhs = self.var_lhs(&r.variable);
        self.emit_line("{ let mut _rnd_line = String::new();");
        self.indent += 1;
        self.emit_line("std::io::stdin().read_line(&mut _rnd_line).unwrap();");
        self.emit_line(&format!("{lhs} = RVal::Str(_rnd_line.trim_end_matches('\\n').trim_end_matches('\\r').to_string()); }}"));
        self.indent -= 1;
    }

    fn emit_if(&mut self, i: &IfStmt) {
        let cond = self.gen_expr(&i.condition);
        self.emit_line(&format!("if {cond}.is_truthy() {{"));
        self.indent += 1;
        for stmt in &i.then_body { self.emit_stmt(stmt); }
        self.indent -= 1;

        for (cond_expr, body) in &i.else_ifs {
            let c = self.gen_expr(cond_expr);
            self.emit_line(&format!("}} else if {c}.is_truthy() {{"));
            self.indent += 1;
            for stmt in body { self.emit_stmt(stmt); }
            self.indent -= 1;
        }

        if let Some(else_body) = &i.else_body {
            self.emit_line("} else {");
            self.indent += 1;
            for stmt in else_body { self.emit_stmt(stmt); }
            self.indent -= 1;
        }
        self.emit_line("}");
    }

    fn emit_switch(&mut self, sw: &SwitchStmt) {
        let subj = self.gen_expr(&sw.subject);
        self.emit_line(&format!("let _rnd_sw = {subj};"));
        let mut first = true;
        for case in &sw.cases {
            let kw = if first { "if" } else { "} else if" };
            first = false;
            let cond = self.gen_switch_pattern(&case.pattern);
            if cond == "true" {
                // Default case
                self.emit_line("} else {");
            } else {
                self.emit_line(&format!("{kw} {cond} {{"));
            }
            self.indent += 1;
            for stmt in &case.body { self.emit_stmt(stmt); }
            self.indent -= 1;
        }
        if !sw.cases.is_empty() {
            self.emit_line("}");
        }
    }

    fn gen_switch_pattern(&self, pattern: &SwitchPattern) -> String {
        match pattern {
            SwitchPattern::Default => "true".to_string(),
            SwitchPattern::Exact(e) => {
                let v = self.gen_expr(e);
                format!("rval_eq(&_rnd_sw, &{v})")
            }
            SwitchPattern::Comparison(op, e) => {
                let v = self.gen_expr(e);
                match op {
                    CmpOp::Lt    => format!("rval_lt(&_rnd_sw, &{v})"),
                    CmpOp::LtEq  => format!("rval_lteq(&_rnd_sw, &{v})"),
                    CmpOp::Gt    => format!("rval_gt(&_rnd_sw, &{v})"),
                    CmpOp::GtEq  => format!("rval_gteq(&_rnd_sw, &{v})"),
                    CmpOp::Eq    => format!("rval_eq(&_rnd_sw, &{v})"),
                    CmpOp::NotEq => format!("rval_neq(&_rnd_sw, &{v})"),
                }
            }
        }
    }

    fn emit_for(&mut self, fl: &ForLoopStmt) {
        let start = self.gen_expr(&fl.start);
        let end   = self.gen_expr(&fl.end);
        let inc   = self.gen_expr(&fl.increment);
        let var_lhs = self.var_lhs(&fl.var);

        self.emit_line("{");
        self.indent += 1;
        self.emit_line(&format!("if let (RVal::Int(mut _rnd_i), RVal::Int(_rnd_end), RVal::Int(_rnd_inc)) = ({start}, {end}, {inc}) {{"));
        self.indent += 1;
        self.emit_line("while (_rnd_inc > 0 && _rnd_i <= _rnd_end) || (_rnd_inc < 0 && _rnd_i >= _rnd_end) {");
        self.indent += 1;
        self.emit_line(&format!("{var_lhs} = RVal::Int(_rnd_i);"));
        for stmt in &fl.body { self.emit_stmt(stmt); }
        self.emit_line("_rnd_i = _rnd_i.wrapping_add(_rnd_inc);");
        self.indent -= 1;
        self.emit_line("}");
        self.indent -= 1;
        self.emit_line("}");
        self.indent -= 1;
        self.emit_line("}");
    }

    fn emit_while(&mut self, wl: &WhileLoopStmt) {
        let cond = self.gen_expr(&wl.condition);
        self.emit_line(&format!("while {{ let _rnd_c = {cond}; _rnd_c.is_truthy() }} {{"));
        self.indent += 1;
        for stmt in &wl.body { self.emit_stmt(stmt); }
        self.indent -= 1;
        self.emit_line("}");
    }

    fn emit_foreach(&mut self, fe: &ForEachStmt) {
        let col = self.gen_expr(&fe.collection);
        let var_lhs = self.var_lhs(&fe.var);

        self.emit_line("{");
        self.indent += 1;
        self.emit_line(&format!("if let RVal::Json(serde_json::Value::Array(_rnd_arr)) = {col} {{"));
        self.indent += 1;
        self.emit_line("for _rnd_elem in _rnd_arr.into_iter() {");
        self.indent += 1;
        self.emit_line(&format!("{var_lhs} = json_to_rval(&_rnd_elem);"));
        for stmt in &fe.body { self.emit_stmt(stmt); }
        self.indent -= 1;
        self.emit_line("}");
        self.indent -= 1;
        self.emit_line("} else { panic!(\"for each: collection must be a json array\"); }");
        self.indent -= 1;
        self.emit_line("}");
    }

    fn emit_return(&mut self, expr: &Option<Expr>) {
        match expr {
            None => self.emit_line("return RVal::Null;"),
            Some(e) => {
                let v = self.gen_expr(e);
                self.emit_line(&format!("return {v};"));
            }
        }
    }

    fn emit_try(&mut self, tc: &TryCatchStmt) {
        // Best-effort: emit try body, then emit catch bodies guarded by
        // a constant `false` condition (they won't run, but the code compiles).
        self.emit_line("// try block (catch not fully supported in compiled mode)");
        self.emit_line("{");
        self.indent += 1;
        for stmt in &tc.try_body { self.emit_stmt(stmt); }
        self.indent -= 1;
        self.emit_line("}");
        if let Some(finally_body) = &tc.finally_body {
            self.emit_line("// finally block");
            self.emit_line("{");
            self.indent += 1;
            for stmt in finally_body { self.emit_stmt(stmt); }
            self.indent -= 1;
            self.emit_line("}");
        }
    }

    fn emit_remove(&mut self, expr: &Expr) {
        if let Expr::Index(col_expr, key_expr) = expr {
            if let Some(root) = extract_root_ident(col_expr) {
                let key = self.gen_expr(key_expr);
                let mr = self.var_mut_ref(&root);
                self.emit_line(&format!("rnd_remove_key({mr}, {key});"));
            } else {
                self.emit_line("panic!(\"remove: complex lvalue not supported in compiled mode\");");
            }
        } else {
            self.emit_line("panic!(\"remove requires an index expression\");");
        }
    }

    fn emit_append_stmt(&mut self, col_expr: &Expr, val_expr: &Expr) {
        if let Some(root) = extract_root_ident(col_expr) {
            let val = self.gen_expr(val_expr);
            let mr = self.var_mut_ref(&root);
            self.emit_line(&format!("rnd_append({mr}, {val});"));
        } else {
            self.emit_line("panic!(\"append: complex lvalue not supported in compiled mode\");");
        }
    }

    // ── Expression generation ─────────────────────────────────

    pub fn gen_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Literal(lit) => self.gen_literal(lit),
            Expr::Identifier(name) => self.var_get(name),
            Expr::BinaryOp(lhs, op, rhs) => self.gen_binop(lhs, op, rhs),
            Expr::UnaryOp(op, inner) => self.gen_unaryop(op, inner),
            Expr::Index(col, key) => {
                let c = self.gen_expr(col);
                let k = self.gen_expr(key);
                format!("rval_index({c}, {k})")
            }
            Expr::Call(name, args) => self.gen_call(name, args),
            Expr::IsNull(e) => {
                let v = self.gen_expr(e);
                format!("RVal::Bool({v}.is_null())")
            }
            Expr::IsNotNull(e) => {
                let v = self.gen_expr(e);
                format!("RVal::Bool(!{v}.is_null())")
            }
            Expr::JsonLiteral(val) => {
                let json_str = serde_json::to_string(val).unwrap_or_default();
                let escaped = json_str.replace('\\', "\\\\").replace('"', "\\\"");
                format!("RVal::Json(serde_json::from_str(\"{escaped}\").unwrap())")
            }
            // GUI / query expressions — not supported.
            Expr::ObjectPath(_) | Expr::ShowForm { .. } | Expr::CloseForm { .. }
            | Expr::Dialog(_) | Expr::Await(_) => {
                "panic!(\"GUI/query expression not supported in compiled mode\")".to_string()
            }
            Expr::PixelValue(n) => format!("RVal::Int({n} as i64)"),
            Expr::DurationValue(ms) => format!("RVal::Int({ms} as i64)"),
            Expr::PositionLiteral(t, l, w, h) => {
                let json_str = format!("[{t},{l},{w},{h}]");
                format!("RVal::Json(serde_json::from_str(\"{json_str}\").unwrap())")
            }
        }
    }

    fn gen_literal(&self, lit: &Literal) -> String {
        match lit {
            Literal::Integer(n)  => format!("RVal::Int({n}i64)"),
            Literal::Float(f)    => format!("RVal::Float({f}f64)"),
            Literal::Str(s)      => {
                let escaped = s.replace('\\', "\\\\").replace('"', "\\\"")
                               .replace('\n', "\\n").replace('\r', "\\r")
                               .replace('\t', "\\t");
                format!("RVal::Str(\"{escaped}\".to_string())")
            }
            Literal::Currency(c) => format!("RVal::Currency({c}i64)"),
            Literal::Boolean(b)  => format!("RVal::Bool({b})"),
            Literal::DateTime(s) => {
                // s is an ISO 8601 string; parse at runtime.
                let escaped = s.replace('"', "\\\"");
                format!("RVal::DateTime(chrono::DateTime::parse_from_rfc3339(\"{escaped}\").unwrap_or_else(|_| chrono::DateTime::parse_from_str(\"{escaped}\", \"%Y-%m-%dT%H:%M:%S%z\").unwrap()))")
            }
            Literal::Null => "RVal::Null".to_string(),
        }
    }

    fn gen_binop(&self, lhs: &Expr, op: &BinOp, rhs: &Expr) -> String {
        let l = self.gen_expr(lhs);
        let r = self.gen_expr(rhs);
        match op {
            BinOp::Add | BinOp::StrConcat => format!("rval_add({l}, {r})"),
            BinOp::Sub => format!("rval_sub({l}, {r})"),
            BinOp::Mul => format!("rval_mul({l}, {r})"),
            BinOp::Div => format!("rval_div({l}, {r})"),
            BinOp::Mod => format!("rval_mod({l}, {r})"),
            BinOp::Pow => format!("rval_pow({l}, {r})"),
            BinOp::Eq    => format!("RVal::Bool(rval_eq(&{l}, &{r}))"),
            BinOp::NotEq  => format!("RVal::Bool(rval_neq(&{l}, &{r}))"),
            BinOp::Lt    => format!("RVal::Bool(rval_lt(&{l}, &{r}))"),
            BinOp::LtEq  => format!("RVal::Bool(rval_lteq(&{l}, &{r}))"),
            BinOp::Gt    => format!("RVal::Bool(rval_gt(&{l}, &{r}))"),
            BinOp::GtEq  => format!("RVal::Bool(rval_gteq(&{l}, &{r}))"),
            BinOp::And   => format!("RVal::Bool({l}.is_truthy() && {r}.is_truthy())"),
            BinOp::Or    => format!("{{ let _a = {l}; if _a.is_truthy() {{ _a }} else {{ {r} }} }}"),
        }
    }

    fn gen_unaryop(&self, op: &UnaryOp, inner: &Expr) -> String {
        let v = self.gen_expr(inner);
        match op {
            UnaryOp::Neg => format!("(match {v} {{ RVal::Int(n) => RVal::Int(-n), RVal::Float(f) => RVal::Float(-f), RVal::Currency(c) => RVal::Currency(-c), other => panic!(\"negation requires numeric, got {{}}\", other.type_name()) }})"),
            UnaryOp::Not => format!("RVal::Bool(!{v}.is_truthy())"),
        }
    }

    fn gen_call(&self, name: &str, args: &[Expr]) -> String {
        // User-defined functions first.
        if self.functions.contains_key(name) {
            let arg_strs: String = args.iter()
                .map(|a| self.gen_expr(a))
                .collect::<Vec<_>>()
                .join(", ");
            // Always use &mut ctx so that the call works both from main()
            // (where ctx: Ctx) and from other user functions (where ctx: &mut Ctx).
            return if arg_strs.is_empty() {
                format!("usr_{}(&mut ctx)", safe_ident(name))
            } else {
                format!("usr_{}(&mut ctx, {arg_strs})", safe_ident(name))
            };
        }

        // Built-ins.
        self.gen_builtin(name, args)
    }

    fn gen_builtin(&self, name: &str, args: &[Expr]) -> String {
        let a = |i: usize| self.gen_expr(&args[i]);
        match name {
            "newline"        => "rnd_newline()".to_string(),
            "now"            => "rnd_now()".to_string(),
            "os"             => "rnd_os()".to_string(),
            "string"         => format!("rnd_string({})",       a(0)),
            "integer"        => format!("rnd_integer({})",      a(0)),
            "float"          => format!("rnd_float({})",        a(0)),
            "boolean"        => format!("rnd_boolean({})",      a(0)),
            "cast"           => format!("rnd_cast({}, {})",     a(0), a(1)),
            "length"         => format!("rnd_length({})",       a(0)),
            "abs"            => format!("rnd_abs({})",          a(0)),
            "floor"          => format!("rnd_floor({})",        a(0)),
            "ceil"           => format!("rnd_ceil({})",         a(0)),
            "round"          => format!("rnd_round({}, {})",    a(0), a(1)),
            "sqrt"           => format!("rnd_sqrt({})",         a(0)),
            "pow"            => format!("rnd_pow({}, {})",      a(0), a(1)),
            "min"            => format!("rnd_min({}, {})",      a(0), a(1)),
            "max"            => format!("rnd_max({}, {})",      a(0), a(1)),
            "clamp"          => format!("rnd_clamp({}, {}, {})", a(0), a(1), a(2)),
            "upper"          => format!("rnd_upper({})",        a(0)),
            "lower"          => format!("rnd_lower({})",        a(0)),
            "trim"           => format!("rnd_trim({})",         a(0)),
            "substr"         => format!("rnd_substr({}, {}, {})", a(0), a(1), a(2)),
            "replace"        => format!("rnd_replace({}, {}, {})", a(0), a(1), a(2)),
            "split"          => format!("rnd_split({}, {})",    a(0), a(1)),
            "join"           => format!("rnd_join({}, {})",     a(0), a(1)),
            "startswith"     => format!("rnd_startswith({}, {})", a(0), a(1)),
            "endswith"       => format!("rnd_endswith({}, {})", a(0), a(1)),
            "contains"       => format!("rnd_contains({}, {})", a(0), a(1)),
            "keys"           => format!("rnd_keys({})",         a(0)),
            "values"         => format!("rnd_values({})",       a(0)),
            "has_key"        => format!("rnd_has_key({}, {})",  a(0), a(1)),
            "type"           => format!("rnd_type_of(&{})",     a(0)),
            "isnull"         => format!("rnd_isnull(&{})",      a(0)),
            "exists"         => format!("rnd_exists({})",       a(0)),
            "delete"         => format!("rnd_delete({})",       a(0)),
            "mkdir"          => format!("rnd_mkdir({})",        a(0)),
            "sleep"          => format!("rnd_sleep({})",        a(0)),
            "read_text"      => format!("rnd_read_text({})",    a(0)),
            "write_text"     => format!("rnd_write_text({}, {})", a(0), a(1)),
            "read_json"      => format!("rnd_read_json({})",    a(0)),
            "write_json"     => format!("rnd_write_json({}, {})", a(0), a(1)),
            "read_csv"       => format!("rnd_read_csv({}, {})", a(0), a(1)),
            "write_csv"      => format!("rnd_write_csv({}, {}, {})", a(0), a(1), a(2)),
            "day"            => format!("rnd_day({})",          a(0)),
            "month"          => format!("rnd_month({})",        a(0)),
            "year"           => format!("rnd_year({})",         a(0)),
            "hour"           => format!("rnd_hour({})",         a(0)),
            "minute"         => format!("rnd_minute({})",       a(0)),
            "second"         => format!("rnd_second({})",       a(0)),
            "dateformat"     => format!("rnd_dateformat({}, {})", a(0), a(1)),
            "timestamp"      => format!("rnd_timestamp({})",    a(0)),
            "fromtimestamp"  => format!("rnd_fromtimestamp({})", a(0)),
            "dayofweek"      => format!("rnd_dayofweek({})",    a(0)),
            "adddays"        => format!("rnd_adddays({}, {})",  a(0), a(1)),
            "addhours"       => format!("rnd_addhours({}, {})", a(0), a(1)),
            "diffdays"       => format!("rnd_diffdays({}, {})", a(0), a(1)),
            "timezone"       => format!("rnd_timezone({})",     a(0)),
            "execute"        => format!("rnd_execute({})",      a(0)),
            // Mutable builtins — need a root identifier.
            "append" => {
                if let Some(root) = extract_root_ident(&args[0]) {
                    let val = a(1);
                    let mr = self.var_mut_ref_for(&root);
                    format!("{{ rnd_append({mr}, {val}); RVal::Null }}")
                } else {
                    "panic!(\"append(): first argument must be a simple variable\")".to_string()
                }
            }
            "remove_at" => {
                if let Some(root) = extract_root_ident(&args[0]) {
                    let idx = a(1);
                    let mr = self.var_mut_ref_for(&root);
                    format!("{{ rnd_remove_at({mr}, {idx}); RVal::Null }}")
                } else {
                    "panic!(\"remove_at(): first argument must be a simple variable\")".to_string()
                }
            }
            "env" | "env_exists" => {
                format!("panic!(\"env() / env_exists() not supported in compiled mode\")")
            }
            other => {
                format!("panic!(\"unknown built-in: {other}\")")
            }
        }
    }

    /// Variant of `var_mut_ref` usable from `gen_expr` (which takes `&self`).
    fn var_mut_ref_for(&self, name: &str) -> String {
        self.var_mut_ref(name)
    }
}

// ── Helpers ───────────────────────────────────────────────────

/// Return the root identifier of an expression like `contacts["alice"]`.
fn extract_root_ident(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Identifier(name) => Some(name.clone()),
        Expr::Index(inner, _)  => extract_root_ident(inner),
        _ => None,
    }
}

/// Collect the index path below the root identifier (i.e., all the keys except the final one).
fn collect_path_after_root(expr: &Expr) -> Vec<Expr> {
    match expr {
        Expr::Identifier(_) => vec![],
        Expr::Index(inner, key) => {
            let mut path = collect_path_after_root(inner);
            path.push(*key.clone());
            path
        }
        _ => vec![],
    }
}
