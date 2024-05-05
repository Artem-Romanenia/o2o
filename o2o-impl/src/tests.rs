#![cfg(test)]

use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Error};
use crate::expand::derive;
use test_case::test_case;

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

#[test_case(quote! { struct Entity {} }; "bare_struct")]
#[test_case(quote! { enum Enum {} }; "bare_enum")]
#[test_case(quote! {
    #[ghosts(field: { 123 })]
    struct Entity {}
}; "struct_only_ghosts_instr")]
#[test_case(quote! {
    #[ghosts(field: { 123 })]
    enum Enum {}
}; "enum_only_ghosts_instr")]
#[test_case(quote! {
    struct Entity {
        #[map(diff_field)]
        field: i32,
    }
}; "struct_only_member_instr")]
#[test_case(quote! {
    enum Enum {
        #[map(DiffVariant)]
        Variant
    }
}; "enum_only_member_instr")]
fn missing_map_instructions(code_fragment: TokenStream) {
    let input: DeriveInput = syn::parse2(code_fragment).unwrap();
    let output = derive(&input);
    let message = get_error(output, true);

    assert_eq!(message, "At least one trait instruction is expected.");
}

#[test_case(quote! {
    #[map(EntityDto)]
    #[parent(EntityDto)]
    struct Entity {}
}, vec![ "Member instruction 'parent' should be used on a member. To turn this message off, use #[o2o(allow_unknown)]" ]; "struct_misplaced_parent_instr")]
#[test_case(quote! {
    #[map(EnumDto)]
    #[parent(EnumDto)]
    enum Enum {}
}, vec![ "Member instruction 'parent' is not applicable to enums. To turn this message off, use #[o2o(allow_unknown)]" ]; "enum_misplaced_parent_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    #[child(EntityDto)]
    struct Entity {}
}, vec![ "Perhaps you meant 'children'? To turn this message off, use #[o2o(allow_unknown)]" ]; "struct_misnamed_child_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    #[child(EntityDto)]
    enum Enum {}
}, vec![ "Member instruction 'child' is not applicable to enums. To turn this message off, use #[o2o(allow_unknown)]" ]; "enum_misplaced_child_instr")]
#[test_case(quote! {
    #[map(i32)]
    #[as_type(i32)]
    struct Entity {}
}, vec![ "Member instruction 'as_type' should be used on a member. To turn this message off, use #[o2o(allow_unknown)]" ]; "struct_misplaced_as_type_instr")]
#[test_case(quote! {
    #[map(i32)]
    #[as_type(i32)]
    enum Enum {}
}, vec![ "Member instruction 'as_type' is not applicable to enums. To turn this message off, use #[o2o(allow_unknown)]" ]; "enum_misplaced_as_type_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    #[repeat(EntityDto)]
    struct Entity {}
}, vec![ "Member instruction 'repeat' should be used on a member. To turn this message off, use #[o2o(allow_unknown)]" ]; "struct_misplaced_repeat_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    #[repeat(EntityDto)]
    enum Enum {}
}, vec![ "Member instruction 'repeat' should be used on a member. To turn this message off, use #[o2o(allow_unknown)]" ]; "enum_misplaced_repeat_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    #[stop_repeat(EntityDto)]
    struct Entity {}
}, vec![ "Member instruction 'stop_repeat' should be used on a member. To turn this message off, use #[o2o(allow_unknown)]" ]; "struct_misplaced_stop_repeat_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    #[stop_repeat(EntityDto)]
    enum Enum {}
}, vec![ "Member instruction 'stop_repeat' should be used on a member. To turn this message off, use #[o2o(allow_unknown)]" ]; "enum_misplaced_stop_repeat_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    #[ghost(EntityDto)]
    struct Entity {}
}, vec![ "Perhaps you meant 'ghosts'? To turn this message off, use #[o2o(allow_unknown)]" ]; "struct_misnamed_ghost_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    #[ghost(EntityDto)]
    enum Enum {}
}, vec![ "Perhaps you meant 'ghosts'? To turn this message off, use #[o2o(allow_unknown)]" ]; "enum_misnamed_ghost_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    #[ghost_ref(EntityDto)]
    struct Entity {}
}, vec![ "Perhaps you meant 'ghosts_ref'? To turn this message off, use #[o2o(allow_unknown)]" ]; "struct_misnamed_ghost_ref_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    #[ghost_ref(EntityDto)]
    enum Enum {}
}, vec![ "Perhaps you meant 'ghosts_ref'? To turn this message off, use #[o2o(allow_unknown)]" ]; "enum_misnamed_ghost_ref_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    #[ghost_owned(EntityDto)]
    struct Entity {}
}, vec![ "Perhaps you meant 'ghosts_owned'? To turn this message off, use #[o2o(allow_unknown)]" ]; "struct_misnamed_ghost_owned_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    #[ghost_owned(EntityDto)]
    enum Enum {}
}, vec![ "Perhaps you meant 'ghosts_owned'? To turn this message off, use #[o2o(allow_unknown)]" ]; "enum_misnamed_ghost_owned_instr")]
#[test_case(quote! {
    #[o2o(mapp(EntityDto))]
    struct Entity {}
}, vec![ "Struct instruction 'mapp' is not supported.", "At least one trait instruction is expected." ]; "struct_unrecognized_instr")]
#[test_case(quote! {
    #[o2o(mapp(EntityDto))]
    enum Enum {}
}, vec![ "Struct instruction 'mapp' is not supported.", "At least one trait instruction is expected." ]; "enum_unrecognized_instr")]
#[test_case(quote! {
    #[o2o(map(EntityDto))]
    #[o2o(parent(EntityDto))]
    struct Entity {}
}, vec![ "Member instruction 'parent' should be used on a member." ]; "own_struct_misplaced_parent_instr")]
#[test_case(quote! {
    #[o2o(map(EntityDto))]
    #[o2o(parent(EntityDto))]
    enum Enum {}
}, vec![ "Member instruction 'parent' is not applicable to enums." ]; "own_enum_misplaced_parent_instr")]
#[test_case(quote! {
    #[o2o(map(EntityDto))]
    #[o2o(child(EntityDto))]
    struct Entity {}
}, vec![ "Perhaps you meant 'children'?" ]; "own_struct_misnamed_child_instr")]
#[test_case(quote! {
    #[o2o(map(EntityDto))]
    #[o2o(child(EntityDto))]
    enum Enum {}
}, vec![ "Member instruction 'child' is not applicable to enums." ]; "own_enum_misnamed_child_instr")]
#[test_case(quote! {
    #[map(i32)]
    #[o2o(as_type(i32))]
    struct Entity {}
}, vec![ "Member instruction 'as_type' should be used on a member." ]; "own_struct_misplaced_as_type_instr")]
#[test_case(quote! {
    #[map(i32)]
    #[o2o(as_type(i32))]
    enum Enum {}
}, vec![ "Member instruction 'as_type' is not applicable to enums." ]; "own_enum_misplaced_as_type_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    #[o2o(repeat(EntityDto))]
    struct Entity {}
}, vec![ "Member instruction 'repeat' should be used on a member." ]; "own_struct_misplaced_repeat_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    #[o2o(repeat(EntityDto))]
    enum Enum {}
}, vec![ "Member instruction 'repeat' should be used on a member." ]; "own_enum_misplaced_repeat_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    #[o2o(stop_repeat(EntityDto))]
    struct Entity {}
}, vec![ "Member instruction 'stop_repeat' should be used on a member." ]; "own_struct_misplaced_stop_repeat_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    #[o2o(stop_repeat(EntityDto))]
    enum Enum {}
}, vec![ "Member instruction 'stop_repeat' should be used on a member." ]; "own_enum_misplaced_stop_repeat_instr")]
#[test_case(quote! {
    #[o2o(map(EntityDto))]
    #[o2o(ghost(EntityDto))]
    struct Entity {}
}, vec![ "Perhaps you meant 'ghosts'?" ]; "own_struct_misnamed_ghost_instr")]
#[test_case(quote! {
    #[o2o(map(EntityDto))]
    #[o2o(ghost(EntityDto))]
    enum Enum {}
}, vec![ "Perhaps you meant 'ghosts'?" ]; "own_enum_misnamed_ghost_instr")]
#[test_case(quote! {
    #[o2o(map(EntityDto))]
    #[o2o(ghost_ref(EntityDto))]
    struct Entity {}
}, vec![ "Perhaps you meant 'ghosts_ref'?" ]; "own_struct_misnamed_ghost_ref_instr")]
#[test_case(quote! {
    #[o2o(map(EntityDto))]
    #[o2o(ghost_ref(EntityDto))]
    enum Enum {}
}, vec![ "Perhaps you meant 'ghosts_ref'?" ]; "own_enum_misnamed_ghost_ref_instr")]
#[test_case(quote! {
    #[o2o(map(EntityDto))]
    #[o2o(ghost_owned(EntityDto))]
    struct Entity {}
}, vec![ "Perhaps you meant 'ghosts_owned'?" ]; "own_struct_misnamed_ghost_owned_instr")]
#[test_case(quote! {
    #[o2o(map(EntityDto))]
    #[o2o(ghost_owned(EntityDto))]
    enum Enum {}
}, vec![ "Perhaps you meant 'ghosts_owned'?" ]; "own_enum_misnamed_ghost_owned_instr")]
fn unrecognized_struct_instructions(code_fragment: TokenStream, errs: Vec<&str>) {
    let input: DeriveInput = syn::parse2(code_fragment).unwrap();
    let output = derive(&input);
    let errors: Vec<Error> = get_error_iter(output).collect();

    assert_eq!(errs.len(), errors.len());

    for err in errs {
        assert!(errors.iter().any(|x| x.to_string() == err))
    }
}

