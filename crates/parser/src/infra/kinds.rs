//! Syntax tree node and token kinds.
//!
//! This follows the shape used by rust-analyzer: one compact enum contains
//! trivia, leaf tokens, contextual keywords, and composite syntax nodes. The
//! lexer may emit coarser tokens than this enum; parser/token-source code is
//! expected to refine identifiers into keywords and stitch multi-char puncts.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SyntaxKind {
  // Trivia.
  Whitespace,
  Comment,
  Shebang,

  // Literals and names.
  Ident,
  Underscore,
  Label,
  Int,
  Char,
  Byte,
  String,
  RawString,
  UnknownPrefix,

  // One-char tokens.
  Semi,
  Comma,
  Dot,
  OpenParen,
  CloseParen,
  OpenBrack,
  CloseBrack,
  OpenBrace,
  CloseBrace,
  At,
  Hash,
  Tilde,
  Question,
  Colon,
  Dollar,
  Eq,
  Bang,
  Lt,
  Gt,
  Minus,
  Amp,
  Pipe,
  Plus,
  Star,
  Slash,
  Backslash,
  Caret,
  Percent,

  // Multi-char tokens.
  ThinArrow,
  DotDot,
  DotPipe,
  PlusPlus,
  MinusMinus,
  PlusEq,
  MinusEq,
  StarEq,
  SlashEq,
  PercentEq,
  TildeEq,
  EqEq,
  BangEq,
  GtEq,
  LtEq,
  Shl,
  Shr,
  AmpAmp,
  PipePipe,
  DotDotDot,
  ShlEq,
  ShrEq,
  ColonColon,

  // Keywords.
  KwAs,
  KwBreak,
  KwCase,
  KwCont,
  KwElse,
  KwFalse,
  KwFn,
  KwFor,
  KwIf,
  KwImpl,
  KwIn,
  KwIterate,
  KwMod,
  KwMut,
  KwNominal,
  KwPri,
  KwPro,
  KwPub,
  KwReturn,
  KwSelf,
  KwSuper,
  KwTrue,
  KwType,
  KwUnit,
  KwUsing,
  KwWhile,

  // Error recovery and sentinels.
  Eof,
  Error,
  Unknown,

  // Root.
  SourceFile,

  // Attributes.
  Attr,
  AttrInner,
  AttrItem,
  AttrArg,

  // Items.
  ModuleInner,
  Module,
  BindingItem,
  FunctionItem,
  TypeItem,
  ImplItem,
  AssociatedItem,
  UsingItem,
  ModuleItem,
  EmptyItem,
  Visibility,
  ErrorItem,

  // Bindings, parameters, and fields.
  Binding,
  ParameterList,
  Parameter,
  TupleField,
  StructField,
  FieldName,
  FieldValue,

  // Types.
  TypeKind,
  InferType,
  PathType,
  FnPtrType,
  RefType,
  PtrType,
  TupleType,
  ParenType,
  ArrayType,
  SliceType,
  StructType,
  ErrorType,

  // Statements.
  ExprStmt,
  AssignStmt,

  // Patterns.
  WildcardPat,
  IdentPat,
  ErrorPat,

  // Expressions.
  WildcardExpr,
  TupleExpr,
  ParenExpr,
  ArrayExpr,
  RepeatExpr,
  StructExpr,
  BlockExpr,
  LiteralExpr,
  PathExpr,
  CaseExpr,
  IfExpr,
  WhileExpr,
  ForExpr,
  IterateExpr,
  BinaryExpr,
  PrefixExpr,
  RefExpr,
  PostfixExpr,
  CastExpr,
  CallExpr,
  IndexExpr,
  IndexArg,
  ReturnExpr,
  BreakExpr,
  ContinueExpr,
  ClosureExpr,
  FieldExpr,
  MethodCallExpr,
  LabeledExpr,
  ChainExpr,
  ElseClause,
  CaseArmList,
  CaseArm,
  ArgList,
  ErrorExpr,

  // Paths and use trees.
  Path,
  PathSegment,
  UsingTree,
  UsingTreeList,
  Rename,

  // Token trees.
  TokenTree,
  DelimitedTokenTree,
}

impl SyntaxKind {
  pub fn is_trivia(self) -> bool {
    matches!(self, Self::Whitespace | Self::Comment | Self::Shebang)
  }

