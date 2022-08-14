pub trait Scope {
    type Declaration;
    fn resolve<'scope>(&'scope self, name: &str) -> Option<&'scope Self::Declaration>;
}
