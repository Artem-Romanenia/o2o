#![cfg(test)]

use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Error};
use crate::expand::derive;

// region: Debuger

// use std::io::Write;
// #[test]
// fn debuger() {
//     let code_fragment = quote!{
//     };

//     let input: DeriveInput = syn::parse2(code_fragment).unwrap();
//     let output = derive(&input);

//     match output {
//         Ok(output) => {
//             let text = output.to_string();
//             _ = std::io::stdout().write_all(format!("\nOutput:\n\n{}\n\n", text).as_ref());
//         },
//         Err(err) => {
//             let mut err_iter = err.into_iter();
//             let error = err_iter.next();
//             let message = error.expect("One error expected").to_string();
//             _ = std::io::stdout().write_all(format!("\nError:\n\n{}\n\n", message).as_ref());
//         }
//     }
// }

// endregion: Debuger

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
        let message = get_error(output, true);
    
        assert_eq!(message, "At least one trait instruction is expected.");
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
        let message = get_error(output, false);
    
        assert_eq!(message, format!("Struct instruction '{}' is not supported.", err));
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
        let message = get_error(output, false);
    
        assert_eq!(message, format!("Member instruction '{}' is not supported.", err));
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
        let message = get_error(output, true);
    
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
        let errors: Vec<Error> = get_error_iter(output).collect();

        for (ty, should_contain) in errs {
            match should_contain {
                true => assert!(errors.iter().any(|x| x.to_string() == format!("Missing #[children(...)] instruction for {}", ty))),
                false => assert!(!errors.iter().any(|x| x.to_string() == format!("Missing #[children(...)] instruction for {}", ty)))
            }
        }
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
        let errors: Vec<Error> = get_error_iter(output).collect();
    
        for (field, ty, should_contain) in errs {
            match should_contain {
                true => assert!(errors.iter().any(|x| x.to_string() == format!("Missing '{}: [Type Path]' instruction for type {}", field, ty))),
                false => assert!(!errors.iter().any(|x| x.to_string() == format!("Missing '{}: [Type Path]' instruction for type {}", field, ty)))
            }
        }
    }
}

#[test]
fn incomplete_ghost_instruction() {
    let test_data = vec![
        (quote! {
            #[into(Entity)]
            struct EntityDto {
                some_val: i32,
                #[ghost]
                another_val: i32,
            }
        }, vec![]),
        (quote! {
            #[from(Entity)]
            struct EntityDto {
                some_val: i32,
                #[ghost]
                another_val: i32,
            }
        }, vec![
            ("another_val", "Entity")
        ]),
        (quote! {
            #[from(Entity)]
            struct EntityDto {
                some_val: i32,
                #[ghost()]
                another_val: i32,
            }
        }, vec![
            ("another_val", "Entity")
        ]),
        (quote! {
            #[map(Entity)]
            struct EntityDto {
                some_val: i32,
                #[ghost]
                another_val: i32,
            }
        }, vec![
            ("another_val", "Entity")
        ]),
        (quote! {
            #[map(Entity)]
            struct EntityDto {
                some_val: i32,
                #[ghost()]
                another_val: i32,
            }
        }, vec![
            ("another_val", "Entity")
        ]),
        (quote! {
            #[from(Entity)]
            #[from(Entity2)]
            struct EntityDto {
                some_val: i32,
                #[ghost]
                another_val: i32,
                #[ghost(Entity2)]
                third_val: i32,
            }
        }, vec![
            ("another_val", "Entity"),
            ("another_val", "Entity2"),
            ("third_val", "Entity2")
        ])
    ];

    for (code_fragment, errs) in test_data {
        let input: DeriveInput = syn::parse2(code_fragment).unwrap();
        let output = derive(&input);

        if errs.len() > 0 {
            let errors: Vec<Error> = get_error_iter(output).collect();

            assert_eq!(errs.len(), errors.len());

            for (field, ty) in errs {
                assert!(errors.iter().any(|x| x.to_string() == format!("Member instruction #[ghost(...)] for member '{}' should provide default value for type {}", field, ty)))
            }
        } else {
            assert!(output.is_ok())
        }
    }
}

