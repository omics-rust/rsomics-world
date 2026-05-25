//! Tokenizer and recursive-descent parser for the filter-expression grammar.

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedChar(char),
    UnexpectedToken(String),
    UnterminatedString,
    UnsupportedSyntax(String),
    EmptyExpression,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedChar(c) => write!(f, "unexpected character '{c}'"),
            ParseError::UnexpectedToken(s) => write!(f, "unexpected token '{s}'"),
            ParseError::UnterminatedString => write!(f, "unterminated string literal"),
            ParseError::UnsupportedSyntax(s) => write!(f, "unsupported syntax: {s}"),
            ParseError::EmptyExpression => write!(f, "empty expression"),
        }
    }
}

impl std::error::Error for ParseError {}

// ── Field reference ───────────────────────────────────────────────────────────

/// A field reference parsed from the expression.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldRef {
    /// `FMT/<tag>` or `FORMAT/<tag>` — per-sample FORMAT field.
    Fmt(String),
    /// `INFO/<tag>` — site-level INFO field.
    Info(String),
    /// `QUAL` — site-level QUAL column.
    Qual,
    /// `FILTER` — site-level FILTER string.
    Filter,
    /// `GT` — per-sample genotype with bcftools GT string semantics.
    Gt,
}

// ── Value ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Numeric literal.
    Num(f64),
    /// String literal (stripped of quotes).
    Str(String),
}

// ── Expression AST ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum CmpOp {
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogOp {
    /// `&`  — per-sample AND: sample passes if it satisfies BOTH sub-expressions.
    And,
    /// `&&` — bcftools "sample-wise AND": per-sample OR (sample passes if it satisfies EITHER).
    /// Named "vec-and" in bcftools filter.c; confusingly acts as per-sample OR.
    AndVec,
    /// `|`  — site-level OR (alias: per-sample OR, same as `AndVec` for setGT purposes).
    Or,
    /// `||` — site-level OR (same as `|` in practice).
    OrVec,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// `<field> <op> <value>`
    Cmp {
        field: FieldRef,
        op: CmpOp,
        val: Value,
    },
    /// Logical combination of two sub-expressions.
    Logic {
        op: LogOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    /// `(<expr>)`
    Paren(Box<Expr>),
}

// ── Token ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Ident(String),
    NumLit(f64),
    StrLit(String),
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
    /// Single `&` — per-sample AND (true intersection).
    And,
    /// Double `&&` — per-sample OR in bcftools semantics (vec-and).
    AndVec,
    /// Single `|` — OR.
    Or,
    /// Double `||` — OR (same as `|`).
    OrVec,
    LParen,
    RParen,
    Eof,
}

// ── Tokenizer ─────────────────────────────────────────────────────────────────

