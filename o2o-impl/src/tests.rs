#![cfg(test)]

use std::io::Write;

use quote::quote;
use syn::DeriveInput;
use crate::expand::derive;

#[test]
fn test_test() {
    let q2 = quote! {
        #[map(EntityModel)]
        struct Entity {
            #[map(TupleEntityDto| 0)]
            parent_int: i32,
            #[parent]
            base: BaseEntity,
            #[parent]
            child: Child,
        }
    };

    let input: DeriveInput = syn::parse2(q2).unwrap();
    let output = derive(&input).unwrap();
    std::io::stdout().write_fmt(format_args!("{}\n\n", output)).unwrap();
}