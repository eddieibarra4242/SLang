use crate::parser::ParserError::UnexpectedToken;
use crate::scanner::Token;

pub(crate) enum ParserError {
  UnexpectedToken(Token, Vec<&'static str>)
}

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

  fn match_kind(&mut self, kind: &'static str) -> Result<Token, ParserError> {
    return if self.current() == kind {
      let prev = self.scanner[self.current_ndx].clone();
      self.current_ndx += 1;
      Ok(prev)
    } else {
      Err(UnexpectedToken(self.current_token(), vec![kind]))
    };
  }

  fn current(&self) -> &str {
    self.scanner[self.current_ndx].kind.as_str()
  }

  fn current_token(&self) -> Token {
    self.scanner[self.current_ndx].clone()
  }

  pub(crate) fn parse(&mut self) -> Result<(), ParserError> {
    self.program()?;
    self.match_kind("EOF")?;
    Ok(())
  }

  fn program(&mut self) -> Result<(), ParserError> {
    if ["const", "entry", "fn", "fragment", "in", "let", "loc", "out", "vertex"].contains(&self.current()) {
      self.global_stmt()?;
      self.global_stmt_list()?;
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["const", "entry", "fn", "fragment", "in", "let", "loc", "out", "vertex"]))
    }
  }

  fn global_stmt_list(&mut self) -> Result<(), ParserError> {
    if ["const", "entry", "fn", "fragment", "in", "let", "loc", "out", "vertex"].contains(&self.current()) {
      self.global_stmt()?;
      self.global_stmt_list()?;
      Ok(())
    } else if ["EOF"].contains(&self.current()) {
      // do nothing
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["EOF", "const", "entry", "fn", "fragment", "in", "let", "loc", "out", "vertex"]))
    }
  }

  fn global_stmt(&mut self) -> Result<(), ParserError> {
    if ["entry", "fn", "fragment", "vertex"].contains(&self.current()) {
      self.function()?;
      Ok(())
    } else if ["const", "in", "let", "loc", "out"].contains(&self.current()) {
      self.global_var_decl()?;
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["const", "entry", "fn", "fragment", "in", "let", "loc", "out", "vertex"]))
    }
  }

  fn function(&mut self) -> Result<(), ParserError> {
    if ["entry", "fn", "fragment", "vertex"].contains(&self.current()) {
      self.function_def()?;
      self.compound_stmt()?;
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["entry", "fn", "fragment", "vertex"]))
    }
  }

  fn stage_attr(&mut self) -> Result<(), ParserError> {
    if ["vertex"].contains(&self.current()) {
      self.match_kind("vertex")?;
      Ok(())
    } else if ["fragment"].contains(&self.current()) {
      self.match_kind("fragment")?;
      Ok(())
    } else if ["entry", "fn"].contains(&self.current()) {
      // do nothing
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["entry", "fn", "fragment", "vertex"]))
    }
  }

  fn entry_attr(&mut self) -> Result<(), ParserError> {
    if ["entry"].contains(&self.current()) {
      self.match_kind("entry")?;
      Ok(())
    } else if ["fn"].contains(&self.current()) {
      // do nothing
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["entry", "fn"]))
    }
  }

  fn function_def(&mut self) -> Result<(), ParserError> {
    if ["entry", "fn", "fragment", "vertex"].contains(&self.current()) {
      self.stage_attr()?;
      self.entry_attr()?;
      self.match_kind("fn")?;
      self.match_kind("ID")?;
      self.parameters()?;
      self.match_kind("->")?;
      self.types()?;
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["entry", "fn", "fragment", "vertex"]))
    }
  }

  fn parameters(&mut self) -> Result<(), ParserError> {
    if ["("].contains(&self.current()) {
      self.match_kind("(")?;
      self.param_list()?;
      self.match_kind(")")?;
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["("]))
    }
  }

  fn param_list(&mut self) -> Result<(), ParserError> {
    if ["const", "let"].contains(&self.current()) {
      self.param_list_non_empty()?;
      Ok(())
    } else if [")"].contains(&self.current()) {
      // do nothing
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec![")", "const", "let"]))
    }
  }

  fn param_list_non_empty(&mut self) -> Result<(), ParserError> {
    if ["const", "let"].contains(&self.current()) {
      self.var_decl()?;
      self.param_list_tail()?;
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["const", "let"]))
    }
  }

  fn param_list_tail(&mut self) -> Result<(), ParserError> {
    if [","].contains(&self.current()) {
      self.match_kind(",")?;
      self.param_list_non_empty()?;
      Ok(())
    } else if [")"].contains(&self.current()) {
      // do nothing
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec![")", ","]))
    }
  }

  fn global_var_decl(&mut self) -> Result<(), ParserError> {
    if ["const", "in", "let", "loc", "out"].contains(&self.current()) {
      self.opt_attr_list()?;
      self.var_decl()?;
      self.match_kind(";")?;
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["const", "in", "let", "loc", "out"]))
    }
  }

  fn opt_attr_list(&mut self) -> Result<(), ParserError> {
    if ["in", "loc", "out"].contains(&self.current()) {
      self.attrs()?;
      self.opt_attr_list()?;
      Ok(())
    } else if ["const", "let"].contains(&self.current()) {
      // do nothing
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["const", "in", "let", "loc", "out"]))
    }
  }

  fn attrs(&mut self) -> Result<(), ParserError> {
    if ["out"].contains(&self.current()) {
      self.match_kind("out")?;
      Ok(())
    } else if ["in"].contains(&self.current()) {
      self.match_kind("in")?;
      Ok(())
    } else if ["loc"].contains(&self.current()) {
      self.match_kind("loc")?;
      self.match_kind("(")?;
      self.match_kind("NUM_LIT")?;
      self.match_kind(")")?;
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["in", "loc", "out"]))
    }
  }

  fn compound_stmt(&mut self) -> Result<(), ParserError> {
    if ["{"].contains(&self.current()) {
      self.match_kind("{")?;
      self.stmt_list()?;
      self.match_kind("}")?;
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["{"]))
    }
  }

  fn stmt_list(&mut self) -> Result<(), ParserError> {
    if ["ID", "const", "let", "return"].contains(&self.current()) {
      self.stmt()?;
      self.stmt_list()?;
      Ok(())
    } else if ["}"].contains(&self.current()) {
      // do nothing
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["}", "ID", "const", "let", "return"]))
    }
  }

  fn stmt(&mut self) -> Result<(), ParserError> {
    if ["return"].contains(&self.current()) {
      self.match_kind("return")?;
      self.expr()?;
      self.match_kind(";")?;
      Ok(())
    } else if ["const", "let"].contains(&self.current()) {
      self.var_decl()?;
      self.match_kind(";")?;
      Ok(())
    } else if ["ID"].contains(&self.current()) {
      self.match_kind("ID")?;
      self.match_kind("=")?;
      self.expr()?;
      self.match_kind(";")?;
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["ID", "const", "let", "return"]))
    }
  }

  fn var_decl(&mut self) -> Result<(), ParserError> {
    if ["const", "let"].contains(&self.current()) {
      self.var_decl_keyword()?;
      self.match_kind("ID")?;
      self.var_decl_tail()?;
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["const", "let"]))
    }
  }

  fn var_decl_tail(&mut self) -> Result<(), ParserError> {
    if [":"].contains(&self.current()) {
      self.match_kind(":")?;
      self.types()?;
      self.opt_assignment()?;
      Ok(())
    } else if ["="].contains(&self.current()) {
      self.match_kind("=")?;
      self.expr()?;
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec![":", "="]))
    }
  }

  fn var_decl_keyword(&mut self) -> Result<(), ParserError> {
    if ["let"].contains(&self.current()) {
      self.match_kind("let")?;
      Ok(())
    } else if ["const"].contains(&self.current()) {
      self.match_kind("const")?;
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["const", "let"]))
    }
  }

  fn opt_assignment(&mut self) -> Result<(), ParserError> {
    if ["="].contains(&self.current()) {
      self.match_kind("=")?;
      self.expr()?;
      Ok(())
    } else if [")", ",", ";"].contains(&self.current()) {
      // do nothing
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec![")", ",", ";", "="]))
    }
  }

  fn expr(&mut self) -> Result<(), ParserError> {
    if ["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      self.prod_expr()?;
      self.expr_tail()?;
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"]))
    }
  }

  fn expr_tail(&mut self) -> Result<(), ParserError> {
    if ["or"].contains(&self.current()) {
      self.match_kind("or")?;
      self.expr()?;
      Ok(())
    } else if [")", ",", ";", "]"].contains(&self.current()) {
      // do nothing
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec![")", ",", ";", "]", "or"]))
    }
  }

  fn prod_expr(&mut self) -> Result<(), ParserError> {
    if ["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      self.comp_expr()?;
      self.prod_expr_tail()?;
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"]))
    }
  }

  fn prod_expr_tail(&mut self) -> Result<(), ParserError> {
    if ["and"].contains(&self.current()) {
      self.match_kind("and")?;
      self.prod_expr()?;
      Ok(())
    } else if [")", ",", ";", "]", "or"].contains(&self.current()) {
      // do nothing
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec![")", ",", ";", "]", "and", "or"]))
    }
  }

  fn comp_expr(&mut self) -> Result<(), ParserError> {
    if ["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      self.add_expr()?;
      self.comp_expr_tail()?;
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"]))
    }
  }

  fn comp_expr_tail(&mut self) -> Result<(), ParserError> {
    if ["!=", "&", "<", "<=", "==", ">", ">=", "|"].contains(&self.current()) {
      self.comp_binops()?;
      self.comp_expr()?;
      Ok(())
    } else if [")", ",", ";", "]", "and", "or"].contains(&self.current()) {
      // do nothing
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["!=", "&", ")", ",", ";", "<", "<=", "==", ">", ">=", "]", "|", "and", "or"]))
    }
  }

  fn add_expr(&mut self) -> Result<(), ParserError> {
    if ["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      self.mul_expr()?;
      self.add_expr_tail()?;
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"]))
    }
  }

  fn add_expr_tail(&mut self) -> Result<(), ParserError> {
    if ["+", "-"].contains(&self.current()) {
      self.add_binops()?;
      self.add_expr()?;
      Ok(())
    } else if ["!=", "&", ")", ",", ";", "<", "<=", "==", ">", ">=", "]", "|", "and", "or"].contains(&self.current()) {
      // do nothing
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["!=", "&", ")", "+", ",", "-", ";", "<", "<=", "==", ">", ">=", "]", "|", "and", "or"]))
    }
  }

  fn mul_expr(&mut self) -> Result<(), ParserError> {
    if ["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      self.unary_expr()?;
      self.mul_expr_tail()?;
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"]))
    }
  }

  fn mul_expr_tail(&mut self) -> Result<(), ParserError> {
    if ["%", "*", "/"].contains(&self.current()) {
      self.mul_binops()?;
      self.mul_expr()?;
      Ok(())
    } else if ["!=", "&", ")", "+", ",", "-", ";", "<", "<=", "==", ">", ">=", "]", "|", "and", "or"].contains(&self.current()) {
      // do nothing
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["!=", "%", "&", ")", "*", "+", ",", "-", "/", ";", "<", "<=", "==", ">", ">=", "]", "|", "and", "or"]))
    }
  }

  fn unary_expr(&mut self) -> Result<(), ParserError> {
    if ["(", "[", "ID", "NUM_LIT", "false", "true", "vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      self.base_expr()?;
      Ok(())
    } else if ["-", "not"].contains(&self.current()) {
      self.unary_expr_head()?;
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"]))
    }
  }

  fn unary_expr_head(&mut self) -> Result<(), ParserError> {
    if ["-", "not"].contains(&self.current()) {
      self.unary_ops()?;
      self.unary_expr()?;
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["-", "not"]))
    }
  }

  fn base_expr(&mut self) -> Result<(), ParserError> {
    if ["vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      self.type_constructor()?;
      Ok(())
    } else if ["ID"].contains(&self.current()) {
      self.id_expr()?;
      Ok(())
    } else if ["["].contains(&self.current()) {
      self.array_literal()?;
      Ok(())
    } else if ["NUM_LIT"].contains(&self.current()) {
      self.match_kind("NUM_LIT")?;
      Ok(())
    } else if ["true"].contains(&self.current()) {
      self.match_kind("true")?;
      Ok(())
    } else if ["false"].contains(&self.current()) {
      self.match_kind("false")?;
      Ok(())
    } else if ["("].contains(&self.current()) {
      self.grouped_expr()?;
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["(", "[", "ID", "NUM_LIT", "false", "true", "vec2", "vec3", "vec4", "void"]))
    }
  }

  fn id_expr(&mut self) -> Result<(), ParserError> {
    if ["ID"].contains(&self.current()) {
      self.match_kind("ID")?;
      self.id_expr_tail()?;
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["ID"]))
    }
  }

  fn id_expr_tail(&mut self) -> Result<(), ParserError> {
    if ["["].contains(&self.current()) {
      self.match_kind("[")?;
      self.expr()?;
      self.match_kind("]")?;
      Ok(())
    } else if ["("].contains(&self.current()) {
      self.func_call_args()?;
      Ok(())
    } else if ["!=", "%", "&", ")", "*", "+", ",", "-", "/", ";", "<", "<=", "==", ">", ">=", "]", "|", "and", "or"].contains(&self.current()) {
      // do nothing
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["!=", "%", "&", "(", ")", "*", "+", ",", "-", "/", ";", "<", "<=", "==", ">", ">=", "[", "]", "|", "and", "or"]))
    }
  }

  fn array_literal(&mut self) -> Result<(), ParserError> {
    if ["["].contains(&self.current()) {
      self.match_kind("[")?;
      self.expr_list()?;
      self.match_kind("]")?;
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["["]))
    }
  }

  fn type_constructor(&mut self) -> Result<(), ParserError> {
    if ["vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      self.base_types()?;
      self.func_call_args()?;
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["vec2", "vec3", "vec4", "void"]))
    }
  }

  fn func_call_args(&mut self) -> Result<(), ParserError> {
    if ["("].contains(&self.current()) {
      self.match_kind("(")?;
      self.expr_list()?;
      self.match_kind(")")?;
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["("]))
    }
  }

  fn expr_list(&mut self) -> Result<(), ParserError> {
    if ["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      self.expr()?;
      self.expr_list_tail()?;
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"]))
    }
  }

  fn expr_list_tail(&mut self) -> Result<(), ParserError> {
    if [","].contains(&self.current()) {
      self.match_kind(",")?;
      self.expr_list()?;
      Ok(())
    } else if [")", "]"].contains(&self.current()) {
      // do nothing
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec![")", ",", "]"]))
    }
  }

  fn grouped_expr(&mut self) -> Result<(), ParserError> {
    if ["("].contains(&self.current()) {
      self.match_kind("(")?;
      self.expr()?;
      self.match_kind(")")?;
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["("]))
    }
  }

  fn mul_binops(&mut self) -> Result<(), ParserError> {
    if ["*"].contains(&self.current()) {
      self.match_kind("*")?;
      Ok(())
    } else if ["/"].contains(&self.current()) {
      self.match_kind("/")?;
      Ok(())
    } else if ["%"].contains(&self.current()) {
      self.match_kind("%")?;
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["%", "*", "/"]))
    }
  }

  fn add_binops(&mut self) -> Result<(), ParserError> {
    if ["+"].contains(&self.current()) {
      self.match_kind("+")?;
      Ok(())
    } else if ["-"].contains(&self.current()) {
      self.match_kind("-")?;
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["+", "-"]))
    }
  }

  fn comp_binops(&mut self) -> Result<(), ParserError> {
    if ["!="].contains(&self.current()) {
      self.match_kind("!=")?;
      Ok(())
    } else if ["&"].contains(&self.current()) {
      self.match_kind("&")?;
      Ok(())
    } else if ["<="].contains(&self.current()) {
      self.match_kind("<=")?;
      Ok(())
    } else if ["<"].contains(&self.current()) {
      self.match_kind("<")?;
      Ok(())
    } else if [">="].contains(&self.current()) {
      self.match_kind(">=")?;
      Ok(())
    } else if [">"].contains(&self.current()) {
      self.match_kind(">")?;
      Ok(())
    } else if ["=="].contains(&self.current()) {
      self.match_kind("==")?;
      Ok(())
    } else if ["|"].contains(&self.current()) {
      self.match_kind("|")?;
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["!=", "&", "<", "<=", "==", ">", ">=", "|"]))
    }
  }

  fn unary_ops(&mut self) -> Result<(), ParserError> {
    if ["-"].contains(&self.current()) {
      self.match_kind("-")?;
      Ok(())
    } else if ["not"].contains(&self.current()) {
      self.match_kind("not")?;
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["-", "not"]))
    }
  }

  fn types(&mut self) -> Result<(), ParserError> {
    if ["vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      self.base_types()?;
      self.opt_indexing()?;
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["vec2", "vec3", "vec4", "void"]))
    }
  }

  fn opt_indexing(&mut self) -> Result<(), ParserError> {
    if ["["].contains(&self.current()) {
      self.match_kind("[")?;
      self.match_kind("NUM_LIT")?;
      self.match_kind("]")?;
      Ok(())
    } else if [")", ",", ";", "=", "{"].contains(&self.current()) {
      // do nothing
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec![")", ",", ";", "=", "[", "{"]))
    }
  }

  fn base_types(&mut self) -> Result<(), ParserError> {
    if ["void"].contains(&self.current()) {
      self.match_kind("void")?;
      Ok(())
    } else if ["vec2"].contains(&self.current()) {
      self.match_kind("vec2")?;
      Ok(())
    } else if ["vec3"].contains(&self.current()) {
      self.match_kind("vec3")?;
      Ok(())
    } else if ["vec4"].contains(&self.current()) {
      self.match_kind("vec4")?;
      Ok(())
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["vec2", "vec3", "vec4", "void"]))
    }
  }
}