  pub fn is_keyword(self) -> bool {
    matches!(
      self,
      Self::KwAs
        | Self::KwBreak
        | Self::KwCase
        | Self::KwCont
        | Self::KwElse
        | Self::KwFalse
        | Self::KwFn
        | Self::KwFor
        | Self::KwIf
        | Self::KwImpl
        | Self::KwIn
        | Self::KwIterate
        | Self::KwMod
        | Self::KwMut
        | Self::KwNominal
        | Self::KwPri
        | Self::KwPro
        | Self::KwPub
        | Self::KwReturn
        | Self::KwSelf
        | Self::KwSuper
        | Self::KwTrue
        | Self::KwType
        | Self::KwUnit
        | Self::KwUsing
        | Self::KwWhile
    )
  }

  pub fn is_literal(self) -> bool {
    matches!(
      self,
      Self::Int
        | Self::Char
        | Self::Byte
        | Self::String
        | Self::RawString
        | Self::KwTrue
        | Self::KwFalse
    )
  }

  pub fn is_punct(self) -> bool {
    matches!(
      self,
      Self::Semi
        | Self::Comma
        | Self::Dot
        | Self::OpenParen
        | Self::CloseParen
        | Self::OpenBrack
        | Self::CloseBrack
        | Self::OpenBrace
        | Self::CloseBrace
        | Self::At
        | Self::Hash
        | Self::Tilde
        | Self::Question
        | Self::Colon
        | Self::Dollar
        | Self::Eq
        | Self::Bang
        | Self::Lt
        | Self::Gt
        | Self::Minus
        | Self::Amp
        | Self::Pipe
        | Self::Plus
        | Self::Star
        | Self::Slash
        | Self::Backslash
        | Self::Caret
        | Self::Percent
        | Self::ThinArrow
        | Self::DotDot
        | Self::DotPipe
        | Self::PlusPlus
        | Self::MinusMinus
        | Self::PlusEq
        | Self::MinusEq
        | Self::StarEq
        | Self::SlashEq
        | Self::PercentEq
        | Self::TildeEq
        | Self::EqEq
        | Self::BangEq
        | Self::GtEq
        | Self::LtEq
        | Self::Shl
        | Self::Shr
        | Self::AmpAmp
        | Self::PipePipe
        | Self::DotDotDot
        | Self::ShlEq
        | Self::ShrEq
        | Self::ColonColon
    )
  }

  pub fn fixed_text(self) -> Option<&'static str> {
    let text = match self {
      Self::Underscore => "_",
      Self::Semi => ";",
      Self::Comma => ",",
      Self::Dot => ".",
      Self::OpenParen => "(",
      Self::CloseParen => ")",
      Self::OpenBrack => "[",
      Self::CloseBrack => "]",
      Self::OpenBrace => "{",
      Self::CloseBrace => "}",
      Self::At => "@",
      Self::Hash => "#",
      Self::Tilde => "~",
      Self::Question => "?",
      Self::Colon => ":",
      Self::Dollar => "$",
      Self::Eq => "=",
      Self::Bang => "!",
      Self::Lt => "<",
      Self::Gt => ">",
      Self::Minus => "-",
      Self::Amp => "&",
      Self::Pipe => "|",
      Self::Plus => "+",
      Self::Star => "*",
      Self::Slash => "/",
      Self::Backslash => "\\",
      Self::Caret => "^",
      Self::Percent => "%",
      Self::ThinArrow => "->",
      Self::DotDot => "..",
      Self::DotPipe => ".|",
      Self::PlusPlus => "++",
      Self::MinusMinus => "--",
      Self::PlusEq => "+=",
      Self::MinusEq => "-=",
      Self::StarEq => "*=",
      Self::SlashEq => "/=",
      Self::PercentEq => "%=",
      Self::TildeEq => "~=",
      Self::EqEq => "==",
      Self::BangEq => "!=",
      Self::GtEq => ">=",
      Self::LtEq => "<=",
      Self::Shl => "<<",
      Self::Shr => ">>",
      Self::AmpAmp => "&&",
      Self::PipePipe => "||",
      Self::DotDotDot => "...",
      Self::ShlEq => "<<=",
      Self::ShrEq => ">>=",
      Self::ColonColon => "::",
      Self::KwAs => "as",
      Self::KwBreak => "break",
      Self::KwCase => "case",
      Self::KwCont => "cont",
      Self::KwElse => "else",
      Self::KwFalse => "false",
      Self::KwFn => "fn",
      Self::KwFor => "for",
      Self::KwIf => "if",
      Self::KwImpl => "impl",
      Self::KwIn => "in",
      Self::KwIterate => "iterate",
      Self::KwMod => "mod",
      Self::KwMut => "mut",
      Self::KwNominal => "nominal",
      Self::KwPri => "pri",
      Self::KwPro => "pro",
      Self::KwPub => "pub",
      Self::KwReturn => "return",
      Self::KwSelf => "self",
      Self::KwSuper => "super",
      Self::KwTrue => "true",
      Self::KwType => "type",
      Self::KwUnit => "unit",
      Self::KwUsing => "using",
      Self::KwWhile => "while",
      _ => return None,
    };

