use crate::scanner::Token;

pub(crate) struct Parser {
  scanner: Vec<Token>,
  current_ndx: usize,
}

impl Parser {
  pub(crate) fn new(tokens: Vec<Token>) -> Self {
    Parser {
      scanner: tokens,
      current_ndx: 0,
    }
  }

  fn error(&self, msg: &str, expected: &[&str]) {
    let token = self.scanner[self.current_ndx].clone();
    panic!("Parse error {} at {} {:?}:  Expected {:?}", msg, token.value, token.span, expected);
  }

  fn match_kind(&mut self, kind: &str) -> Token {
    if self.current() == kind {
      let prev = self.scanner[self.current_ndx].clone();
      self.current_ndx += 1;
      return prev;
    } else {
      self.error("", &[kind]);
    }
    self.scanner[self.current_ndx].clone()
  }

  fn current(&self) -> &str {
    self.scanner[self.current_ndx].kind.as_str()
  }

  pub(crate) fn parse(&mut self) {
    self.program();
    self.match_kind("EOF");
  }

  fn program(&mut self) {
    if ["const", "entry", "fn", "fragment", "in", "let", "loc", "out", "vertex"].contains(&self.current()) {
      self.global_stmt();
      self.global_stmt_list();
    } else {
      self.error("syntax error", &["const", "entry", "fn", "fragment", "in", "let", "loc", "out", "vertex"]);
    }
  }

  fn global_stmt_list(&mut self) {
    if ["const", "entry", "fn", "fragment", "in", "let", "loc", "out", "vertex"].contains(&self.current()) {
      self.global_stmt();
      self.global_stmt_list();
    } else if ["EOF"].contains(&self.current()) {
      // do nothing
    } else {
      self.error("syntax error", &["EOF", "const", "entry", "fn", "fragment", "in", "let", "loc", "out", "vertex"]);
    }
  }

  fn global_stmt(&mut self) {
    if ["entry", "fn", "fragment", "vertex"].contains(&self.current()) {
      self.function();
    } else if ["const", "in", "let", "loc", "out"].contains(&self.current()) {
      self.global_var_decl();
    } else {
      self.error("syntax error", &["const", "entry", "fn", "fragment", "in", "let", "loc", "out", "vertex"]);
    }
  }

  fn function(&mut self) {
    if ["entry", "fn", "fragment", "vertex"].contains(&self.current()) {
      self.function_def();
      self.compound_stmt();
    } else {
      self.error("syntax error", &["entry", "fn", "fragment", "vertex"]);
    }
  }

  fn stage_attr(&mut self) {
    if ["vertex"].contains(&self.current()) {
      self.match_kind("vertex");
    } else if ["fragment"].contains(&self.current()) {
      self.match_kind("fragment");
    } else if ["entry", "fn"].contains(&self.current()) {
      // do nothing
    } else {
      self.error("syntax error", &["entry", "fn", "fragment", "vertex"]);
    }
  }

  fn entry_attr(&mut self) {
    if ["entry"].contains(&self.current()) {
      self.match_kind("entry");
    } else if ["fn"].contains(&self.current()) {
      // do nothing
    } else {
      self.error("syntax error", &["entry", "fn"]);
    }
  }

  fn function_def(&mut self) {
    if ["entry", "fn", "fragment", "vertex"].contains(&self.current()) {
      self.stage_attr();
      self.entry_attr();
      self.match_kind("fn");
      self.match_kind("ID");
      self.parameters();
      self.match_kind("->");
      self.types();
    } else {
      self.error("syntax error", &["entry", "fn", "fragment", "vertex"]);
    }
  }

  fn parameters(&mut self) {
    if ["("].contains(&self.current()) {
      self.match_kind("(");
      self.param_list();
      self.match_kind(")");
    } else {
      self.error("syntax error", &["("]);
    }
  }

  fn param_list(&mut self) {
    if ["const", "let"].contains(&self.current()) {
      self.param_list_non_empty();
    } else if [")"].contains(&self.current()) {
      // do nothing
    } else {
      self.error("syntax error", &[")", "const", "let"]);
    }
  }

  fn param_list_non_empty(&mut self) {
    if ["const", "let"].contains(&self.current()) {
      self.var_decl();
      self.param_list_tail();
    } else {
      self.error("syntax error", &["const", "let"]);
    }
  }

