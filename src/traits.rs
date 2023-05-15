pub trait IntoExisting<T> {
    fn into_existing(self, other: &mut T) ;
}