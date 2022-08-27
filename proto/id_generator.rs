pub(crate) struct IdGenerator {
    index: usize,
}

impl IdGenerator {
    pub fn new() -> IdGenerator {
        IdGenerator { index: 0 }
    }

    pub fn create<A, T>(&mut self, args: A) -> T
    where
        T: UniqueId<Args = A>,
    {
        T::create_with_id(self.next().unwrap(), args)
    }
}

impl Iterator for IdGenerator {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        self.index += 1;
        Some(self.index)
    }
}

pub(crate) trait UniqueId {
    type Args;
    fn create_with_id(id: usize, args: Self::Args) -> Self;
}
