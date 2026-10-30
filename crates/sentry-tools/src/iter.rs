use std::slice::Iter;

pub trait EnumIterator<T> {
    fn iterator() -> Iter<'static, T>;
}

#[derive(Debug)]
pub struct FieldDiff {
    pub field: &'static str,
    pub are_equal: bool,
}
pub trait CompareFields<T> {
    fn compare_fields(&self, other: &T) -> Vec<FieldDiff>;
    fn is_equal_to(&self, other: &T) -> bool {
        self.compare_fields(other).iter().all(|d| d.are_equal)
    }
}