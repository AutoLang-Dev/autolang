use crate::{TextRange, parse::SyntaxError};

pub fn merge_errors(
  old_errors: &[SyntaxError],
  old_dirty_range: TextRange,
  new_dirty_range: TextRange,
  delta: i64,
  new_errors: &[SyntaxError],
  include_end: bool,
) -> Vec<SyntaxError> {
  let old_start = u32::from(old_dirty_range.start());
  let old_end = u32::from(old_dirty_range.end());
  let new_start = u32::from(new_dirty_range.start());
  let mut errors = Vec::new();

  for error in old_errors {
    if old_start <= error.offset
      && (error.offset < old_end || include_end && error.offset == old_end)
    {
      continue;
    }

    let offset = if error.offset >= old_end {
      (i64::from(error.offset) + delta) as u32
    } else {
      error.offset
    };
    errors.push(SyntaxError { offset, ..*error });
  }

  errors.extend(new_errors.iter().map(|error| SyntaxError {
    error: error.error,
    offset: new_start + error.offset,
  }));
  errors.sort_by_key(|error| error.offset);
  errors
}