#[test_case(quote! {
    #[map(EntityDto)]
    struct Entity {
        #[ghosts()]
        child: i32,
    }
}, "Perhaps you meant 'ghost'? To turn this message off, use #[o2o(allow_unknown)]"; "struct_misplaced_ghosts_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    struct Entity {
        #[ghosts_owned()]
        child: i32,
    }
}, "Perhaps you meant 'ghost_owned'? To turn this message off, use #[o2o(allow_unknown)]"; "struct_misplaced_ghosts_owned_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    struct Entity {
        #[ghosts_ref()]
        child: i32,
    }
}, "Perhaps you meant 'ghost_ref'? To turn this message off, use #[o2o(allow_unknown)]"; "struct_misplaced_ghosts_ref_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    struct Entity {
        #[children()]
        child: i32,
    }
}, "Perhaps you meant 'child'? To turn this message off, use #[o2o(allow_unknown)]"; "struct_misplaced_children_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    struct Entity {
        #[where_clause()]
        child: i32,
    }
}, "Struct instruction 'where_clause' should be used on a struct. To turn this message off, use #[o2o(allow_unknown)]"; "struct_misplaced_where_clause_instr")]
#[test_case(quote! {
    #[map(EnumDto)]
    enum Enum {
        #[where_clause()]
        Variant,
    }
}, "Struct instruction 'where_clause' should be used on a struct. To turn this message off, use #[o2o(allow_unknown)]"; "enum_misplaced_where_clause_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    struct Entity {
        #[o2o(mapp(diff_field))]
        child: i32,
    }
}, "Member instruction 'mapp' is not supported."; "struct_unrecognized_instr")]
#[test_case(quote! {
    #[map(EnumDto)]
    enum Enum {
        #[o2o(mapp(DiffVar))]
        Variant,
    }
}, "Member instruction 'mapp' is not supported."; "enum_unrecognized_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    struct Entity {
        #[o2o(ghosts())]
        child: i32,
    }
}, "Perhaps you meant 'ghost'?"; "own_struct_misplaced_ghosts_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    struct Entity {
        #[o2o(ghosts_owned())]
        child: i32,
    }
}, "Perhaps you meant 'ghost_owned'?"; "own_struct_misplaced_ghosts_owned_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    struct Entity {
        #[o2o(ghosts_ref())]
        child: i32,
    }
}, "Perhaps you meant 'ghost_ref'?"; "own_struct_misplaced_ghosts_ref_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    struct Entity {
        #[o2o(children())]
        child: i32,
    }
}, "Perhaps you meant 'child'?"; "own_struct_misplaced_children_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    struct Entity {
        #[o2o(where_clause())]
        child: i32,
    }
}, "Struct instruction 'where_clause' should be used on a struct."; "own_struct_misplaced_where_clause_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    enum Enum {
        #[o2o(where_clause())]
        Variant,
    }
}, "Struct instruction 'where_clause' should be used on a struct."; "own_enum_misplaced_where_clause_instr")]
fn unrecognized_member_instructions(code_fragment: TokenStream, err: &str) {
    let input: DeriveInput = syn::parse2(code_fragment).unwrap();
    let output = derive(&input);
    let message = get_error(output, true);

    assert_eq!(message, err);
}

