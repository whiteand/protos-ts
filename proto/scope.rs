use super::package::Declaration;

pub(super) trait Scope {
    fn resolve<'scope>(&'scope self, name: &str) -> Option<&'scope Declaration>;
}
