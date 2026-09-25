mod types;
mod members;
mod data_type_attrs;
mod member_attrs;
mod aux;

#[cfg(feature = "syn2")]
use syn2 as syn;

pub(super) use syn::{Attribute, Member, Ident, Index, token::{Brace, Paren, Bracket, Comma}, WherePredicate, AngleBracketedGenericArguments, PathArguments, Generics, DeriveInput, DataStruct, Fields, DataEnum, Path, Token, parse::{Parse, ParseStream, ParseBuffer}, punctuated::Punctuated, spanned::Spanned, Result, Error, parenthesized, braced, bracketed};
pub(super) use proc_macro2::{Span, TokenStream, TokenTree};
pub(super) use quote::{ToTokens, quote};

pub(crate) use types::*;
pub(crate) use members::*;
pub(crate) use data_type_attrs::*;
pub(crate) use member_attrs::*;
pub(crate) use aux::*;