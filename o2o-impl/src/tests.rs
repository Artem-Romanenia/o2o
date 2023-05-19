#![cfg(test)]

use std::io::Write;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Error};
use crate::expand::derive;

#[test]
fn debuger() {
    let code_fragment = quote!{
        #[derive(o2o)]
#[from(TupleEntityDto)]
#[from(EntityDto)]
#[into_existing(TupleEntityDto)]
#[into_existing(EntityDto)]
#[panic_debug_info]
struct TupleBaseEntity(
    #[parent]
    TupleBase,
    #[map(TupleEntityDto| 3)]
    #[map(EntityDto| base_entity_int)]
    i32
);
    };

    let input: DeriveInput = syn::parse2(code_fragment).unwrap();
    let output = derive(&input);

    match output {
        Ok(output) => {
            let text = output.to_string();
            _ = std::io::stdout().write_all(format!("\nOutput:\n\n{}\n\n", text).as_ref());
        },
        Err(err) => {
            let mut err_iter = err.into_iter();
            let error = err_iter.next();
            let message = error.expect("One error expected").to_string();
            _ = std::io::stdout().write_all(format!("\nError:\n\n{}\n\n", message).as_ref());
        }
    }
}

#[test]
fn missing_map_instructions() {
    let code_fragments = vec![
        quote! {
            struct Entity {}
        },
        quote! {
            #[ghost(field: |_| { 123 })]
            struct Entity {}
        },
        quote! {
            struct Entity {
                #[map(diff_field)]
                field: i32,
            }
        },
    ];

    for code_fragment in code_fragments {
        let input: DeriveInput = syn::parse2(code_fragment).unwrap();
        let output = derive(&input);
        let message = get_error(output);
    
        assert_eq!(message, "At least one 'map'-like struct level instruction is expected.");
    }
}

#[test]
fn unrecognized_struct_instructions() {
    let test_data = vec![
        (quote! {
            #[o2o(mapp(EntityDto))]
            struct Entity {}
        }, "mapp"),
        (quote! {
            #[o2o(map(EntityDto))]
            #[o2o(parent(EntityDto))]
            struct Entity {}
        }, "parent"),
        (quote! {
            #[o2o(map(EntityDto))]
            #[o2o(child(EntityDto))]
            struct Entity {}
        }, "child"),
    ];

    for (code_fragment, err) in test_data {
        let input: DeriveInput = syn::parse2(code_fragment).unwrap();
        let output = derive(&input);
        let message = get_error(output);
    
        assert_eq!(message, format!("Struct level instruction '{}' is not supported.", err));
    }
}

#[test]
fn unrecognized_member_instructions() {
    let test_data = vec![
        (quote! {
            #[map(EntityDto)]
            struct Entity {
                #[o2o(mapp(diff_field))]
                child: i32,
            }
        }, "mapp"),
        (quote! {
            #[map(EntityDto)]
            struct Entity {
                #[o2o(children(diff_field))]
                child: i32,
            }
        }, "children"),
        (quote! {
            #[map(EntityDto)]
            struct Entity {
                #[o2o(where_clause(diff_field))]
                child: i32,
            }
        }, "where_clause"),
    ];

    for (code_fragment, err) in test_data {
        let input: DeriveInput = syn::parse2(code_fragment).unwrap();
        let output = derive(&input);
        let message = get_error(output);
    
        assert_eq!(message, format!("Member level instruction '{}' is not supported.", err));
    }
}

#[test]
fn unrecognized_struct_instructions_no_bark() {
    let code_fragment = quote!{
        #[from_owned(NamedStruct)]
        #[unknown()]
        struct NamedStructDto {}
    };

    let input: DeriveInput = syn::parse2(code_fragment).unwrap();
    let output = derive(&input);

    assert!(output.is_ok());
}

#[test]
fn unrecognized_member_instructions_no_bark() {
    let code_fragment = quote!{
        #[from_owned(NamedStruct)]
        struct NamedStructDto {
            #[unknown()]
            field: i32,
        }
    };

    let input: DeriveInput = syn::parse2(code_fragment).unwrap();
    let output = derive(&input);

    assert!(output.is_ok());
}

