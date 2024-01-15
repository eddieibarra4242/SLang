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
    if ["loc", "fn", "fragment", "let", "out", "in", "vertex", "entry"].contains(&self.current()) {
      self.global_stmt();
      self.global_stmt_list();
    } else {
      self.error("syntax error", &["out", "in", "vertex", "let", "entry", "fragment", "loc", "fn"]);
    }
  }

  fn global_stmt_list(&mut self) {
    if ["vertex", "let", "fragment", "in", "out", "loc", "fn", "entry"].contains(&self.current()) {
      self.global_stmt();
      self.global_stmt_list();
    } else if ["EOF"].contains(&self.current()) {
      // do nothing
    } else {
      self.error("syntax error", &["vertex", "let", "fn", "loc", "entry", "fragment", "in", "out", "EOF"]);
    }
  }

  fn global_stmt(&mut self) {
    if ["fragment", "vertex", "entry", "fn"].contains(&self.current()) {
      self.function();
    } else if ["loc", "in", "out", "let"].contains(&self.current()) {
      self.scoped_var_decl();
    } else {
      self.error("syntax error", &["loc", "vertex", "out", "in", "fn", "fragment", "let", "entry"]);
    }
  }

  fn function(&mut self) {
    if ["entry", "fn", "vertex", "fragment"].contains(&self.current()) {
      self.function_def();
      self.compound_stmt();
    } else {
      self.error("syntax error", &["fn", "vertex", "entry", "fragment"]);
    }
  }

  fn stage_attr(&mut self) {
    if ["vertex"].contains(&self.current()) {
      self.match_kind("vertex");
    } else if ["fragment"].contains(&self.current()) {
      self.match_kind("fragment");
    } else if ["fn", "entry"].contains(&self.current()) {
      // do nothing
    } else {
      self.error("syntax error", &["vertex", "entry", "fragment", "fn"]);
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
    if ["fragment", "entry", "vertex", "fn"].contains(&self.current()) {
      self.stage_attr();
      self.entry_attr();
      self.match_kind("fn");
      self.match_kind("ID");
      self.parameters();
      self.match_kind("->");
      self.types();
    } else {
      self.error("syntax error", &["fragment", "entry", "vertex", "fn"]);
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
    if ["ID"].contains(&self.current()) {
      self.param_list_non_empty();
    } else if [")"].contains(&self.current()) {
      // do nothing
    } else {
      self.error("syntax error", &["ID", ")"]);
    }
  }

  fn param_list_non_empty(&mut self) {
    if ["ID"].contains(&self.current()) {
      self.var_decl();
      self.param_list_tail();
    } else {
      self.error("syntax error", &["ID"]);
    }
  }

  fn param_list_tail(&mut self) {
    if [","].contains(&self.current()) {
      self.match_kind(",");
      self.param_list_non_empty();
    } else if [")"].contains(&self.current()) {
      // do nothing
    } else {
      self.error("syntax error", &[",", ")"]);
    }
  }

  fn scoped_var_decl(&mut self) {
    if ["let", "loc", "out", "in"].contains(&self.current()) {
      self.opt_attr_list();
      self.match_kind("let");
      self.var_decl();
      self.opt_assignment();
      self.match_kind(";");
    } else {
      self.error("syntax error", &["let", "out", "loc", "in"]);
    }
  }

  fn opt_assignment(&mut self) {
    if ["="].contains(&self.current()) {
      self.match_kind("=");
      self.expr();
    } else if [";"].contains(&self.current()) {
      // do nothing
    } else {
      self.error("syntax error", &["=", ";"]);
    }
  }

  fn opt_attr_list(&mut self) {
    if ["out", "loc", "in"].contains(&self.current()) {
      self.attrs();
      self.opt_attr_list();
    } else if ["let"].contains(&self.current()) {
      // do nothing
    } else {
      self.error("syntax error", &["loc", "let", "out", "in"]);
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
      self.error("syntax error", &["loc", "in", "out"]);
    }
  }

  fn var_decl(&mut self) {
    if ["ID"].contains(&self.current()) {
      self.match_kind("ID");
      self.match_kind(":");
      self.types();
    } else {
      self.error("syntax error", &["ID"]);
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
    if ["ID", "return"].contains(&self.current()) {
      self.stmt();
      self.stmt_list();
    } else if ["}"].contains(&self.current()) {
      // do nothing
    } else {
      self.error("syntax error", &["}", "ID", "return"]);
    }
  }

  fn stmt(&mut self) {
    if ["return"].contains(&self.current()) {
      self.match_kind("return");
      self.expr();
      self.match_kind(";");
    } else if ["ID"].contains(&self.current()) {
      self.match_kind("ID");
      self.match_kind("=");
      self.expr();
      self.match_kind(";");
    } else {
      self.error("syntax error", &["return", "ID"]);
    }
  }

  fn expr(&mut self) {
    if ["void", "(", "false", "not", "[", "NUM_LIT", "-", "vec2", "ID", "true", "vec3", "vec4"].contains(&self.current()) {
      self.prod_expr();
      self.expr_tail();
    } else {
      self.error("syntax error", &["-", "ID", "vec3", "true", "void", "not", "[", "false", "(", "vec2", "vec4", "NUM_LIT"]);
    }
  }

  fn expr_tail(&mut self) {
    if ["or"].contains(&self.current()) {
      self.match_kind("or");
      self.expr();
    } else if ["]", ";", ")", ","].contains(&self.current()) {
      // do nothing
    } else {
      self.error("syntax error", &[")", "]", "or", ",", ";"]);
    }
  }

  fn prod_expr(&mut self) {
    if ["vec2", "vec3", "-", "true", "NUM_LIT", "vec4", "[", "not", "void", "false", "ID", "("].contains(&self.current()) {
      self.comp_expr();
      self.prod_expr_tail();
    } else {
      self.error("syntax error", &["-", "NUM_LIT", "vec3", "not", "void", "(", "true", "ID", "vec4", "[", "vec2", "false"]);
    }
  }

  fn prod_expr_tail(&mut self) {
    if ["and"].contains(&self.current()) {
      self.match_kind("and");
      self.prod_expr();
    } else if [";", "or", ")", "]", ","].contains(&self.current()) {
      // do nothing
    } else {
      self.error("syntax error", &["]", ";", ")", ",", "or", "and"]);
    }
  }

  fn comp_expr(&mut self) {
    if ["-", "NUM_LIT", "true", "false", "(", "ID", "[", "not", "vec3", "vec4", "void", "vec2"].contains(&self.current()) {
      self.add_expr();
      self.comp_expr_tail();
    } else {
      self.error("syntax error", &["vec3", "NUM_LIT", "(", "false", "-", "ID", "[", "true", "not", "void", "vec2", "vec4"]);
    }
  }

  fn comp_expr_tail(&mut self) {
    if ["<", ">", "==", "<=", "&", ">=", "|", "!="].contains(&self.current()) {
      self.comp_binops();
      self.comp_expr();
    } else if [";", "or", "]", ",", "and", ")"].contains(&self.current()) {
      // do nothing
    } else {
      self.error("syntax error", &[")", ";", "|", ">=", "==", "]", "!=", "or", ",", "<=", "<", ">", "&", "and"]);
    }
  }

  fn add_expr(&mut self) {
    if ["-", "vec3", "false", "ID", "[", "void", "(", "vec4", "vec2", "true", "NUM_LIT", "not"].contains(&self.current()) {
      self.mul_expr();
      self.add_expr_tail();
    } else {
      self.error("syntax error", &["(", "not", "void", "[", "vec4", "vec3", "-", "vec2", "false", "true", "ID", "NUM_LIT"]);
    }
  }

  fn add_expr_tail(&mut self) {
    if ["-", "+"].contains(&self.current()) {
      self.add_binops();
      self.add_expr();
    } else if [">", "<=", "and", "&", "|", "<", "==", ";", ",", "]", "or", ")", ">=", "!="].contains(&self.current()) {
      // do nothing
    } else {
      self.error("syntax error", &["&", ")", "-", "<=", "<", ";", ",", ">=", "and", "or", ">", "|", "+", "==", "!=", "]"]);
    }
  }

  fn mul_expr(&mut self) {
    if ["not", "false", "void", "vec4", "NUM_LIT", "ID", "-", "vec2", "vec3", "[", "true", "("].contains(&self.current()) {
      self.unary_expr();
      self.mul_expr_tail();
    } else {
      self.error("syntax error", &["vec3", "vec4", "NUM_LIT", "false", "vec2", "[", "true", "not", "(", "-", "ID", "void"]);
    }
  }

  fn mul_expr_tail(&mut self) {
    if ["/", "*", "%"].contains(&self.current()) {
      self.mul_binops();
      self.mul_expr();
    } else if ["!=", "==", ",", ")", "or", "|", "and", "]", "&", "<=", "<", "-", "+", ";", ">", ">="].contains(&self.current()) {
      // do nothing
    } else {
      self.error("syntax error", &["<", ">", ",", "!=", "]", "<=", ";", "or", "==", "-", "%", "&", "/", ")", "+", "*", "and", ">=", "|"]);
    }
  }

  fn unary_expr(&mut self) {
    if ["vec2", "false", "void", "true", "NUM_LIT", "(", "vec4", "ID", "[", "vec3"].contains(&self.current()) {
      self.base_expr();
    } else if ["-", "not"].contains(&self.current()) {
      self.unary_expr_head();
    } else {
      self.error("syntax error", &["NUM_LIT", "-", "void", "[", "vec3", "not", "vec4", "(", "ID", "true", "vec2", "false"]);
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
    if ["vec3", "void", "vec4", "vec2"].contains(&self.current()) {
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
      self.error("syntax error", &["false", "void", "vec3", "[", "vec4", "vec2", "NUM_LIT", "(", "ID", "true"]);
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
    } else if ["==", "and", ">", "]", "<", ";", ",", "or", "/", "+", "%", "*", "&", "<=", ">=", "-", ")", "|", "!="].contains(&self.current()) {
      // do nothing
    } else {
      self.error("syntax error", &["[", "<=", "and", ")", "*", ",", "%", ";", ">=", "|", ">", "or", "]", "&", "<", "==", "/", "+", "-", "!="]);
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
    if ["void", "vec2", "vec3", "vec4"].contains(&self.current()) {
      self.base_types();
      self.func_call_args();
    } else {
      self.error("syntax error", &["vec2", "vec3", "void", "vec4"]);
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
    if ["[", "true", "not", "false", "void", "vec3", "(", "ID", "NUM_LIT", "-", "vec2", "vec4"].contains(&self.current()) {
      self.expr();
      self.expr_list_tail();
    } else {
      self.error("syntax error", &["vec3", "(", "not", "ID", "void", "vec4", "[", "vec2", "-", "false", "NUM_LIT", "true"]);
    }
  }

  fn expr_list_tail(&mut self) {
    if [","].contains(&self.current()) {
      self.match_kind(",");
      self.expr_list();
    } else if ["]", ")"].contains(&self.current()) {
      // do nothing
    } else {
      self.error("syntax error", &[",", ")", "]"]);
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
      self.error("syntax error", &["-", "+"]);
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
      self.error("syntax error", &["!=", "<=", ">=", "&", "<", ">", "==", "|"]);
    }
  }

  fn unary_ops(&mut self) {
    if ["-"].contains(&self.current()) {
      self.match_kind("-");
    } else if ["not"].contains(&self.current()) {
      self.match_kind("not");
    } else {
      self.error("syntax error", &["not", "-"]);
    }
  }

  fn types(&mut self) {
    if ["vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      self.base_types();
      self.opt_indexing();
    } else {
      self.error("syntax error", &["vec4", "void", "vec2", "vec3"]);
    }
  }

  fn opt_indexing(&mut self) {
    if ["["].contains(&self.current()) {
      self.match_kind("[");
      self.match_kind("NUM_LIT");
      self.match_kind("]");
    } else if [";", "=", "{", ",", ")"].contains(&self.current()) {
      // do nothing
    } else {
      self.error("syntax error", &["{", "[", ";", ")", "=", ","]);
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
      self.error("syntax error", &["vec4", "void", "vec2", "vec3"]);
    }
  }
}