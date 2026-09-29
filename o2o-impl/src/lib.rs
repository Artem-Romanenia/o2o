use std::fmt::Formatter;

use quote::ToTokens;

#[cfg(all(feature = "syn", feature = "syn2"))]
compile_error!("Features 'syn' and 'syn2' cannot be enabled at the same time");

#[cfg(not(all(feature = "syn", feature = "syn2")))]
mod ast;
#[cfg(not(all(feature = "syn", feature = "syn2")))]
mod model;
#[cfg(not(all(feature = "syn", feature = "syn2")))]
pub mod expand;
#[cfg(not(all(feature = "syn", feature = "syn2")))]
mod kw;
#[cfg(not(all(feature = "syn", feature = "syn2")))]
mod validate;
#[cfg(not(all(feature = "syn", feature = "syn2")))]
mod render;


mod tests;


fn debug_to_tokens<'a>(g: &'a dyn ToTokens, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
    fmt.write_fmt(format_args!("{0}", g.to_token_stream().to_string()))?;
    Ok(())
}