#[test]
fn more_than_one_default_instruction() {
    let test_data = vec![
        (quote! {
            #[map(EntityDto)]
            #[children(test: Test)]
            #[children(test: Test)]
            struct Entity {}
        }, "children"),
        (quote! {
            #[map(EntityDto)]
            #[where_clause(T: Clone)]
            #[where_clause(T: Clone)]
            struct Entity {}
        }, "where_clause"),
        (quote! {
            #[map(EntityDto)]
            #[ghost(field: |_| { Clone })]
            #[ghost(field: |_| { Clone })]
            struct Entity {}
        }, "ghost"),
    ];

    for (code_fragment, err) in test_data {
        let input: DeriveInput = syn::parse2(code_fragment).unwrap();
        let output = derive(&input);
        let message = get_error(output);
    
        assert_eq!(message, format!("There can be at most one default #[{}(...)] instruction.", err));
    }
}

#[test]
fn missing_children_instruction() {
    let test_data = vec![
        (quote! {
            #[map(EntityDto)]
            #[map(EntityModel)]
            struct Entity {
                #[child(base.base)]
                base_base_int: i32,
                #[child(child)]
                child_int: i32,
            }
        }, vec![ 
            ("EntityDto", true),
            ("EntityModel", true) 
        ]),
        (quote! {
            #[map(EntityDto)]
            #[map(EntityModel)]
            #[children()]
            struct Entity {
                #[child(base.base)]
                base_base_int: i32,
                #[child(child)]
                child_int: i32,
            }
        }, vec![ 
            ("EntityDto", false),
            ("EntityModel", false) 
        ]),
        (quote! {
            #[map(EntityDto)]
            #[map(EntityModel)]
            #[children(EntityDto| base: Base)]
            struct Entity {
                #[child(base.base)]
                base_base_int: i32,
                #[child(child)]
                child_int: i32,
            }
        }, vec![ 
            ("EntityDto", false),
            ("EntityModel", true) 
        ]),
    ];

    for (code_fragment, errs) in test_data {
        let input: DeriveInput = syn::parse2(code_fragment).unwrap();
        let output = derive(&input);
        let message = get_error(output);

        for (ty, should_contain) in errs {
            match should_contain {
                true => assert!(message.contains(&*format!("Missing #[children(...)] instruction for {}", ty))),
                false => assert!(!message.contains(&*format!("Missing #[children(...)] instruction for {}", ty)))
            }
        }
        
        assert!(message.contains(""));
    }
}

#[test]
fn incomplete_children_instruction()  {
    let test_data = vec![
        (quote! {
            #[map(EntityDto)]
            #[map(EntityModel)]
            #[children()]
            struct Entity {
                #[child(base.base)]
                base_base_int: i32,
                #[child(child)]
                child_int: i32,
            }
        }, vec![
            ("base", "EntityDto", true),
            ("base", "EntityModel", true),
            ("base.base", "EntityDto", true),
            ("base.base", "EntityModel", true),
            ("child", "EntityDto", true),
            ("child", "EntityModel", true),
        ]),
        (quote! {
            #[map(EntityDto)]
            #[map(EntityModel)]
            #[children(base: Base)]
            struct Entity {
                #[child(base.base)]
                base_base_int: i32,
                #[child(child)]
                child_int: i32,
            }
        }, vec![
            ("base", "EntityDto", false),
            ("base", "EntityModel", false),
            ("base.base", "EntityDto", true),
            ("base.base", "EntityModel", true),
            ("child", "EntityDto", true),
            ("child", "EntityModel", true),
        ]),
        (quote! {
            #[map(EntityDto)]
            #[map(EntityModel)]
            #[children(EntityDto| base: Base)]
            #[children(EntityModel| child: Child)]
            struct Entity {
                #[child(base.base)]
                base_base_int: i32,
                #[child(child)]
                child_int: i32,
            }
        }, vec![
            ("base", "EntityDto", false),
            ("base", "EntityModel", true),
            ("base.base", "EntityDto", true),
            ("base.base", "EntityModel", true),
            ("child", "EntityDto", true),
            ("child", "EntityModel", false),
        ]),
    ];

    for (code_fragment, errs) in test_data {
        let input: DeriveInput = syn::parse2(code_fragment).unwrap();
        let output = derive(&input);
        let message = get_error(output);
    
        for (field, ty, should_contain) in errs {
            match should_contain {
                true => assert!(message.contains(&*format!("Missing '{}: [Type Path]' instruction for type {}", field, ty))),
                false => assert!(!message.contains(&*format!("Missing '{}: [Type Path]' instruction for type {}", field, ty)))
            }
        }
    }
}

fn get_error(output: Result<TokenStream, Error>) -> String {
    assert!(output.is_err());
    let mut err_iter = output.unwrap_err().into_iter();
    let error = err_iter.next();
    let message = error.expect("One error expected").to_string();
    assert!(err_iter.next().is_none());

    message
}