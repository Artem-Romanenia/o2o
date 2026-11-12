pub trait IntoExisting<T> {
    fn into_existing(self, other: &mut T);
}

/// The counterpart of [`IntoExisting`] for the type that carries the `o2o` instructions:
/// updates `self` in place from `other`; fields marked `#[ghost]` keep their current values.
pub trait FromExisting<T> {
    #[allow(clippy::wrong_self_convention)]
    fn from_existing(&mut self, other: T);
}

pub trait TryIntoExisting<T> {
    type Error;
    fn try_into_existing(self, other: &mut T) -> Result<(), Self::Error>;
}