    Some(text)
  }

  pub fn from_keyword(text: &str) -> Option<Self> {
    let kind = match text {
      "as" => Self::KwAs,
      "break" => Self::KwBreak,
      "case" => Self::KwCase,
      "cont" => Self::KwCont,
      "else" => Self::KwElse,
      "false" => Self::KwFalse,
      "fn" => Self::KwFn,
      "for" => Self::KwFor,
      "if" => Self::KwIf,
      "impl" => Self::KwImpl,
      "in" => Self::KwIn,
      "iterate" => Self::KwIterate,
      "mod" => Self::KwMod,
      "mut" => Self::KwMut,
      "nominal" => Self::KwNominal,
      "pri" => Self::KwPri,
      "pro" => Self::KwPro,
      "pub" => Self::KwPub,
      "return" => Self::KwReturn,
      "self" => Self::KwSelf,
      "super" => Self::KwSuper,
      "true" => Self::KwTrue,
      "type" => Self::KwType,
      "unit" => Self::KwUnit,
      "using" => Self::KwUsing,
      "while" => Self::KwWhile,
      _ => return None,
    };
    Some(kind)
  }

  pub fn from_punct(text: &str) -> Option<Self> {
    let kind = match text {
      ";" => Self::Semi,
      "," => Self::Comma,
      "." => Self::Dot,
      "_" => Self::Underscore,
      "(" => Self::OpenParen,
      ")" => Self::CloseParen,
      "[" => Self::OpenBrack,
      "]" => Self::CloseBrack,
      "{" => Self::OpenBrace,
      "}" => Self::CloseBrace,
      "@" => Self::At,
      "#" => Self::Hash,
      "~" => Self::Tilde,
      "?" => Self::Question,
      ":" => Self::Colon,
      "$" => Self::Dollar,
      "=" => Self::Eq,
      "!" => Self::Bang,
      "<" => Self::Lt,
      ">" => Self::Gt,
      "-" => Self::Minus,
      "&" => Self::Amp,
      "|" => Self::Pipe,
      "+" => Self::Plus,
      "*" => Self::Star,
      "/" => Self::Slash,
      "\\" => Self::Backslash,
      "^" => Self::Caret,
      "%" => Self::Percent,
      "->" => Self::ThinArrow,
      ".." => Self::DotDot,
      ".|" => Self::DotPipe,
      "++" => Self::PlusPlus,
      "--" => Self::MinusMinus,
      "+=" => Self::PlusEq,
      "-=" => Self::MinusEq,
      "*=" => Self::StarEq,
      "/=" => Self::SlashEq,
      "%=" => Self::PercentEq,
      "~=" => Self::TildeEq,
      "==" => Self::EqEq,
      "!=" => Self::BangEq,
      ">=" => Self::GtEq,
      "<=" => Self::LtEq,
      "<<" => Self::Shl,
      ">>" => Self::Shr,
      "&&" => Self::AmpAmp,
      "||" => Self::PipePipe,
      "..." => Self::DotDotDot,
      "<<=" => Self::ShlEq,
      ">>=" => Self::ShrEq,
      "::" => Self::ColonColon,
      _ => return None,
    };
    Some(kind)
  }
}

