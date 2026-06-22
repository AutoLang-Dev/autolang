use crate::{
  SyntaxKind::{self, Eof},
  T,
  infra::{
    event::{self, Error, Event, Output},
    input::Input,
    marker::{CompletedMarker, Marker},
  },
};
use std::num::NonZeroU32;

#[derive(Debug)]
pub struct Parser {
  input: Input,
  pos: usize,
  events: Vec<Event>,
  errors: Vec<Error>,
}

impl Parser {
  pub fn new(input: Input) -> Self {
    Self {
      input,
      pos: 0,
      events: Vec::new(),
      errors: Vec::new(),
    }
  }

  pub fn finish(self) -> Output {
    event::process(self.events, self.errors)
  }

  pub fn start(&mut self) -> Marker {
    let pos = self
      .events
      .len()
      .try_into()
      .expect("too many parser events");
    self.events.push(Event::Placeholder);
    Marker::new(pos)
  }

  pub fn complete(&mut self, mut marker: Marker, kind: SyntaxKind) -> CompletedMarker {
    marker.defuse();

    let event = &mut self.events[marker.pos as usize];
    assert!(matches!(event, Event::Placeholder));
    *event = Event::Start {
      kind,
      forward_parent: None,
    };

    self.events.push(Event::Finish);

    CompletedMarker::new(marker.pos)
  }

  pub fn abandon(&mut self, mut marker: Marker) {
    marker.defuse();

    let pos = marker.pos as usize;
    if pos == self.events.len() - 1 {
      let event = self.events.pop().expect("marker event must exist");
      assert!(matches!(event, Event::Placeholder));
    }
  }

  pub fn precede(&mut self, marker: CompletedMarker) -> Marker {
    let new_start = self
      .events
      .len()
      .try_into()
      .expect("too many parser events");
    self.events.push(Event::Placeholder);

    match &mut self.events[marker.pos as usize] {
      Event::Start { forward_parent, .. } => {
        assert!(forward_parent.is_none());
        *forward_parent =
          Some(NonZeroU32::new(new_start - marker.pos).expect("non-zero forward parent"));
      }
      event => panic!("expected start event, got {event:?}"),
    }

    Marker::new(new_start)
  }

  pub fn current(&self) -> SyntaxKind {
    self.nth(0)
  }

  pub fn pos(&self) -> usize {
    self.pos
  }

  pub fn nth(&self, n: usize) -> SyntaxKind {
    self.input.kind(self.pos + n)
  }

  pub fn at_eof(&self) -> bool {
    self.current() == Eof
  }

  pub fn at(&self, kind: SyntaxKind) -> bool {
    self.nth_at(0, kind)
  }

  pub fn nth_at(&self, n: usize, kind: SyntaxKind) -> bool {
    match kind {
      T![->] => self.at_composite2(n, T![-], T![>]),
      T![..] => self.at_composite2(n, T![.], T![.]),
      T![.|] => self.at_composite2(n, T![.], T![|]),
      T![++] => self.at_composite2(n, T![+], T![+]),
      T![--] => self.at_composite2(n, T![-], T![-]),
      T![+=] => self.at_composite2(n, T![+], T![=]),
      T![-=] => self.at_composite2(n, T![-], T![=]),
      T![*=] => self.at_composite2(n, T![*], T![=]),
      T![/=] => self.at_composite2(n, T![/], T![=]),
      T![%=] => self.at_composite2(n, T![%], T![=]),
      T![~=] => self.at_composite2(n, T![~], T![=]),
      T![==] => self.at_composite2(n, T![=], T![=]),
      T![!=] => self.at_composite2(n, T![!], T![=]),
      T![>=] => self.at_composite2(n, T![>], T![=]),
      T![<=] => self.at_composite2(n, T![<], T![=]),
      T![<<] => self.at_composite2(n, T![<], T![<]),
      T![>>] => self.at_composite2(n, T![>], T![>]),
      T![&&] => self.at_composite2(n, T![&], T![&]),
      T![||] => self.at_composite2(n, T![|], T![|]),
      T![::] => self.at_composite2(n, T![:], T![:]),

      T![...] => self.at_composite3(n, T![.], T![.], T![.]),
      T![<<=] => self.at_composite3(n, T![<], T![<], T![=]),
      T![>>=] => self.at_composite3(n, T![>], T![>], T![=]),

      _ => self.nth(n) == kind,
    }
  }

  fn at_composite2(&self, n: usize, first: SyntaxKind, second: SyntaxKind) -> bool {
    self.nth(n) == first && self.nth(n + 1) == second && self.input.is_joint(self.pos + n)
  }

  fn at_composite3(
    &self,
    n: usize,
    first: SyntaxKind,
    second: SyntaxKind,
    third: SyntaxKind,
  ) -> bool {
    self.nth(n) == first
      && self.nth(n + 1) == second
      && self.nth(n + 2) == third
      && self.input.is_joint(self.pos + n)
      && self.input.is_joint(self.pos + n + 1)
  }

  pub fn bump(&mut self, kind: SyntaxKind) {
    assert!(self.bump_if(kind));
  }

  pub fn bump_any(&mut self) {
    let kind = self.current();
    if kind == Eof {
      panic!("cannot bump EOF");
    }

    self.events.push(Event::Token {
      kind,
      n_raw_tokens: 1,
    });
    self.pos += 1;
  }

  pub fn bump_if(&mut self, kind: SyntaxKind) -> bool {
    if !self.at(kind) {
      return false;
    }

    let n_raw_tokens = composite_n_raw_tokens(kind);
    self.do_bump(kind, n_raw_tokens);
    true
  }

  fn do_bump(&mut self, kind: SyntaxKind, n_raw_tokens: u8) {
    self.events.push(Event::Token { kind, n_raw_tokens });
    self.pos += n_raw_tokens as usize;
  }

  pub fn expect(&mut self, kind: SyntaxKind) -> bool {
    if self.bump_if(kind) {
      return true;
    }

    self.error(Error::Expected {
      expected: kind,
      actual: self.current(),
    });
    false
  }

  pub fn error(&mut self, error: Error) {
    let index = self
      .errors
      .len()
      .try_into()
      .expect("too many parser errors");
    self.errors.push(error);
    self.events.push(Event::Error(index));
  }
}

fn composite_n_raw_tokens(kind: SyntaxKind) -> u8 {
  match kind {
    T![->]
    | T![..]
    | T![.|]
    | T![++]
    | T![--]
    | T![+=]
    | T![-=]
    | T![*=]
    | T![/=]
    | T![%=]
    | T![~=]
    | T![==]
    | T![!=]
    | T![>=]
    | T![<=]
    | T![<<]
    | T![>>]
    | T![&&]
    | T![||]
    | T![::] => 2,
    T![...] | T![<<=] | T![>>=] => 3,
    _ => 1,
  }
}