  fn param_list_tail(&mut self) {
    if [","].contains(&self.current()) {
      self.match_kind(",");
      self.param_list_non_empty();
    } else if [")"].contains(&self.current()) {
      // do nothing
    } else {
      self.error("syntax error", &[")", ","]);
    }
  }

  fn global_var_decl(&mut self) {
    if ["const", "in", "let", "loc", "out"].contains(&self.current()) {
      self.opt_attr_list();
      self.var_decl();
      self.match_kind(";");
    } else {
      self.error("syntax error", &["const", "in", "let", "loc", "out"]);
    }
  }

  fn opt_attr_list(&mut self) {
    if ["in", "loc", "out"].contains(&self.current()) {
      self.attrs();
      self.opt_attr_list();
    } else if ["const", "let"].contains(&self.current()) {
      // do nothing
    } else {
      self.error("syntax error", &["const", "in", "let", "loc", "out"]);
    }
  }

  fn attrs(&mut self) {
    if ["out"].contains(&self.current()) {
      self.match_kind("out");
    } else if ["in"].contains(&self.current()) {
      self.match_kind("in");
    } else if ["loc"].contains(&self.current()) {
      self.match_kind("loc");
      self.match_kind("(");
      self.match_kind("NUM_LIT");
      self.match_kind(")");
    } else {
      self.error("syntax error", &["in", "loc", "out"]);
    }
  }

  fn compound_stmt(&mut self) {
    if ["{"].contains(&self.current()) {
      self.match_kind("{");
      self.stmt_list();
      self.match_kind("}");
    } else {
      self.error("syntax error", &["{"]);
    }
  }

  fn stmt_list(&mut self) {
    if ["const", "let", "return"].contains(&self.current()) {
      self.stmt();
      self.stmt_list();
    } else if ["}"].contains(&self.current()) {
      // do nothing
    } else {
      self.error("syntax error", &["}", "const", "let", "return"]);
    }
  }

  fn stmt(&mut self) {
    if ["return"].contains(&self.current()) {
      self.match_kind("return");
      self.expr();
      self.match_kind(";");
    } else if ["const", "let"].contains(&self.current()) {
      self.var_decl();
      self.match_kind(";");
    } else {
      self.error("syntax error", &["const", "let", "return"]);
    }
  }

  fn var_decl(&mut self) {
    if ["const", "let"].contains(&self.current()) {
      self.var_decl_keyword();
      self.match_kind("ID");
      self.match_kind(":");
      self.types();
      self.opt_assignment();
    } else if ["const", "let"].contains(&self.current()) {
      self.var_decl_keyword();
      self.match_kind("ID");
      self.match_kind("=");
      self.expr();
    } else {
      self.error("syntax error", &["const", "let"]);
    }
  }

  fn var_decl_keyword(&mut self) {
    if ["let"].contains(&self.current()) {
      self.match_kind("let");
    } else if ["const"].contains(&self.current()) {
      self.match_kind("const");
    } else {
      self.error("syntax error", &["const", "let"]);
    }
  }

  fn opt_assignment(&mut self) {
    if ["="].contains(&self.current()) {
      self.match_kind("=");
      self.expr();
    } else if [")", ",", ";"].contains(&self.current()) {
      // do nothing
    } else {
      self.error("syntax error", &[")", ",", ";", "="]);
    }
  }

  fn expr(&mut self) {
    if ["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      self.prod_expr();
      self.expr_tail();
    } else {
      self.error("syntax error", &["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"]);
    }
  }

  fn expr_tail(&mut self) {
    if ["or"].contains(&self.current()) {
      self.match_kind("or");
      self.expr();
    } else if [")", ",", ";", "]"].contains(&self.current()) {
      // do nothing
    } else {
      self.error("syntax error", &[")", ",", ";", "]", "or"]);
    }
  }

  fn prod_expr(&mut self) {
    if ["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      self.comp_expr();
      self.prod_expr_tail();
    } else {
      self.error("syntax error", &["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"]);
    }
  }