#[macro_export]
macro_rules! T {
  [;] => { $crate::SyntaxKind::Semi };
  [,] => { $crate::SyntaxKind::Comma };
  [.] => { $crate::SyntaxKind::Dot };
  [_] => { $crate::SyntaxKind::Underscore };
  ['('] => { $crate::SyntaxKind::OpenParen };
  [')'] => { $crate::SyntaxKind::CloseParen };
  ['['] => { $crate::SyntaxKind::OpenBrack };
  [']'] => { $crate::SyntaxKind::CloseBrack };
  ['{'] => { $crate::SyntaxKind::OpenBrace };
  ['}'] => { $crate::SyntaxKind::CloseBrace };
  [@] => { $crate::SyntaxKind::At };
  [#] => { $crate::SyntaxKind::Hash };
  [~] => { $crate::SyntaxKind::Tilde };
  [?] => { $crate::SyntaxKind::Question };
  [:] => { $crate::SyntaxKind::Colon };
  ['$'] => { $crate::SyntaxKind::Dollar };
  [=] => { $crate::SyntaxKind::Eq };
  [!] => { $crate::SyntaxKind::Bang };
  [<] => { $crate::SyntaxKind::Lt };
  [>] => { $crate::SyntaxKind::Gt };
  [-] => { $crate::SyntaxKind::Minus };
  [&] => { $crate::SyntaxKind::Amp };
  [|] => { $crate::SyntaxKind::Pipe };
  [+] => { $crate::SyntaxKind::Plus };
  [*] => { $crate::SyntaxKind::Star };
  [/] => { $crate::SyntaxKind::Slash };
  ['\\'] => { $crate::SyntaxKind::Backslash };
  [^] => { $crate::SyntaxKind::Caret };
  [%] => { $crate::SyntaxKind::Percent };
  [->] => { $crate::SyntaxKind::ThinArrow };
  [..] => { $crate::SyntaxKind::DotDot };
  [.|] => { $crate::SyntaxKind::DotPipe };
  [++] => { $crate::SyntaxKind::PlusPlus };
  [--] => { $crate::SyntaxKind::MinusMinus };
  [+=] => { $crate::SyntaxKind::PlusEq };
  [-=] => { $crate::SyntaxKind::MinusEq };
  [*=] => { $crate::SyntaxKind::StarEq };
  [/=] => { $crate::SyntaxKind::SlashEq };
  [%=] => { $crate::SyntaxKind::PercentEq };
  [~=] => { $crate::SyntaxKind::TildeEq };
  [==] => { $crate::SyntaxKind::EqEq };
  [!=] => { $crate::SyntaxKind::BangEq };
  [>=] => { $crate::SyntaxKind::GtEq };
  [<=] => { $crate::SyntaxKind::LtEq };
  [<<] => { $crate::SyntaxKind::Shl };
  [>>] => { $crate::SyntaxKind::Shr };
  [&&] => { $crate::SyntaxKind::AmpAmp };
  [||] => { $crate::SyntaxKind::PipePipe };
  [...] => { $crate::SyntaxKind::DotDotDot };
  [<<=] => { $crate::SyntaxKind::ShlEq };
  [>>=] => { $crate::SyntaxKind::ShrEq };
  [::] => { $crate::SyntaxKind::ColonColon };

  [as] => { $crate::SyntaxKind::KwAs };
  [break] => { $crate::SyntaxKind::KwBreak };
  [case] => { $crate::SyntaxKind::KwCase };
  [cont] => { $crate::SyntaxKind::KwCont };
  [else] => { $crate::SyntaxKind::KwElse };
  [false] => { $crate::SyntaxKind::KwFalse };
  [fn] => { $crate::SyntaxKind::KwFn };
  [for] => { $crate::SyntaxKind::KwFor };
  [if] => { $crate::SyntaxKind::KwIf };
  [impl] => { $crate::SyntaxKind::KwImpl };
  [in] => { $crate::SyntaxKind::KwIn };
  [iterate] => { $crate::SyntaxKind::KwIterate };
  [mod] => { $crate::SyntaxKind::KwMod };
  [mut] => { $crate::SyntaxKind::KwMut };
  [nominal] => { $crate::SyntaxKind::KwNominal };
  [pri] => { $crate::SyntaxKind::KwPri };
  [pro] => { $crate::SyntaxKind::KwPro };
  [pub] => { $crate::SyntaxKind::KwPub };
  [return] => { $crate::SyntaxKind::KwReturn };
  [self] => { $crate::SyntaxKind::KwSelf };
  [super] => { $crate::SyntaxKind::KwSuper };
  [true] => { $crate::SyntaxKind::KwTrue };
  [type] => { $crate::SyntaxKind::KwType };
  [unit] => { $crate::SyntaxKind::KwUnit };
  [using] => { $crate::SyntaxKind::KwUsing };
  [while] => { $crate::SyntaxKind::KwWhile };
}
