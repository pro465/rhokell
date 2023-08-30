use std::fmt::*;

#[derive(Clone, Debug)]
pub struct Error {
    pub loc: Loc,
    pub ty: ErrorTy,
    pub desc: String,
}

impl Error {
    pub fn report(&self) {
        eprintln!("{} @ {}: {}", self.ty, self.loc, self.desc);
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Loc {
    pub line: u64,
    pub col: u64,
}

impl Loc {
    pub fn new() -> Self {
        Self { line: 1, col: 1 }
    }
    pub fn new_line(&mut self) {
        self.line += 1;
        self.col = 1;
    }
    pub fn col(&mut self) {
        self.col += 1;
    }
}

impl Display for Loc {
    fn fmt<'a>(&self, fmt: &mut Formatter<'a>) -> Result {
        write!(fmt, "line {}, column {}", self.line, self.col)
    }
}

impl Default for Loc {
    fn default() -> Self {
        Loc::new()
    }
}

#[derive(Clone, Debug)]
pub enum ErrorTy {
    SyntaxError,
    CExprError,
}

impl Display for ErrorTy {
    fn fmt<'a>(&self, fmt: &mut Formatter<'a>) -> Result {
        use ErrorTy::*;
        match self {
            SyntaxError => write!(fmt, "syntax error"),
            CExprError => write!(fmt, "closedness check error"),
        }
    }
}
