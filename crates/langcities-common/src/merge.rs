pub trait Merge<T> {
    fn merge_with(&mut self, rhs: T);
}

impl<T> Merge<Option<T>> for Option<T> {
    fn merge_with(&mut self, rhs: Option<T>) {
        if let Some(v) = rhs {
            self.replace(v);
        }
    }
}

impl<T: Clone> Merge<Option<&T>> for Option<T> {
    fn merge_with(&mut self, rhs: Option<&T>) {
        if let Some(v) = rhs {
            self.replace(v.clone());
        }
    }
}
