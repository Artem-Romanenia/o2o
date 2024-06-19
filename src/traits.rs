pub trait IntoExisting<T> {
    fn into_existing(self, other: &mut T);
}

pub trait TryIntoExisting<T> {
    type Error;
    fn try_into_existing(self, other: &mut T) -> Result<(), Self::Error>;
}