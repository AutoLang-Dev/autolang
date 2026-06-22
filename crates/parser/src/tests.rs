macro_rules! snap {
  ($desc:expr, $value:expr $(,)?) => {{
    insta::with_settings!({
      description => $desc,
    }, {
      insta::assert_debug_snapshot!($value);
    });
  }};
}

mod event;
mod grammar;
mod input;
mod lexed;
mod parser;
mod tree_step;