#[test_case(quote! {
    #[from_owned(NamedStruct)]
    #[mapp(EntityDto)]
    struct Entity {}
}; "struct_unrecognized_instr")]
#[test_case(quote! {
    #[from_owned(EnumDto)]
    #[mapp(EnumDto)]
    enum Enum {}
}; "enum_unrecognized_instr")]
#[test_case(quote! {
    #[o2o(allow_unknown)]
    #[map(EntityDto)]
    #[parent(EntityDto)]
    struct Entity {}
}; "struct_misplaced_parent_instr")]
#[test_case(quote! {
    #[o2o(allow_unknown)]
    #[map(EnumDto)]
    #[parent(EnumDto)]
    enum Enum {}
}; "enum_misplaced_parent_instr")]
#[test_case(quote! {
    #[o2o(allow_unknown)]
    #[map(EntityDto)]
    #[child(EntityDto)]
    struct Entity {}
}; "struct_misplaced_child_instr")]
#[test_case(quote! {
    #[o2o(allow_unknown)]
    #[map(EnumDto)]
    #[child(EnumDto)]
    enum Enum {}
}; "enum_misplaced_child_instr")]
#[test_case(quote! {
    #[o2o(allow_unknown)]
    #[map(EntityDto)]
    #[ghost(EntityDto)]
    struct Entity {}
}; "struct_misplaced_ghost_instr")]
#[test_case(quote! {
    #[o2o(allow_unknown)]
    #[map(EnumDto)]
    #[ghost(EnumDto)]
    enum Enum {}
}; "enum_misplaced_ghost_instr")]
fn unrecognized_struct_instructions_no_bark(code_fragment: TokenStream) {
    let input: DeriveInput = syn::parse2(code_fragment).unwrap();
    let output = derive(&input);

    assert!(output.is_ok());
}