#[test]
fn incomplete_field_attr_instruction() {
    let test_data = vec![
        (quote! {
            #[into(Entity as {})]
            struct EntityDto (i32);
        }, vec![
            ("owned_into", "Entity", false),
            ("ref_into", "Entity", false)
        ]),
        (quote! {
            #[into(Entity as {})]
            struct EntityDto (#[from]i32);
        }, vec![
            ("owned_into", "Entity", false),
            ("ref_into", "Entity", false)
        ]),
        (quote! {
            #[into_existing(Entity as {})]
            struct EntityDto (i32);
        }, vec![
            ("owned_into_existing", "Entity", false),
            ("ref_into_existing", "Entity", false)
        ]),
        (quote! {
            #[from(Entity as {})]
            struct EntityDto (i32);
        }, vec![
            ("from_owned", "Entity", true),
            ("from_ref", "Entity", true)
        ]),
        (quote! {
            #[from(Entity as {})]
            struct EntityDto (#[into()]i32);
        }, vec![
            ("from_owned", "Entity", true),
            ("from_ref", "Entity", true)
        ]),
        (quote! {
            #[map(Entity as {})]
            #[map(Entity2 as {})]
            struct EntityDto (#[map(Entity2| test)]i32);
        }, vec![
            ("from_owned", "Entity", true),
            ("from_ref", "Entity", true),
            ("owned_into", "Entity", false),
            ("ref_into", "Entity", false)
        ]),
        (quote! {
            #[map(Entity as {})]
            #[from(Entity2 as {})]
            struct EntityDto (#[from(Entity2| {123})]i32);
        }, vec![
            ("from_owned", "Entity", true),
            ("from_ref", "Entity", true),
            ("owned_into", "Entity", false),
            ("ref_into", "Entity", false),
        ]),
        (quote! {
            #[map_owned(StuffWrapper as {})]
            #[o2o(wrapped(payload))]
            struct Stuff(i32);
        }, vec![]),
    ];

    for (code_fragment, errs) in test_data {
        let input: DeriveInput = syn::parse2(code_fragment).unwrap();
        let output = derive(&input);

        if errs.len() > 0 {
            let errors: Vec<Error> = get_error_iter(output).collect();

            assert_eq!(errs.len(), errors.len());

            for (field, ty, or_action) in errs {
                assert!(errors.iter().any(|x| x.to_string() == format!("Member 0 should have member trait instruction with field name{}, that corresponds to #[{}({}...)] trait instruction", if or_action {" or an action"} else {""}, field, ty)))
            }
        } else {
            assert!(output.is_ok())
        }
    }
}

#[test]
fn incomplete_field_attr_instruction_2() {
    let test_data = vec![
        (quote! {
            #[from(Entity as {})]
            struct EntityDto (#[from]i32);
        }, vec![
            ("from", "Entity", true)
        ]),
        (quote! {
            #[from(Entity as {})]
            struct EntityDto (
                #[from]i32,
                #[parent]i16
            );
        }, vec![
            ("from", "Entity", true)
        ]),
        (quote! {
            #[into(Entity as {})]
            struct EntityDto (#[into]i32);
        }, vec![
            ("into", "Entity", false)
        ]),
        (quote! {
            #[into(Entity as {})]
            struct EntityDto (
                #[into]i32,
                #[parent]i16,
                #[ghost]i8
            );
        }, vec![
            ("into", "Entity", false)
        ]),
        (quote! {
            #[into_existing(Entity as {})]
            struct EntityDto (#[into_existing]i32);
        }, vec![
            ("into_existing", "Entity", false)
        ]),
        (quote! {
            #[into_existing(Entity as {})]
            struct EntityDto (
                #[into_existing]i32,
                #[parent]i16,
                #[ghost]i8
            );
        }, vec![
            ("into_existing", "Entity", false)
        ]),
        (quote! {
            #[into_existing(Entity as {})]
            struct EntityDto (
                #[owned_into_existing(test)]
                #[ref_into_existing]
                i32
            );
        }, vec![
            ("ref_into_existing", "Entity", false)
        ]),
        (quote! {
            #[into_existing(Entity as {})]
            struct EntityDto (
                #[owned_into(test)]
                #[ref_into]
                i32
            );
        }, vec![
            ("ref_into", "Entity", false)
        ]),
    ];

    for (code_fragment, errs) in test_data {
        let input: DeriveInput = syn::parse2(code_fragment).unwrap();
        let output = derive(&input);

        if errs.len() > 0 {
            let errors: Vec<Error> = get_error_iter(output).collect();

            assert_eq!(errs.len(), errors.len());

            for (field, ty, or_action) in errs {
                assert!(errors.iter().any(|x| x.to_string() == format!("Member trait instruction #[{}(...)] for member 0 should specify corresponding field name of the {}{}", field, ty, if or_action {" or an action"} else {""})))
            }
        } else {
            assert!(output.is_ok())
        }
    }
}

fn get_error(output: Result<TokenStream, Error>, expect_root_error: bool) -> String {
    assert!(output.is_err());
    let mut err_iter = output.unwrap_err().into_iter();

    if expect_root_error {
        let error = err_iter.next();
        assert_eq!(error.expect("Root error expected").to_string(), "Cannot expand o2o macro");
    }

    let error = err_iter.next();
    let message = error.expect("Two errors expected").to_string();
    assert!(err_iter.next().is_none());

    message
}

fn get_error_iter(output: Result<TokenStream, Error>) -> impl Iterator<Item = Error> {
    assert!(output.is_err());
    let mut err_iter = output.unwrap_err().into_iter();

    let error = err_iter.next();
    assert!(error.expect("Root error expected").to_string() == "Cannot expand o2o macro");

    err_iter
}