struct Lexer<'a> {
    src: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            src: src.as_bytes(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<u8> {
        let b = self.src.get(self.pos).copied();
        self.pos += 1;
        b
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\r' | b'\n')) {
            self.pos += 1;
        }
    }

    fn read_ident(&mut self) -> String {
        let start = self.pos - 1; // first char already consumed
        while matches!(
            self.peek(),
            Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'-' | b'.')
        ) {
            self.pos += 1;
        }
        // Include a trailing `/` to allow `FMT/DP` as a single ident
        if self.peek() == Some(b'/') {
            self.pos += 1;
            while matches!(
                self.peek(),
                Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'-')
            ) {
                self.pos += 1;
            }
        }
        String::from_utf8_lossy(&self.src[start..self.pos]).into_owned()
    }

    fn read_num(&mut self, first: u8) -> Result<f64, ParseError> {
        let start = self.pos - 1;
        if first == b'-' || first == b'+' {
            // sign only — digits must follow
        }
        while matches!(
            self.peek(),
            Some(b'0'..=b'9' | b'.' | b'e' | b'E' | b'+' | b'-')
        ) {
            self.pos += 1;
        }
        let s = std::str::from_utf8(&self.src[start..self.pos]).unwrap_or("0");
        s.parse::<f64>()
            .map_err(|_| ParseError::UnexpectedToken(s.to_owned()))
    }

    fn read_string(&mut self, quote: u8) -> Result<String, ParseError> {
        let mut s = String::new();
        loop {
            match self.advance() {
                None => return Err(ParseError::UnterminatedString),
                Some(b) if b == quote => break,
                Some(b'\\') => match self.advance() {
                    None => return Err(ParseError::UnterminatedString),
                    Some(c) => s.push(c as char),
                },
                Some(b) => s.push(b as char),
            }
        }
        Ok(s)
    }

    fn next_tok(&mut self) -> Result<Tok, ParseError> {
        self.skip_ws();
        let Some(b) = self.advance() else {
            return Ok(Tok::Eof);
        };
        match b {
            b'(' => Ok(Tok::LParen),
            b')' => Ok(Tok::RParen),
            b'<' => {
                if self.peek() == Some(b'=') {
                    self.pos += 1;
                    Ok(Tok::Le)
                } else {
                    Ok(Tok::Lt)
                }
            }
            b'>' => {
                if self.peek() == Some(b'=') {
                    self.pos += 1;
                    Ok(Tok::Ge)
                } else {
                    Ok(Tok::Gt)
                }
            }
            b'=' => {
                if self.peek() == Some(b'=') {
                    self.pos += 1;
                    Ok(Tok::Eq)
                } else {
                    // bare `=` also means equality in bcftools expressions (GT=".")
                    Ok(Tok::Eq)
                }
            }
            b'!' => {
                if self.peek() == Some(b'=') {
                    self.pos += 1;
                    Ok(Tok::Ne)
                } else {
                    Err(ParseError::UnexpectedChar('!'))
                }
            }
            b'&' => {
                if self.peek() == Some(b'&') {
                    self.pos += 1;
                    Ok(Tok::AndVec) // `&&` — bcftools vec-and (per-sample OR)
                } else {
                    Ok(Tok::And) // `&` — per-sample true AND
                }
            }
            b'|' => {
                if self.peek() == Some(b'|') {
                    self.pos += 1;
                    Ok(Tok::OrVec) // `||`
                } else {
                    Ok(Tok::Or) // `|`
                }
            }
            b'"' | b'\'' => {
                let s = self.read_string(b)?;
                Ok(Tok::StrLit(s))
            }
            b'~' | b'@' => Err(ParseError::UnsupportedSyntax(format!(
                "operator '{}' (regex/file-match) is not supported",
                b as char
            ))),
            b if b.is_ascii_alphabetic() || b == b'_' => {
                let ident = self.read_ident();
                // Check for unsupported bracket syntax TAG[0]
                if self.peek() == Some(b'[') {
                    return Err(ParseError::UnsupportedSyntax(format!(
                        "array-index syntax '{ident}[...]' is not supported"
                    )));
                }
                Ok(Tok::Ident(ident))
            }
            b if b.is_ascii_digit() || b == b'-' => self.read_num(b).map(Tok::NumLit),
            other => Err(ParseError::UnexpectedChar(other as char)),
        }
    }
}

// ── Parser ────────────────────────────────────────────────────────────────────

