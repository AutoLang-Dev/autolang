use crate::SyntaxKind;

type Bits = u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
  kind: Vec<SyntaxKind>,
  joint: Vec<Bits>,
}

impl Input {
  pub fn with_capacity(capacity: usize) -> Self {
    Self {
      kind: Vec::with_capacity(capacity),
      joint: Vec::with_capacity(capacity.div_ceil(Bits::BITS as usize)),
    }
  }

  pub fn push(&mut self, kind: SyntaxKind) {
    let idx = self.len();
    if idx.is_multiple_of(Bits::BITS as usize) {
      self.joint.push(0);
    }
    self.kind.push(kind);
  }

  pub fn was_joint(&mut self) {
    let n = self.len() - 1;
    let (idx, b_idx) = bit_index(n);
    self.joint[idx] |= 1 << b_idx;
  }

  pub fn kind(&self, idx: usize) -> SyntaxKind {
    self.kind.get(idx).copied().unwrap_or(SyntaxKind::Eof)
  }

  pub fn is_joint(&self, idx: usize) -> bool {
    if idx >= self.len() {
      return false;
    }
    let (idx, b_idx) = bit_index(idx);
    self.joint[idx] & (1 << b_idx) != 0
  }

  pub fn len(&self) -> usize {
    self.kind.len()
  }
}

fn bit_index(n: usize) -> (usize, usize) {
  let idx = n / Bits::BITS as usize;
  let b_idx = n % Bits::BITS as usize;
  (idx, b_idx)
}
