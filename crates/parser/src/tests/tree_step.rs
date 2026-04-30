use crate::{
  Error, Output, Step,
  SyntaxKind::*,
  T, TreeStep,
  infra::{lexed::LexedStr, tree_step::emit_tree_steps},
  parse,
};

fn collect_tree_steps<'text>(
  lexed: &LexedStr<'text>,
  output: &Output,
) -> (Vec<TreeStep<'text>>, bool) {
  let mut steps = Vec::new();
  let is_eof = emit_tree_steps(lexed, output, &mut |step| steps.push(step));
  (steps, is_eof)
}

macro_rules! tree_snap {
  ($input:literal, $output:expr $(,)?) => {{
    let input = $input;
    let lexed = LexedStr::new(input);
    let output = $output;
    let (steps, is_eof) = collect_tree_steps(&lexed, &output);

    assert!(is_eof);
    snap!(input, steps);
  }};
}

#[test]
fn emits_leading_trivia() {
  tree_snap!(
    "  foo",
    Output::new(
      vec![Step::Token {
        kind: Ident,
        n_raw_tokens: 1,
      }],
      Vec::new(),
    ),
  );
}

#[test]
fn emits_inter_token_trivia() {
  tree_snap!(
    "foo : mod",
    Output::new(
      vec![
        Step::Token {
          kind: Ident,
          n_raw_tokens: 1,
        },
        Step::Token {
          kind: T![:],
          n_raw_tokens: 1,
        },
        Step::Token {
          kind: T![mod],
          n_raw_tokens: 1,
        },
      ],
      Vec::new(),
    ),
  );
}

#[test]
fn emits_trailing_trivia() {
  tree_snap!(
    "foo // comment",
    Output::new(
      vec![Step::Token {
        kind: Ident,
        n_raw_tokens: 1,
      }],
      Vec::new(),
    ),
  );
}

#[test]
fn emits_shebang_as_leading_trivia() {
  let input = "#!/usr/bin/env autolang\nfoo";
  let lexed = LexedStr::new(input);
  assert_eq!(lexed.text_start(2), input.len() as u32 - "foo".len() as u32);

  let output = Output::new(
    vec![Step::Token {
      kind: Ident,
      n_raw_tokens: 1,
    }],
    Vec::new(),
  );

  let (steps, is_eof) = collect_tree_steps(&lexed, &output);

  assert!(is_eof);
  snap!(input, steps);
}

#[test]
fn emits_composite_token_text() {
  tree_snap!(
    "a->b",
    Output::new(
      vec![
        Step::Token {
          kind: Ident,
          n_raw_tokens: 1,
        },
        Step::Token {
          kind: T![->],
          n_raw_tokens: 2,
        },
        Step::Token {
          kind: Ident,
          n_raw_tokens: 1,
        },
      ],
      Vec::new(),
    ),
  );
}

#[test]
fn does_not_merge_trivia_into_composite_token() {
  tree_snap!(
    "a- >b",
    Output::new(
      vec![
        Step::Token {
          kind: Ident,
          n_raw_tokens: 1,
        },
        Step::Token {
          kind: T![-],
          n_raw_tokens: 1,
        },
        Step::Token {
          kind: T![>],
          n_raw_tokens: 1,
        },
        Step::Token {
          kind: Ident,
          n_raw_tokens: 1,
        },
      ],
      Vec::new(),
    ),
  );
}

#[test]
fn emits_error_with_copied_error() {
  let input = "foo";
  let lexed = LexedStr::new(input);
  let error = Error::Expected {
    expected: T![:],
    actual: Ident,
  };
  let output = Output::new(vec![Step::Error(0)], vec![error]);

  let (steps, is_eof) = collect_tree_steps(&lexed, &output);

  assert!(!is_eof);
  assert_eq!(steps, vec![TreeStep::Error(error)]);
  snap!(input, steps);
}

#[test]
fn emits_trailing_trivia_before_final_exit() {
  let input = "foo: mod; // trailing";
  let lexed = LexedStr::new(input);
  let output = parse(&lexed);
  let (steps, is_eof) = collect_tree_steps(&lexed, &output);

  assert!(is_eof);
  snap!(input, steps);
}
