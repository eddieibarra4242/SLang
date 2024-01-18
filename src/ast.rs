use crate::scanner::Token;

#[derive(Clone, Debug)]
pub(crate) enum Expr {
  // todo: add spans for error messages.
  IdExpr(Id),
  //              array, index
  ArrayLookupExpr(Box<Expr>, Box<Expr>),
  UnaryExpr(Token, Box<Expr>),
  BinaryExpr(Token, Box<Expr>, Box<Expr>),
  LiteralExpr(Literal),
  FunctionExpr(Id, Vec<Expr>), // fixme: function names are more complicated than ids...
}

#[derive(Clone, Debug)]
pub(crate) enum Literal {
  Number(Token),
  Boolean(Token, bool),
  Array(Vec<Expr>)
}

#[derive(Clone, Debug)]
pub(crate) struct Id {
  token: Token
}

impl Id {
  pub(crate) fn new(token: Token) -> Self {
    Id {
      token
    }
  }
}