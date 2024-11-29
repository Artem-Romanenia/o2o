#[cfg(all(feature = "syn", feature = "syn2"))]
compile_error!("Features 'syn' and 'syn2' cannot be enabled at the same time");

#[cfg(not(all(feature = "syn", feature = "syn2")))]
mod ast;
#[cfg(not(all(feature = "syn", feature = "syn2")))]
mod attr;
#[cfg(not(all(feature = "syn", feature = "syn2")))]
pub mod expand;
#[cfg(not(all(feature = "syn", feature = "syn2")))]
mod kw;
#[cfg(not(all(feature = "syn", feature = "syn2")))]
mod validate;

mod tests;