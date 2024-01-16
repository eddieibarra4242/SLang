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
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["const", "entry", "fn", "fragment", "in", "let", "loc", "out", "vertex"]));
    }
    Ok(())
  }

  fn global_stmt_list(&mut self) -> Result<(), ParserError> {
    if ["const", "entry", "fn", "fragment", "in", "let", "loc", "out", "vertex"].contains(&self.current()) {
      self.global_stmt()?;
      self.global_stmt_list()?;
    } else if ["EOF"].contains(&self.current()) {
      // do nothing
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["EOF", "const", "entry", "fn", "fragment", "in", "let", "loc", "out", "vertex"]));
    }
    Ok(())
  }

  fn global_stmt(&mut self) -> Result<(), ParserError> {
    if ["entry", "fn", "fragment", "vertex"].contains(&self.current()) {
      self.function()?;
    } else if ["const", "in", "let", "loc", "out"].contains(&self.current()) {
      self.global_var_decl()?;
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["const", "entry", "fn", "fragment", "in", "let", "loc", "out", "vertex"]));
    }
    Ok(())
  }

  fn function(&mut self) -> Result<(), ParserError> {
    if ["entry", "fn", "fragment", "vertex"].contains(&self.current()) {
      self.function_def()?;
      self.compound_stmt()?;
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["entry", "fn", "fragment", "vertex"]));
    }
    Ok(())
  }

  fn stage_attr(&mut self) -> Result<(), ParserError> {
    if ["vertex"].contains(&self.current()) {
      self.match_kind("vertex")?;
    } else if ["fragment"].contains(&self.current()) {
      self.match_kind("fragment")?;
    } else if ["entry", "fn"].contains(&self.current()) {
      // do nothing
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["entry", "fn", "fragment", "vertex"]));
    }
    Ok(())
  }

  fn entry_attr(&mut self) -> Result<(), ParserError> {
    if ["entry"].contains(&self.current()) {
      self.match_kind("entry")?;
    } else if ["fn"].contains(&self.current()) {
      // do nothing
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["entry", "fn"]));
    }
    Ok(())
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
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["entry", "fn", "fragment", "vertex"]));
    }
    Ok(())
  }

  fn parameters(&mut self) -> Result<(), ParserError> {
    if ["("].contains(&self.current()) {
      self.match_kind("(")?;
      self.param_list()?;
      self.match_kind(")")?;
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["("]));
    }
    Ok(())
  }

  fn param_list(&mut self) -> Result<(), ParserError> {
    if ["const", "let"].contains(&self.current()) {
      self.param_list_non_empty()?;
    } else if [")"].contains(&self.current()) {
      // do nothing
    } else {
      return Err(UnexpectedToken(self.current_token(), vec![")", "const", "let"]));
    }
    Ok(())
  }

  fn param_list_non_empty(&mut self) -> Result<(), ParserError> {
    if ["const", "let"].contains(&self.current()) {
      self.var_decl()?;
      self.param_list_tail()?;
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["const", "let"]));
    }
    Ok(())
  }

  fn param_list_tail(&mut self) -> Result<(), ParserError> {
    if [","].contains(&self.current()) {
      self.match_kind(",")?;
      self.param_list_non_empty()?;
    } else if [")"].contains(&self.current()) {
      // do nothing
    } else {
      return Err(UnexpectedToken(self.current_token(), vec![")", ","]));
    }
    Ok(())
  }

  fn global_var_decl(&mut self) -> Result<(), ParserError> {
    if ["const", "in", "let", "loc", "out"].contains(&self.current()) {
      self.opt_attr_list()?;
      self.var_decl()?;
      self.match_kind(";")?;
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["const", "in", "let", "loc", "out"]));
    }
    Ok(())
  }

  fn opt_attr_list(&mut self) -> Result<(), ParserError> {
    if ["in", "loc", "out"].contains(&self.current()) {
      self.attrs()?;
      self.opt_attr_list()?;
    } else if ["const", "let"].contains(&self.current()) {
      // do nothing
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["const", "in", "let", "loc", "out"]));
    }
    Ok(())
  }

  fn attrs(&mut self) -> Result<(), ParserError> {
    if ["out"].contains(&self.current()) {
      self.match_kind("out")?;
    } else if ["in"].contains(&self.current()) {
      self.match_kind("in")?;
    } else if ["loc"].contains(&self.current()) {
      self.match_kind("loc")?;
      self.match_kind("(")?;
      self.match_kind("NUM_LIT")?;
      self.match_kind(")")?;
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["in", "loc", "out"]));
    }
    Ok(())
  }

  fn compound_stmt(&mut self) -> Result<(), ParserError> {
    if ["{"].contains(&self.current()) {
      self.match_kind("{")?;
      self.stmt_list()?;
      self.match_kind("}")?;
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["{"]));
    }
    Ok(())
  }

  fn stmt_list(&mut self) -> Result<(), ParserError> {
    if ["ID", "const", "let", "return"].contains(&self.current()) {
      self.stmt()?;
      self.stmt_list()?;
    } else if ["}"].contains(&self.current()) {
      // do nothing
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["}", "ID", "const", "let", "return"]));
    }
    Ok(())
  }

  fn stmt(&mut self) -> Result<(), ParserError> {
    if ["return"].contains(&self.current()) {
      self.match_kind("return")?;
      self.expr()?;
      self.match_kind(";")?;
    } else if ["const", "let"].contains(&self.current()) {
      self.var_decl()?;
      self.match_kind(";")?;
    } else if ["ID"].contains(&self.current()) {
      self.match_kind("ID")?;
      self.match_kind("=")?;
      self.expr()?;
      self.match_kind(";")?;
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["ID", "const", "let", "return"]));
    }
    Ok(())
  }

  fn var_decl(&mut self) -> Result<(), ParserError> {
    if ["const", "let"].contains(&self.current()) {
      self.var_decl_keyword()?;
      self.match_kind("ID")?;
      self.var_decl_tail()?;
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["const", "let"]));
    }
    Ok(())
  }

  fn var_decl_tail(&mut self) -> Result<(), ParserError> {
    if [":"].contains(&self.current()) {
      self.match_kind(":")?;
      self.types()?;
      self.opt_assignment()?;
    } else if ["="].contains(&self.current()) {
      self.match_kind("=")?;
      self.expr()?;
    } else {
      return Err(UnexpectedToken(self.current_token(), vec![":", "="]));
    }
    Ok(())
  }

  fn var_decl_keyword(&mut self) -> Result<(), ParserError> {
    if ["let"].contains(&self.current()) {
      self.match_kind("let")?;
    } else if ["const"].contains(&self.current()) {
      self.match_kind("const")?;
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["const", "let"]));
    }
    Ok(())
  }

  fn opt_assignment(&mut self) -> Result<(), ParserError> {
    if ["="].contains(&self.current()) {
      self.match_kind("=")?;
      self.expr()?;
    } else if [")", ",", ";"].contains(&self.current()) {
      // do nothing
    } else {
      return Err(UnexpectedToken(self.current_token(), vec![")", ",", ";", "="]));
    }
    Ok(())
  }

  fn expr(&mut self) -> Result<(), ParserError> {
    if ["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      self.prod_expr()?;
      self.expr_tail()?;
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"]));
    }
    Ok(())
  }

  fn expr_tail(&mut self) -> Result<(), ParserError> {
    if ["or"].contains(&self.current()) {
      self.match_kind("or")?;
      self.expr()?;
    } else if [")", ",", ";", "]"].contains(&self.current()) {
      // do nothing
    } else {
      return Err(UnexpectedToken(self.current_token(), vec![")", ",", ";", "]", "or"]));
    }
    Ok(())
  }

  fn prod_expr(&mut self) -> Result<(), ParserError> {
    if ["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      self.comp_expr()?;
      self.prod_expr_tail()?;
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"]));
    }
    Ok(())
  }

  fn prod_expr_tail(&mut self) -> Result<(), ParserError> {
    if ["and"].contains(&self.current()) {
      self.match_kind("and")?;
      self.prod_expr()?;
    } else if [")", ",", ";", "]", "or"].contains(&self.current()) {
      // do nothing
    } else {
      return Err(UnexpectedToken(self.current_token(), vec![")", ",", ";", "]", "and", "or"]));
    }
    Ok(())
  }

  fn comp_expr(&mut self) -> Result<(), ParserError> {
    if ["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      self.add_expr()?;
      self.comp_expr_tail()?;
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"]));
    }
    Ok(())
  }

  fn comp_expr_tail(&mut self) -> Result<(), ParserError> {
    if ["!=", "&", "<", "<=", "==", ">", ">=", "|"].contains(&self.current()) {
      self.comp_binops()?;
      self.comp_expr()?;
    } else if [")", ",", ";", "]", "and", "or"].contains(&self.current()) {
      // do nothing
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["!=", "&", ")", ",", ";", "<", "<=", "==", ">", ">=", "]", "|", "and", "or"]));
    }
    Ok(())
  }

  fn add_expr(&mut self) -> Result<(), ParserError> {
    if ["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      self.mul_expr()?;
      self.add_expr_tail()?;
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"]));
    }
    Ok(())
  }

  fn add_expr_tail(&mut self) -> Result<(), ParserError> {
    if ["+", "-"].contains(&self.current()) {
      self.add_binops()?;
      self.add_expr()?;
    } else if ["!=", "&", ")", ",", ";", "<", "<=", "==", ">", ">=", "]", "|", "and", "or"].contains(&self.current()) {
      // do nothing
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["!=", "&", ")", "+", ",", "-", ";", "<", "<=", "==", ">", ">=", "]", "|", "and", "or"]));
    }
    Ok(())
  }

  fn mul_expr(&mut self) -> Result<(), ParserError> {
    if ["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      self.unary_expr()?;
      self.mul_expr_tail()?;
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"]));
    }
    Ok(())
  }

  fn mul_expr_tail(&mut self) -> Result<(), ParserError> {
    if ["%", "*", "/"].contains(&self.current()) {
      self.mul_binops()?;
      self.mul_expr()?;
    } else if ["!=", "&", ")", "+", ",", "-", ";", "<", "<=", "==", ">", ">=", "]", "|", "and", "or"].contains(&self.current()) {
      // do nothing
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["!=", "%", "&", ")", "*", "+", ",", "-", "/", ";", "<", "<=", "==", ">", ">=", "]", "|", "and", "or"]));
    }
    Ok(())
  }

  fn unary_expr(&mut self) -> Result<(), ParserError> {
    if ["(", "[", "ID", "NUM_LIT", "false", "true", "vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      self.base_expr()?;
    } else if ["-", "not"].contains(&self.current()) {
      self.unary_expr_head()?;
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"]));
    }
    Ok(())
  }

  fn unary_expr_head(&mut self) -> Result<(), ParserError> {
    if ["-", "not"].contains(&self.current()) {
      self.unary_ops()?;
      self.unary_expr()?;
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["-", "not"]));
    }
    Ok(())
  }

  fn base_expr(&mut self) -> Result<(), ParserError> {
    if ["vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      self.type_constructor()?;
    } else if ["ID"].contains(&self.current()) {
      self.id_expr()?;
    } else if ["["].contains(&self.current()) {
      self.array_literal()?;
    } else if ["NUM_LIT"].contains(&self.current()) {
      self.match_kind("NUM_LIT")?;
    } else if ["true"].contains(&self.current()) {
      self.match_kind("true")?;
    } else if ["false"].contains(&self.current()) {
      self.match_kind("false")?;
    } else if ["("].contains(&self.current()) {
      self.grouped_expr()?;
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["(", "[", "ID", "NUM_LIT", "false", "true", "vec2", "vec3", "vec4", "void"]));
    }
    Ok(())
  }

  fn id_expr(&mut self) -> Result<(), ParserError> {
    if ["ID"].contains(&self.current()) {
      self.match_kind("ID")?;
      self.id_expr_tail()?;
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["ID"]));
    }
    Ok(())
  }

  fn id_expr_tail(&mut self) -> Result<(), ParserError> {
    if ["["].contains(&self.current()) {
      self.match_kind("[")?;
      self.expr()?;
      self.match_kind("]")?;
    } else if ["("].contains(&self.current()) {
      self.func_call_args()?;
    } else if ["!=", "%", "&", ")", "*", "+", ",", "-", "/", ";", "<", "<=", "==", ">", ">=", "]", "|", "and", "or"].contains(&self.current()) {
      // do nothing
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["!=", "%", "&", "(", ")", "*", "+", ",", "-", "/", ";", "<", "<=", "==", ">", ">=", "[", "]", "|", "and", "or"]));
    }
    Ok(())
  }

  fn array_literal(&mut self) -> Result<(), ParserError> {
    if ["["].contains(&self.current()) {
      self.match_kind("[")?;
      self.expr_list()?;
      self.match_kind("]")?;
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["["]));
    }
    Ok(())
  }

  fn type_constructor(&mut self) -> Result<(), ParserError> {
    if ["vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      self.base_types()?;
      self.func_call_args()?;
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["vec2", "vec3", "vec4", "void"]));
    }
    Ok(())
  }

  fn func_call_args(&mut self) -> Result<(), ParserError> {
    if ["("].contains(&self.current()) {
      self.match_kind("(")?;
      self.expr_list()?;
      self.match_kind(")")?;
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["("]));
    }
    Ok(())
  }

  fn expr_list(&mut self) -> Result<(), ParserError> {
    if ["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      self.expr()?;
      self.expr_list_tail()?;
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"]));
    }
    Ok(())
  }

  fn expr_list_tail(&mut self) -> Result<(), ParserError> {
    if [","].contains(&self.current()) {
      self.match_kind(",")?;
      self.expr_list()?;
    } else if [")", "]"].contains(&self.current()) {
      // do nothing
    } else {
      return Err(UnexpectedToken(self.current_token(), vec![")", ",", "]"]));
    }
    Ok(())
  }

  fn grouped_expr(&mut self) -> Result<(), ParserError> {
    if ["("].contains(&self.current()) {
      self.match_kind("(")?;
      self.expr()?;
      self.match_kind(")")?;
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["("]));
    }
    Ok(())
  }

  fn mul_binops(&mut self) -> Result<(), ParserError> {
    if ["*"].contains(&self.current()) {
      self.match_kind("*")?;
    } else if ["/"].contains(&self.current()) {
      self.match_kind("/")?;
    } else if ["%"].contains(&self.current()) {
      self.match_kind("%")?;
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["%", "*", "/"]));
    }
    Ok(())
  }

  fn add_binops(&mut self) -> Result<(), ParserError> {
    if ["+"].contains(&self.current()) {
      self.match_kind("+")?;
    } else if ["-"].contains(&self.current()) {
      self.match_kind("-")?;
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["+", "-"]));
    }
    Ok(())
  }

  fn comp_binops(&mut self) -> Result<(), ParserError> {
    if ["!="].contains(&self.current()) {
      self.match_kind("!=")?;
    } else if ["&"].contains(&self.current()) {
      self.match_kind("&")?;
    } else if ["<="].contains(&self.current()) {
      self.match_kind("<=")?;
    } else if ["<"].contains(&self.current()) {
      self.match_kind("<")?;
    } else if [">="].contains(&self.current()) {
      self.match_kind(">=")?;
    } else if [">"].contains(&self.current()) {
      self.match_kind(">")?;
    } else if ["=="].contains(&self.current()) {
      self.match_kind("==")?;
    } else if ["|"].contains(&self.current()) {
      self.match_kind("|")?;
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["!=", "&", "<", "<=", "==", ">", ">=", "|"]));
    }
    Ok(())
  }

  fn unary_ops(&mut self) -> Result<(), ParserError> {
    if ["-"].contains(&self.current()) {
      self.match_kind("-")?;
    } else if ["not"].contains(&self.current()) {
      self.match_kind("not")?;
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["-", "not"]));
    }
    Ok(())
  }

  fn types(&mut self) -> Result<(), ParserError> {
    if ["vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      self.base_types()?;
      self.opt_indexing()?;
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["vec2", "vec3", "vec4", "void"]));
    }
    Ok(())
  }

  fn opt_indexing(&mut self) -> Result<(), ParserError> {
    if ["["].contains(&self.current()) {
      self.match_kind("[")?;
      self.match_kind("NUM_LIT")?;
      self.match_kind("]")?;
    } else if [")", ",", ";", "=", "{"].contains(&self.current()) {
      // do nothing
    } else {
      return Err(UnexpectedToken(self.current_token(), vec![")", ",", ";", "=", "[", "{"]));
    }
    Ok(())
  }

  fn base_types(&mut self) -> Result<(), ParserError> {
    if ["void"].contains(&self.current()) {
      self.match_kind("void")?;
    } else if ["vec2"].contains(&self.current()) {
      self.match_kind("vec2")?;
    } else if ["vec3"].contains(&self.current()) {
      self.match_kind("vec3")?;
    } else if ["vec4"].contains(&self.current()) {
      self.match_kind("vec4")?;
    } else {
      return Err(UnexpectedToken(self.current_token(), vec!["vec2", "vec3", "vec4", "void"]));
    }
    Ok(())
  }
}