  fn prod_expr_tail(&mut self) {
    if ["and"].contains(&self.current()) {
      self.match_kind("and");
      self.prod_expr();
    } else if [")", ",", ";", "]", "or"].contains(&self.current()) {
      // do nothing
    } else {
      self.error("syntax error", &[")", ",", ";", "]", "and", "or"]);
    }
  }

  fn comp_expr(&mut self) {
    if ["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      self.add_expr();
      self.comp_expr_tail();
    } else {
      self.error("syntax error", &["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"]);
    }
  }

  fn comp_expr_tail(&mut self) {
    if ["!=", "&", "<", "<=", "==", ">", ">=", "|"].contains(&self.current()) {
      self.comp_binops();
      self.comp_expr();
    } else if [")", ",", ";", "]", "and", "or"].contains(&self.current()) {
      // do nothing
    } else {
      self.error("syntax error", &["!=", "&", ")", ",", ";", "<", "<=", "==", ">", ">=", "]", "|", "and", "or"]);
    }
  }

  fn add_expr(&mut self) {
    if ["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      self.mul_expr();
      self.add_expr_tail();
    } else {
      self.error("syntax error", &["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"]);
    }
  }

  fn add_expr_tail(&mut self) {
    if ["+", "-"].contains(&self.current()) {
      self.add_binops();
      self.add_expr();
    } else if ["!=", "&", ")", ",", ";", "<", "<=", "==", ">", ">=", "]", "|", "and", "or"].contains(&self.current()) {
      // do nothing
    } else {
      self.error("syntax error", &["!=", "&", ")", "+", ",", "-", ";", "<", "<=", "==", ">", ">=", "]", "|", "and", "or"]);
    }
  }

  fn mul_expr(&mut self) {
    if ["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      self.unary_expr();
      self.mul_expr_tail();
    } else {
      self.error("syntax error", &["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"]);
    }
  }

  fn mul_expr_tail(&mut self) {
    if ["%", "*", "/"].contains(&self.current()) {
      self.mul_binops();
      self.mul_expr();
    } else if ["!=", "&", ")", "+", ",", "-", ";", "<", "<=", "==", ">", ">=", "]", "|", "and", "or"].contains(&self.current()) {
      // do nothing
    } else {
      self.error("syntax error", &["!=", "%", "&", ")", "*", "+", ",", "-", "/", ";", "<", "<=", "==", ">", ">=", "]", "|", "and", "or"]);
    }
  }

  fn unary_expr(&mut self) {
    if ["(", "[", "ID", "NUM_LIT", "false", "true", "vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      self.base_expr();
    } else if ["-", "not"].contains(&self.current()) {
      self.unary_expr_head();
    } else {
      self.error("syntax error", &["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"]);
    }
  }

  fn unary_expr_head(&mut self) {
    if ["-", "not"].contains(&self.current()) {
      self.unary_ops();
      self.unary_expr();
    } else {
      self.error("syntax error", &["-", "not"]);
    }
  }

  fn base_expr(&mut self) {
    if ["vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      self.type_constructor();
    } else if ["ID"].contains(&self.current()) {
      self.id_expr();
    } else if ["["].contains(&self.current()) {
      self.array_literal();
    } else if ["NUM_LIT"].contains(&self.current()) {
      self.match_kind("NUM_LIT");
    } else if ["true"].contains(&self.current()) {
      self.match_kind("true");
    } else if ["false"].contains(&self.current()) {
      self.match_kind("false");
    } else if ["("].contains(&self.current()) {
      self.grouped_expr();
    } else {
      self.error("syntax error", &["(", "[", "ID", "NUM_LIT", "false", "true", "vec2", "vec3", "vec4", "void"]);
    }
  }

  fn id_expr(&mut self) {
    if ["ID"].contains(&self.current()) {
      self.match_kind("ID");
      self.id_expr_tail();
    } else {
      self.error("syntax error", &["ID"]);
    }
  }

  fn id_expr_tail(&mut self) {
    if ["["].contains(&self.current()) {
      self.match_kind("[");
      self.expr();
      self.match_kind("]");
    } else if ["!=", "%", "&", ")", "*", "+", ",", "-", "/", ";", "<", "<=", "==", ">", ">=", "]", "|", "and", "or"].contains(&self.current()) {
      // do nothing
    } else {
      self.error("syntax error", &["!=", "%", "&", ")", "*", "+", ",", "-", "/", ";", "<", "<=", "==", ">", ">=", "[", "]", "|", "and", "or"]);
    }
  }