#[test_case(quote! {
    #[from_owned(NamedStruct)]
    struct NamedStructDto {
        #[unknown()]
        field: i32,
    }
}; "struct_unknown_instr")]
#[test_case(quote! {
    #[from_owned(EnumDto)]
    enum Enum {
        #[unknown()]
        Variant,
    }
}; "enum_unknown_instr")]
#[test_case(quote!{
    #[from_owned(NamedStruct)]
    #[o2o(allow_unknown)]
    struct NamedStructDto {
        #[ghosts()]
        field: i32,
    }
}; "struct_misplaced_ghosts_instr")]
#[test_case(quote!{
    #[from_owned(NamedStruct)]
    #[o2o(allow_unknown)]
    struct NamedStructDto {
        #[ghosts_owned()]
        field: i32,
    }
}; "struct_misplaced_ghosts_owned_instr")]
#[test_case(quote!{
    #[from_owned(NamedStruct)]
    #[o2o(allow_unknown)]
    struct NamedStructDto {
        #[ghosts_ref()]
        field: i32,
    }
}; "struct_misplaced_ghosts_ref_instr")]
#[test_case(quote!{
    #[from_owned(NamedStruct)]
    #[o2o(allow_unknown)]
    struct NamedStructDto {
        #[children()]
        field: i32,
    }
}; "struct_misplaced_children_instr")]
#[test_case(quote!{
    #[from_owned(NamedStruct)]
    #[o2o(allow_unknown)]
    struct NamedStructDto {
        #[where_clause()]
        field: i32,
    }
}; "struct_misplaced_where_clause_instr")]
#[test_case(quote!{
    #[from_owned(EnumDto)]
    #[o2o(allow_unknown)]
    enum Enum {
        #[where_clause()]
        Variant,
    }
}; "enum_misplaced_where_clause_instr")]
fn unrecognized_member_instructions_no_bark(code_fragment: TokenStream) {
    let input: DeriveInput = syn::parse2(code_fragment).unwrap();
    let output = derive(&input);

    assert!(output.is_ok());
}

