pub(super) struct IdGenerator {
    index: usize,
}

impl IdGenerator {
    pub fn new() -> IdGenerator {
        IdGenerator { index: 0 }
    }
}

impl Iterator for IdGenerator {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        self.index += 1;
        Some(self.index)
    }
}