struct Parser<'a> {
    lex: Lexer<'a>,
    current: Tok,
}

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Result<Self, ParseError> {
        let mut lex = Lexer::new(src);
        let current = lex.next_tok()?;
        Ok(Self { lex, current })
    }

    fn advance(&mut self) -> Result<Tok, ParseError> {
        let old = std::mem::replace(&mut self.current, self.lex.next_tok()?);
        Ok(old)
    }

    fn expect_eof(&self) -> Result<(), ParseError> {
        if self.current == Tok::Eof {
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken(format!("{:?}", self.current)))
        }
    }

    /// Parse a field reference from an identifier token.
    fn parse_field(ident: &str) -> Result<FieldRef, ParseError> {
        let up = ident.to_ascii_uppercase();
        if up == "QUAL" {
            return Ok(FieldRef::Qual);
        }
        if up == "FILTER" {
            return Ok(FieldRef::Filter);
        }
        if up == "GT" {
            return Ok(FieldRef::Gt);
        }
        // Unsupported aggregation / special fields
        for bad in &[
            "N_MISSING",
            "F_MISSING",
            "N_ALT",
            "N_PASS",
            "F_PASS",
            "SMPL_MAX",
            "SMPL_MIN",
            "SMPL_AVG",
            "AC",
            "AN",
            "AF",
            "MAC",
            "MAF",
            "ILEN",
            "CHROM",
            "POS",
            "REF",
            "ALT",
        ] {
            if up == *bad {
                return Err(ParseError::UnsupportedSyntax(format!(
                    "field '{ident}' is not supported in rsomics-vcf-expr"
                )));
            }
        }
        if let Some(tag) = up
            .strip_prefix("FMT/")
            .or_else(|| up.strip_prefix("FORMAT/"))
        {
            return Ok(FieldRef::Fmt(tag.to_owned()));
        }
        if let Some(tag) = up.strip_prefix("INFO/") {
            return Ok(FieldRef::Info(tag.to_owned()));
        }
        // Bare tag — treat as FORMAT if no prefix (bcftools falls back to INFO for bare names,
        // but FORMAT fields are more common in -t q usage). We try FMT first at eval time.
        // Represent as INFO here; eval layer will check FORMAT first.
        Ok(FieldRef::Info(up))
    }

    fn parse_cmp_op(tok: &Tok) -> Option<CmpOp> {
        match tok {
            Tok::Lt => Some(CmpOp::Lt),
            Tok::Le => Some(CmpOp::Le),
            Tok::Gt => Some(CmpOp::Gt),
            Tok::Ge => Some(CmpOp::Ge),
            Tok::Eq => Some(CmpOp::Eq),
            Tok::Ne => Some(CmpOp::Ne),
            _ => None,
        }
    }

    /// Parse a primary expression: `(expr)` or `field op value`.
    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match &self.current {
            Tok::LParen => {
                self.advance()?;
                let inner = self.parse_or()?;
                if self.current != Tok::RParen {
                    return Err(ParseError::UnexpectedToken(format!("{:?}", self.current)));
                }
                self.advance()?;
                Ok(Expr::Paren(Box::new(inner)))
            }
            Tok::Ident(_) => {
                let Tok::Ident(ident) = self.advance()? else {
                    unreachable!()
                };
                let field = Self::parse_field(&ident)?;
                // Expect comparison operator next
                let op = Self::parse_cmp_op(&self.current)
                    .ok_or_else(|| ParseError::UnexpectedToken(format!("{:?}", self.current)))?;
                self.advance()?;
                // Expect value next
                let val = match &self.current {
                    Tok::NumLit(n) => {
                        let v = Value::Num(*n);
                        self.advance()?;
                        v
                    }
                    Tok::StrLit(_) => {
                        if let Tok::StrLit(s) = self.advance()? {
                            Value::Str(s)
                        } else {
                            unreachable!()
                        }
                    }
                    other => {
                        return Err(ParseError::UnexpectedToken(format!("{other:?}")));
                    }
                };
                Ok(Expr::Cmp { field, op, val })
            }
            other => Err(ParseError::UnexpectedToken(format!("{other:?}"))),
        }
    }

    fn tok_to_logop(tok: &Tok) -> Option<LogOp> {
        match tok {
            Tok::And => Some(LogOp::And),
            Tok::AndVec => Some(LogOp::AndVec),
            Tok::Or => Some(LogOp::Or),
            Tok::OrVec => Some(LogOp::OrVec),
            _ => None,
        }
    }

    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_primary()?;
        while matches!(self.current, Tok::And | Tok::AndVec) {
            let op = Self::tok_to_logop(&self.current).unwrap();
            self.advance()?;
            let rhs = self.parse_primary()?;
            lhs = Expr::Logic {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_and()?;
        while matches!(self.current, Tok::Or | Tok::OrVec) {
            let op = Self::tok_to_logop(&self.current).unwrap();
            self.advance()?;
            let rhs = self.parse_and()?;
            lhs = Expr::Logic {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse(mut self) -> Result<Expr, ParseError> {
        if self.current == Tok::Eof {
            return Err(ParseError::EmptyExpression);
        }
        let expr = self.parse_or()?;
        self.expect_eof()?;
        Ok(expr)
    }
}

/// Parse a bcftools-style filter expression into an AST.
pub fn parse_expr(src: &str) -> Result<Expr, ParseError> {
    Parser::new(src)?.parse()
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_fmt_dp_lt() {
        let expr = parse_expr("FMT/DP<5").unwrap();
        assert_eq!(
            expr,
            Expr::Cmp {
                field: FieldRef::Fmt("DP".into()),
                op: CmpOp::Lt,
                val: Value::Num(5.0),
            }
        );
    }

    #[test]
    fn parse_qual_ge() {
        let expr = parse_expr("QUAL>=30").unwrap();
        assert_eq!(
            expr,
            Expr::Cmp {
                field: FieldRef::Qual,
                op: CmpOp::Ge,
                val: Value::Num(30.0)
            }
        );
    }

    #[test]
    fn parse_gt_eq_missing() {
        let expr = parse_expr("GT=\".\"").unwrap();
        assert_eq!(
            expr,
            Expr::Cmp {
                field: FieldRef::Gt,
                op: CmpOp::Eq,
                val: Value::Str(".".into())
            }
        );
    }

    #[test]
    fn parse_and_expr() {
        // `&` is per-sample true AND
        let expr = parse_expr("FMT/DP>10 & FMT/GQ>=20").unwrap();
        matches!(expr, Expr::Logic { op: LogOp::And, .. });
    }

    #[test]
    fn parse_andvec_expr() {
        // `&&` is bcftools "vec-and" (per-sample OR)
        let expr = parse_expr("FMT/DP>10 && FMT/GQ>=20").unwrap();
        matches!(
            expr,
            Expr::Logic {
                op: LogOp::AndVec,
                ..
            }
        );
    }

    #[test]
    fn parse_or_expr() {
        let expr = parse_expr("FMT/DP<5 || FMT/GQ<10").unwrap();
        matches!(
            expr,
            Expr::Logic {
                op: LogOp::OrVec,
                ..
            }
        );
    }

    #[test]
    fn parse_paren() {
        let expr = parse_expr("(FMT/DP<5)").unwrap();
        matches!(expr, Expr::Paren(_));
    }

    #[test]
    fn unsupported_regex_rejected() {
        assert!(parse_expr("FILTER~PASS").is_err());
    }

    #[test]
    fn unsupported_array_index_rejected() {
        assert!(parse_expr("FMT/AD[0]>5").is_err());
    }

    #[test]
    fn empty_expression_rejected() {
        assert!(parse_expr("").is_err());
    }

    #[test]
    fn parse_format_prefix() {
        let expr = parse_expr("FORMAT/DP<5").unwrap();
        assert_eq!(
            expr,
            Expr::Cmp {
                field: FieldRef::Fmt("DP".into()),
                op: CmpOp::Lt,
                val: Value::Num(5.0),
            }
        );
    }

    #[test]
    fn parse_info_field() {
        let expr = parse_expr("INFO/AF<0.05").unwrap();
        assert_eq!(
            expr,
            Expr::Cmp {
                field: FieldRef::Info("AF".into()),
                op: CmpOp::Lt,
                val: Value::Num(0.05),
            }
        );
    }
}
