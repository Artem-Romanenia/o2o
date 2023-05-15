extern crate o2o_impl;

use proc_macro::TokenStream;
use o2o_impl::expand::derive;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(o2o, attributes(
    owned_into, ref_into, into, 
    from_owned, from_ref, from, 
    map_owned, map_ref, map, 
    owned_into_existing, ref_into_existing, into_existing,
    child, parent, ghost, panic_debug_info, where_clause, o2o))]
pub fn derive_o2o(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive(&input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}