pub trait SliceExt<T> {
    fn try_split_at(&self, mid: usize) -> Option<(&[T], &[T])>;
}

impl<T> SliceExt<T> for &[T] {
    fn try_split_at(&self, mid: usize) -> Option<(&[T], &[T])> {
        if mid > self.len() {
            None
        } else {
            Some(self.split_at(mid))
        }
    }
}
