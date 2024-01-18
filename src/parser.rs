use crate::ast::{Expr, Id};
use crate::ast::Expr::{*};
use crate::ast::Literal::{Array, Boolean, Number};
use crate::parser::IdExprTail::{ArrayIndex, FuncCall};
use crate::parser::ParserError::UnexpectedToken;
use crate::scanner::Token;

enum IdExprTail {
  ArrayIndex(Expr),
  FuncCall(Vec<Expr>)
}

pub(crate) enum ParserError {
  UnexpectedToken(Token, Vec<&'static str>)
}

pub(crate) struct Parser {
  scanner: Vec<Token>,
  current_ndx: usize,
}

fn bin_expr_unwrap_or(lhs: Expr, bin_expr: Option<(Token, Expr)>) -> Expr {
  if bin_expr.is_none() {
    lhs
  } else {
    let (op, rhs) = bin_expr.unwrap();
    BinaryExpr(op, Box::new(lhs), Box::new(rhs))
  }
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

  fn expr(&mut self) -> Result<Expr, ParserError> {
    if ["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      let lhs = self.prod_expr()?;
      let bin_expr = self.expr_tail()?;
      let res = bin_expr_unwrap_or(lhs, bin_expr);
      println!("{:?}\n", res);
      Ok(res)
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"]))
    }
  }

  fn expr_tail(&mut self) -> Result<Option<(Token, Expr)>, ParserError> {
    if ["or"].contains(&self.current()) {
      let op = self.match_kind("or")?;
      let rhs = self.expr()?;
      Ok(Some((op, rhs)))
    } else if [")", ",", ";", "]"].contains(&self.current()) {
      // do nothing
      Ok(None)
    } else {
      Err(UnexpectedToken(self.current_token(), vec![")", ",", ";", "]", "or"]))
    }
  }

  fn prod_expr(&mut self) -> Result<Expr, ParserError> {
    if ["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      let lhs = self.comp_expr()?;
      let bin_expr = self.prod_expr_tail()?;
      Ok(bin_expr_unwrap_or(lhs, bin_expr))
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"]))
    }
  }

  fn prod_expr_tail(&mut self) -> Result<Option<(Token, Expr)>, ParserError> {
    if ["and"].contains(&self.current()) {
      let op = self.match_kind("and")?;
      let rhs = self.prod_expr()?;
      Ok(Some((op, rhs)))
    } else if [")", ",", ";", "]", "or"].contains(&self.current()) {
      // do nothing
      Ok(None)
    } else {
      Err(UnexpectedToken(self.current_token(), vec![")", ",", ";", "]", "and", "or"]))
    }
  }

  fn comp_expr(&mut self) -> Result<Expr, ParserError> {
    if ["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      let lhs = self.add_expr()?;
      let bin_expr = self.comp_expr_tail()?;
      Ok(bin_expr_unwrap_or(lhs, bin_expr))
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"]))
    }
  }

  fn comp_expr_tail(&mut self) -> Result<Option<(Token, Expr)>, ParserError> {
    if ["!=", "&", "<", "<=", "==", ">", ">=", "|"].contains(&self.current()) {
      let op = self.comp_binops()?;
      let rhs = self.comp_expr()?;
      Ok(Some((op, rhs)))
    } else if [")", ",", ";", "]", "and", "or"].contains(&self.current()) {
      // do nothing
      Ok(None)
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["!=", "&", ")", ",", ";", "<", "<=", "==", ">", ">=", "]", "|", "and", "or"]))
    }
  }

  fn add_expr(&mut self) -> Result<Expr, ParserError> {
    if ["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      let lhs = self.mul_expr()?;
      let bin_expr = self.add_expr_tail()?;
      Ok(bin_expr_unwrap_or(lhs, bin_expr))
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"]))
    }
  }

  fn add_expr_tail(&mut self) -> Result<Option<(Token, Expr)>, ParserError> {
    if ["+", "-"].contains(&self.current()) {
      let token = self.add_binops()?;
      let rhs = self.add_expr()?;
      Ok(Some((token, rhs)))
    } else if ["!=", "&", ")", ",", ";", "<", "<=", "==", ">", ">=", "]", "|", "and", "or"].contains(&self.current()) {
      // do nothing
      Ok(None)
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["!=", "&", ")", "+", ",", "-", ";", "<", "<=", "==", ">", ">=", "]", "|", "and", "or"]))
    }
  }

  fn mul_expr(&mut self) -> Result<Expr, ParserError> {
    if ["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      let lhs = self.unary_expr()?;
      let bin_expr = self.mul_expr_tail()?;
      Ok(bin_expr_unwrap_or(lhs, bin_expr))
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"]))
    }
  }

  fn mul_expr_tail(&mut self) -> Result<Option<(Token, Expr)>, ParserError> {
    if ["%", "*", "/"].contains(&self.current()) {
      let op = self.mul_binops()?;
      let rhs = self.mul_expr()?;
      Ok(Some((op, rhs)))
    } else if ["!=", "&", ")", "+", ",", "-", ";", "<", "<=", "==", ">", ">=", "]", "|", "and", "or"].contains(&self.current()) {
      // do nothing
      Ok(None)
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["!=", "%", "&", ")", "*", "+", ",", "-", "/", ";", "<", "<=", "==", ">", ">=", "]", "|", "and", "or"]))
    }
  }

  fn unary_expr(&mut self) -> Result<Expr, ParserError> {
    if ["(", "[", "ID", "NUM_LIT", "false", "true", "vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      let expr = self.base_expr()?;
      Ok(expr)
    } else if ["-", "not"].contains(&self.current()) {
      let expr = self.unary_expr_head()?;
      Ok(expr)
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"]))
    }
  }

  fn unary_expr_head(&mut self) -> Result<Expr, ParserError> {
    if ["-", "not"].contains(&self.current()) {
      let op = self.unary_ops()?;
      let expr = self.unary_expr()?;
      Ok(UnaryExpr(op, Box::new(expr)))
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["-", "not"]))
    }
  }

  fn base_expr(&mut self) -> Result<Expr, ParserError> {
    if ["vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      let expr = self.type_constructor()?;
      Ok(expr)
    } else if ["ID"].contains(&self.current()) {
      let expr = self.id_expr()?;
      Ok(expr)
    } else if ["["].contains(&self.current()) {
      let expr = self.array_literal()?;
      Ok(expr)
    } else if ["NUM_LIT"].contains(&self.current()) {
      let token = self.match_kind("NUM_LIT")?;
      Ok(LiteralExpr(Number(token)))
    } else if ["true"].contains(&self.current()) {
      let token = self.match_kind("true")?;
      Ok(LiteralExpr(Boolean(token, true)))
    } else if ["false"].contains(&self.current()) {
      let token = self.match_kind("false")?;
      Ok(LiteralExpr(Boolean(token, false)))
    } else if ["("].contains(&self.current()) {
      let expr = self.grouped_expr()?;
      Ok(expr)
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["(", "[", "ID", "NUM_LIT", "false", "true", "vec2", "vec3", "vec4", "void"]))
    }
  }

  fn id_expr(&mut self) -> Result<Expr, ParserError> {
    if ["ID"].contains(&self.current()) {
      let token = self.match_kind("ID")?;
      let tail_wrapped = self.id_expr_tail()?;

      let id = Id::new(token);

      if tail_wrapped.is_none() {
        Ok(IdExpr(id))
      } else {
        Ok(match tail_wrapped.unwrap() {
          ArrayIndex(index) => {
            ArrayLookupExpr(Box::new(IdExpr(id)), Box::new(index))
          }
          FuncCall(arg_list) => {
            FunctionExpr(id, arg_list)
          }
        })
      }
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["ID"]))
    }
  }

  fn id_expr_tail(&mut self) -> Result<Option<IdExprTail>, ParserError> {
    if ["["].contains(&self.current()) {
      self.match_kind("[")?;
      let index = self.expr()?;
      self.match_kind("]")?;
      Ok(Some(ArrayIndex(index)))
    } else if ["("].contains(&self.current()) {
      let arg_list = self.func_call_args()?;
      Ok(Some(FuncCall(arg_list)))
    } else if ["!=", "%", "&", ")", "*", "+", ",", "-", "/", ";", "<", "<=", "==", ">", ">=", "]", "|", "and", "or"].contains(&self.current()) {
      // do nothing
      Ok(None)
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["!=", "%", "&", "(", ")", "*", "+", ",", "-", "/", ";", "<", "<=", "==", ">", ">=", "[", "]", "|", "and", "or"]))
    }
  }

  fn array_literal(&mut self) -> Result<Expr, ParserError> {
    if ["["].contains(&self.current()) {
      self.match_kind("[")?;
      let values = self.expr_list()?;
      self.match_kind("]")?;
      Ok(LiteralExpr(Array(values)))
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["["]))
    }
  }

  fn type_constructor(&mut self) -> Result<Expr, ParserError> {
    if ["vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      let func_name = self.base_types()?;
      let arg_list = self.func_call_args()?;
      Ok(FunctionExpr(Id::new(func_name), arg_list))
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["vec2", "vec3", "vec4", "void"]))
    }
  }

  fn func_call_args(&mut self) -> Result<Vec<Expr>, ParserError> {
    if ["("].contains(&self.current()) {
      self.match_kind("(")?;
      let arg_list = self.expr_list()?;
      self.match_kind(")")?;
      Ok(arg_list)
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["("]))
    }
  }

  fn expr_list(&mut self) -> Result<Vec<Expr>, ParserError> {
    if ["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"].contains(&self.current()) {
      let head = self.expr()?;
      let mut tail = self.expr_list_tail()?;
      tail.insert(0, head);
      Ok(tail)
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["(", "-", "[", "ID", "NUM_LIT", "false", "not", "true", "vec2", "vec3", "vec4", "void"]))
    }
  }

  fn expr_list_tail(&mut self) -> Result<Vec<Expr>, ParserError> {
    if [","].contains(&self.current()) {
      self.match_kind(",")?;
      let list = self.expr_list()?;
      Ok(list)
    } else if [")", "]"].contains(&self.current()) {
      // do nothing
      Ok(vec![])
    } else {
      Err(UnexpectedToken(self.current_token(), vec![")", ",", "]"]))
    }
  }

  fn grouped_expr(&mut self) -> Result<Expr, ParserError> {
    if ["("].contains(&self.current()) {
      self.match_kind("(")?;
      let expr = self.expr()?;
      self.match_kind(")")?;
      Ok(expr)
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["("]))
    }
  }

  fn mul_binops(&mut self) -> Result<Token, ParserError> {
    if ["*"].contains(&self.current()) {
      let token = self.match_kind("*")?;
      Ok(token)
    } else if ["/"].contains(&self.current()) {
      let token = self.match_kind("/")?;
      Ok(token)
    } else if ["%"].contains(&self.current()) {
      let token = self.match_kind("%")?;
      Ok(token)
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["%", "*", "/"]))
    }
  }

  fn add_binops(&mut self) -> Result<Token, ParserError> {
    if ["+"].contains(&self.current()) {
      let token = self.match_kind("+")?;
      Ok(token)
    } else if ["-"].contains(&self.current()) {
      let token = self.match_kind("-")?;
      Ok(token)
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["+", "-"]))
    }
  }

  fn comp_binops(&mut self) -> Result<Token, ParserError> {
    if ["!="].contains(&self.current()) {
      let token = self.match_kind("!=")?;
      Ok(token)
    } else if ["&"].contains(&self.current()) {
      let token = self.match_kind("&")?;
      Ok(token)
    } else if ["<="].contains(&self.current()) {
      let token = self.match_kind("<=")?;
      Ok(token)
    } else if ["<"].contains(&self.current()) {
      let token = self.match_kind("<")?;
      Ok(token)
    } else if [">="].contains(&self.current()) {
      let token = self.match_kind(">=")?;
      Ok(token)
    } else if [">"].contains(&self.current()) {
      let token = self.match_kind(">")?;
      Ok(token)
    } else if ["=="].contains(&self.current()) {
      let token = self.match_kind("==")?;
      Ok(token)
    } else if ["|"].contains(&self.current()) {
      let token = self.match_kind("|")?;
      Ok(token)
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["!=", "&", "<", "<=", "==", ">", ">=", "|"]))
    }
  }

  fn unary_ops(&mut self) -> Result<Token, ParserError> {
    if ["-"].contains(&self.current()) {
      let token = self.match_kind("-")?;
      Ok(token)
    } else if ["not"].contains(&self.current()) {
      let token =self.match_kind("not")?;
      Ok(token)
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

  fn base_types(&mut self) -> Result<Token, ParserError> {
    if ["void"].contains(&self.current()) {
      let token = self.match_kind("void")?;
      Ok(token)
    } else if ["vec2"].contains(&self.current()) {
      let token = self.match_kind("vec2")?;
      Ok(token)
    } else if ["vec3"].contains(&self.current()) {
      let token = self.match_kind("vec3")?;
      Ok(token)
    } else if ["vec4"].contains(&self.current()) {
      let token = self.match_kind("vec4")?;
      Ok(token)
    } else {
      Err(UnexpectedToken(self.current_token(), vec!["vec2", "vec3", "vec4", "void"]))
    }
  }
}