  fn array_literal(&mut self) {
    if ["["].contains(&self.current()) {
      self.match_kind("[");
      self.expr_list();
      self.match_kind("]");
    } else {
      self.error("syntax error", &["["]);
    }
  }

  fn type_constructor(&mut self) {
    if ["vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      self.base_types();
      self.func_call_args();
    } else {
      self.error("syntax error", &["vec2", "vec3", "vec4", "void"]);
    }
  }

  fn func_call_args(&mut self) {
    if ["("].contains(&self.current()) {
      self.match_kind("(");
      self.expr_list();
      self.match_kind(")");
    } else {
      self.error("syntax error", &["("]);
    }
  }

  fn expr_list(&mut self) {
    if ["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      self.expr();
      self.expr_list_tail();
    } else {
      self.error("syntax error", &["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"]);
    }
  }

  fn expr_list_tail(&mut self) {
    if [","].contains(&self.current()) {
      self.match_kind(",");
      self.expr_list();
    } else if [")", "]"].contains(&self.current()) {
      // do nothing
    } else {
      self.error("syntax error", &[")", ",", "]"]);
    }
  }

  fn grouped_expr(&mut self) {
    if ["("].contains(&self.current()) {
      self.match_kind("(");
      self.expr();
      self.match_kind(")");
    } else {
      self.error("syntax error", &["("]);
    }
  }

  fn mul_binops(&mut self) {
    if ["*"].contains(&self.current()) {
      self.match_kind("*");
    } else if ["/"].contains(&self.current()) {
      self.match_kind("/");
    } else if ["%"].contains(&self.current()) {
      self.match_kind("%");
    } else {
      self.error("syntax error", &["%", "*", "/"]);
    }
  }

  fn add_binops(&mut self) {
    if ["+"].contains(&self.current()) {
      self.match_kind("+");
    } else if ["-"].contains(&self.current()) {
      self.match_kind("-");
    } else {
      self.error("syntax error", &["+", "-"]);
    }
  }

  fn comp_binops(&mut self) {
    if ["!="].contains(&self.current()) {
      self.match_kind("!=");
    } else if ["&"].contains(&self.current()) {
      self.match_kind("&");
    } else if ["<="].contains(&self.current()) {
      self.match_kind("<=");
    } else if ["<"].contains(&self.current()) {
      self.match_kind("<");
    } else if [">="].contains(&self.current()) {
      self.match_kind(">=");
    } else if [">"].contains(&self.current()) {
      self.match_kind(">");
    } else if ["=="].contains(&self.current()) {
      self.match_kind("==");
    } else if ["|"].contains(&self.current()) {
      self.match_kind("|");
    } else {
      self.error("syntax error", &["!=", "&", "<", "<=", "==", ">", ">=", "|"]);
    }
  }

  fn unary_ops(&mut self) {
    if ["-"].contains(&self.current()) {
      self.match_kind("-");
    } else if ["not"].contains(&self.current()) {
      self.match_kind("not");
    } else {
      self.error("syntax error", &["-", "not"]);
    }
  }

  fn types(&mut self) {
    if ["vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      self.base_types();
      self.opt_indexing();
    } else {
      self.error("syntax error", &["vec2", "vec3", "vec4", "void"]);
    }
  }

  fn opt_indexing(&mut self) {
    if ["["].contains(&self.current()) {
      self.match_kind("[");
      self.match_kind("NUM_LIT");
      self.match_kind("]");
    } else if [")", ",", ";", "=", "{"].contains(&self.current()) {
      // do nothing
    } else {
      self.error("syntax error", &[")", ",", ";", "=", "[", "{"]);
    }
  }

  fn base_types(&mut self) {
    if ["void"].contains(&self.current()) {
      self.match_kind("void");
    } else if ["vec2"].contains(&self.current()) {
      self.match_kind("vec2");
    } else if ["vec3"].contains(&self.current()) {
      self.match_kind("vec3");
    } else if ["vec4"].contains(&self.current()) {
      self.match_kind("vec4");
    } else {
      self.error("syntax error", &["vec2", "vec3", "vec4", "void"]);
    }
  }
}