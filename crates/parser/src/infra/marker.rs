#[derive(Debug)]
pub struct Marker {
  pub pos: u32,
  done: bool,
}

impl Marker {
  pub fn new(pos: u32) -> Self {
    Self { pos, done: false }
  }

  pub fn defuse(&mut self) {
    self.done = true;
  }
}

impl Drop for Marker {
  fn drop(&mut self) {
    if !self.done {
      panic!("marker must be completed or abandoned");
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompletedMarker {
  pub pos: u32,
}

impl CompletedMarker {
  pub fn new(pos: u32) -> Self {
    Self { pos }
  }
}