#[test_case(quote! {
    #[map(EntityDto)]
    #[children(test: Test)]
    #[children(test: Test)]
    struct Entity {}
}, "children"; "struct_children_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    #[where_clause(T: Clone)]
    #[where_clause(T: Clone)]
    struct Entity {}
}, "where_clause"; "struct_where_clause_instr")]
#[test_case(quote! {
    #[map(EnumDto)]
    #[where_clause(T: Clone)]
    #[where_clause(T: Clone)]
    enum Enun {}
}, "where_clause"; "enum_where_clause_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    #[ghosts(field: { Clone })]
    #[ghosts(field: { Clone })]
    struct Entity {}
}, "ghosts"; "struct_ghosts_instr")]
fn more_than_one_default_instruction(code_fragment: TokenStream, err: &str) {
    let input: DeriveInput = syn::parse2(code_fragment).unwrap();
    let output = derive(&input);
    let message = get_error(output, true);

    assert_eq!(message, format!("There can be at most one default #[{}(...)] instruction.", err));
}

#[test_case(quote! {
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
]; "1")]
#[test_case(quote! {
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
]; "2")]
#[test_case(quote! {
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
]; "3")]
fn missing_children_instruction(code_fragment: TokenStream, errs: Vec<(&str, bool)>) {
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

#[test_case(quote! {
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
]; "1")]
#[test_case(quote! {
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
]; "2")]
#[test_case(quote! {
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
]; "3")]
fn incomplete_children_instruction(code_fragment: TokenStream, errs: Vec<(&str, &str, bool)>) {
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

#[test_case(quote! {
    #[into(Entity)]
    struct EntityDto {
        some_val: i32,
        #[ghost]
        another_val: i32,
    }
}, vec![]; "1")]
#[test_case(quote! {
    #[from(Entity)]
    struct EntityDto {
        some_val: i32,
        #[ghost]
        another_val: i32,
    }
}, vec![
    ("another_val", "Entity")
]; "2")]
#[test_case(quote! {
    #[from(Entity)]
    struct EntityDto {
        some_val: i32,
        #[ghost()]
        another_val: i32,
    }
}, vec![
    ("another_val", "Entity")
]; "3")]
#[test_case(quote! {
    #[map(Entity)]
    struct EntityDto {
        some_val: i32,
        #[ghost]
        another_val: i32,
    }
}, vec![
    ("another_val", "Entity")
]; "4")]
#[test_case(quote! {
    #[map(Entity)]
    struct EntityDto {
        some_val: i32,
        #[ghost()]
        another_val: i32,
    }
}, vec![
    ("another_val", "Entity")
]; "5")]
#[test_case(quote! {
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
]; "6")]
fn incomplete_ghost_instruction(code_fragment: TokenStream, errs: Vec<(&str, &str)>) {
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

#[test_case(quote! {
    #[into(Entity as {})]
    struct EntityDto (i32);
}, vec![
    ("owned_into", "Entity", false),
    ("ref_into", "Entity", false)
]; "1")]
#[test_case(quote! {
    #[into(Entity as {})]
    struct EntityDto (#[from]i32);
}, vec![
    ("owned_into", "Entity", false),
    ("ref_into", "Entity", false)
]; "2")]
#[test_case(quote! {
    #[into_existing(Entity as {})]
    struct EntityDto (i32);
}, vec![
    ("owned_into_existing", "Entity", false),
    ("ref_into_existing", "Entity", false)
]; "3")]
#[test_case(quote! {
    #[from(Entity as {})]
    struct EntityDto (i32);
}, vec![
    ("from_owned", "Entity", true),
    ("from_ref", "Entity", true)
]; "4")]
#[test_case(quote! {
    #[from(Entity as {})]
    struct EntityDto (#[into()]i32);
}, vec![
    ("from_owned", "Entity", true),
    ("from_ref", "Entity", true)
]; "5")]
#[test_case(quote! {
    #[map(Entity as {})]
    #[map(Entity2 as {})]
    struct EntityDto (#[map(Entity2| test)]i32);
}, vec![
    ("from_owned", "Entity", true),
    ("from_ref", "Entity", true),
    ("owned_into", "Entity", false),
    ("ref_into", "Entity", false)
]; "6")]
#[test_case(quote! {
    #[map(Entity as {})]
    #[from(Entity2 as {})]
    struct EntityDto (#[from(Entity2| {123})]i32);
}, vec![
    ("from_owned", "Entity", true),
    ("from_ref", "Entity", true),
    ("owned_into", "Entity", false),
    ("ref_into", "Entity", false),
]; "7")]
#[test_case(quote! {
    #[owned_into(StuffWrapper| return StuffWrapper { payload: @ })]
    #[from_owned(StuffWrapper| return @.payload)]
    struct Stuff(i32);
}, vec![]; "8")]
fn incomplete_field_attr_instruction(code_fragment: TokenStream, errs: Vec<(&str, &str, bool)>) {
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

#[test_case(quote! {
    #[from(Entity as {})]
    struct EntityDto (#[from]i32);
}, vec![
    ("from", "Entity", true)
]; "1")]
#[test_case(quote! {
    #[from(Entity as {})]
    struct EntityDto (
        #[from]i32,
        #[parent]i16
    );
}, vec![
    ("from", "Entity", true)
]; "2")]
#[test_case(quote! {
    #[into(Entity as {})]
    struct EntityDto (#[into]i32);
}, vec![
    ("into", "Entity", false)
]; "3")]
#[test_case(quote! {
    #[into(Entity as {})]
    struct EntityDto (
        #[into]i32,
        #[parent]i16,
        #[ghost]i8
    );
}, vec![
    ("into", "Entity", false)
]; "4")]
#[test_case(quote! {
    #[into_existing(Entity as {})]
    struct EntityDto (#[into_existing]i32);
}, vec![
    ("into_existing", "Entity", false)
]; "5")]
#[test_case(quote! {
    #[into_existing(Entity as {})]
    struct EntityDto (
        #[into_existing]i32,
        #[parent]i16,
        #[ghost]i8
    );
}, vec![
    ("into_existing", "Entity", false)
]; "6")]
#[test_case(quote! {
    #[into_existing(Entity as {})]
    struct EntityDto (
        #[owned_into_existing(test)]
        #[ref_into_existing]
        i32
    );
}, vec![
    ("ref_into_existing", "Entity", false)
]; "7")]
#[test_case(quote! {
    #[into_existing(Entity as {})]
    struct EntityDto (
        #[owned_into(test)]
        #[ref_into]
        i32
    );
}, vec![
    ("ref_into", "Entity", false)
]; "8")]
fn incomplete_field_attr_instruction_2(code_fragment: TokenStream, errs: Vec<(&str, &str, bool)>) {
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

#[test_case(quote! {
    #[into(EntityDto)]
    #[ghosts(EntityDto123| ghost: {123})]
    struct Entity123 {
        test: i32
    }
}, vec!["EntityDto123"]; "into_ghosts_instr")]
#[test_case(quote! {
    #[into(EntityDto)]
    #[ghosts_ref(EntityDto123| ghost: {123})]
    struct Entity123 {
        test: i32
    }
}, vec!["EntityDto123"]; "into_ghosts_ref_instr")]
#[test_case(quote! {
    #[into(EntityDto)]
    #[ghosts_owned(EntityDto123| ghost: {123})]
    struct Entity123 {
        test: i32
    }
}, vec!["EntityDto123"]; "into_ghosts_owned_instr")]
#[test_case(quote! {
    #[map_owned(EntityDto)]
    struct Entity {
        #[ghost(None)]
        test: Option<i32>
    }
}, vec!["None"]; "map_owned_ghosts_instr")]
#[test_case(quote! {
    #[map_owned(EntityDto)]
    struct Entity {
        #[ghost_ref(None)]
        test: Option<i32>
    }
}, vec!["None"]; "map_owned_ghosts_ref_instr")]
#[test_case(quote! {
    #[map_owned(EntityDto)]
    struct Entity {
        #[ghost_owned(None)]
        test: Option<i32>
    }
}, vec!["None"]; "map_owned_ghosts_owned_instr")]
#[test_case(quote! {
    #[map_owned(EntityDto)]
    struct Entity {
        #[ghost({None})]
        test: Option<i32>
    }
}, vec![]; "map_owned_ghosts_instr_correct")]
#[test_case(quote! {
    #[map_owned(EntityDto)]
    struct Entity {
        #[ghost(123)]
        test: i32
    }
}, vec![]; "map_owned_ghosts_instr_correct_2")]
#[test_case(quote! {
    #[from(EntityDto)]
    struct Entity {
        #[child(EntityDto123| test)]
        test: i32
    }
}, vec!["EntityDto123"]; "from_child_instr")]
#[test_case(quote! {
    #[into(EntityDto)]
    struct Entity {
        #[child(EntityDto123| test)]
        test: i32
    }
}, vec!["EntityDto123"]; "into_child_instr")]
#[test_case(quote! {
    #[into(EntityDto)]
    #[children(test: Test)]
    struct Entity {
        #[child(EntityDto| test)]
        test: i32
    }
}, vec![]; "into_children_child_instr")]
#[test_case(quote! {
    #[into(EntityDto)]
    #[children(EntityDto123| test: Test)]
    struct Entity123 {
        test: i32
    }
}, vec!["EntityDto123"]; "into_children_instr")]
#[test_case(quote! {
    #[into(EntityDto)]
    #[where_clause(EntityDto123| T: Clone)]
    struct Entity123 {
        test: i32
    }
}, vec!["EntityDto123"]; "struct_into_where_clause_instr")]
#[test_case(quote! {
    #[into(EnumDto)]
    #[where_clause(EnumDto123| T: Clone)]
    enum Enum {}
}, vec!["EnumDto123"]; "enum_into_where_clause_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    struct Entity123 {
        #[map(EntityDto123| another)]
        test: i32
    }
}, vec!["EntityDto123"]; "struct_map_map_instr")]
#[test_case(quote! {
    #[map(EnumDto)]
    enum Enum {
        #[map(EnumDto123| AnotheVar)]
        Variant
    }
}, vec!["EnumDto123"]; "enum_map_map_instr")]
#[test_case(quote! {
    #[map(EnumDto)]
    enum Enum {
        #[literal(EnumDto123| "Test")]
        Variant
    }
}, vec!["EnumDto123"]; "enum_map_literal_instr")]
#[test_case(quote! {
    #[map(EnumDto)]
    enum Enum {
        #[pattern(EnumDto123| _)]
        Variant
    }
}, vec!["EnumDto123"]; "enum_map_pattern_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    struct Entity123 {
        #[parent(EntityDto123)]
        test: i32
    }
}, vec!["EntityDto123"]; "map_parent_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    struct Entity123 {
        #[o2o(as_type(EntityDto123| f32))]
        test: i32
    }
}, vec!["EntityDto123"]; "map_as_type_instr")]
fn dedicated_field_instruction_mismatch(code_fragment: TokenStream, errs: Vec<&str>) {
    let input: DeriveInput = syn::parse2(code_fragment).unwrap();
    let output = derive(&input);

    if errs.len() > 0 {
        let errors: Vec<Error> = get_error_iter(output).collect();

        assert_eq!(errs.len(), errors.len());

        for err_ty in errs {
            assert!(errors.iter().any(|x| x.to_string() == format!("Type '{}' doesn't match any type specified in trait instructions.", err_ty)))
        }
    } else {
        assert!(output.is_ok())
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