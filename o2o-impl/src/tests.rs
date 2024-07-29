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

// region: missing_map_instructions

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

// endregion: missing_map_instructions

// region: unrecognized_struct_instructions

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
    #[skip_repeat(EntityDto)]
    struct Entity {}
}, vec![ "Member instruction 'skip_repeat' should be used on a member. To turn this message off, use #[o2o(allow_unknown)]" ]; "struct_misplaced_skip_repeat_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    #[skip_repeat(EntityDto)]
    enum Enum {}
}, vec![ "Member instruction 'skip_repeat' should be used on a member. To turn this message off, use #[o2o(allow_unknown)]" ]; "enum_misplaced_skip_repeat_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    #[type_hint()]
    struct Entity {}
}, vec![ "Member instruction 'type_hint' should be used on a member. To turn this message off, use #[o2o(allow_unknown)]" ]; "struct_misplaced_type_hint_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    #[type_hint()]
    enum Enum {}
}, vec![ "Member instruction 'type_hint' should be used on a member. To turn this message off, use #[o2o(allow_unknown)]" ]; "enum_misplaced_type_hint_instr")]
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
    #[map(EntityDto)]
    #[o2o(skip_repeat(EntityDto))]
    struct Entity {}
}, vec![ "Member instruction 'skip_repeat' should be used on a member." ]; "own_struct_misplaced_skip_repeat_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    #[o2o(skip_repeat(EntityDto))]
    enum Enum {}
}, vec![ "Member instruction 'skip_repeat' should be used on a member." ]; "own_enum_misplaced_skip_repeat_instr")]
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

// endregion: unrecognized_struct_instructions

// region: unrecognized_member_instructions

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

// endregion: unrecognized_member_instructions

// region: unrecognized_struct_instructions_no_bark

#[test_case(quote! {
    #[from_owned(NamedStruct)]
    #[mapp(EntityDto)]
    struct Entity {}
}; "struct_unrecognized_instr")]
#[test_case(quote! {
    #[from_owned(NamedStruct)]
    #[mapp::mapp(EntityDto)]
    struct Entity {}
}; "struct_unrecognized_path_instr")]
#[test_case(quote! {
    #[from_owned(EnumDto)]
    #[mapp(EnumDto)]
    enum Enum {}
}; "enum_unrecognized_instr")]
#[test_case(quote! {
    #[from_owned(EnumDto)]
    #[mapp::mapp(EnumDto)]
    enum Enum {}
}; "enum_unrecognized_path_instr")]
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

// endregion: unrecognized_struct_instructions_no_bark

// region: unrecognized_member_instructions_no_bark

#[test_case(quote! {
    #[from_owned(NamedStruct)]
    struct NamedStructDto {
        #[unknown()]
        field: i32,
    }
}; "struct_unknown_instr")]
#[test_case(quote! {
    #[from_owned(NamedStruct)]
    struct NamedStructDto {
        #[unknown::unknown()]
        field: i32,
    }
}; "struct_unknown_path_instr")]
#[test_case(quote! {
    #[from_owned(EnumDto)]
    enum Enum {
        #[unknown()]
        Variant,
    }
}; "enum_unknown_instr")]
#[test_case(quote! {
    #[from_owned(EnumDto)]
    enum Enum {
        #[unkwnown::unknown()]
        Variant,
    }
}; "enum_unknown_path_instr")]
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

// endregion: unrecognized_member_instructions_no_bark

// region: more_than_one_default_instruction

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

// endregion: more_than_one_default_instruction

// region: more_than_one_default_member_instruction

#[test_case(quote! {
    #[map(EntityDto)]
    struct Entity {
        #[parent]
        #[parent]
        x: i32
    }
}, "parent"; "struct_parent_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    enum Enum {
        #[literal(200)]
        #[literal(200)]
        Var
    }
}, "literal"; "enum_literal_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    enum Enum {
        #[pattern(200)]
        #[pattern(200)]
        Var
    }
}, "pattern"; "enum_pattern_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    enum Enum {
        #[type_hint(as {})]
        #[type_hint(as {})]
        Var
    }
}, "type_hint"; "enum_type_hint_instr")]
fn more_than_one_default_member_instruction(code_fragment: TokenStream, err: &str) {
    let input: DeriveInput = syn::parse2(code_fragment).unwrap();
    let output = derive(&input);
    let message = get_error(output, true);

    assert_eq!(message, format!("There can be at most one default #[{}(...)] instruction for a given member.", err));
}

// endregion: more_than_one_default_member_instruction

// region: dedicated_instruction_defined_twice

#[test_case(quote! {
    #[map(EntityDto)]
    #[children(EntityDto| test: Test)]
    #[children(EntityDto| test: Test)]
    struct Entity {}
}, "children", "EntityDto"; "struct_children_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    #[where_clause(EntityDto| T: Clone)]
    #[where_clause(EntityDto| T: Clone)]
    struct Entity {}
}, "where_clause", "EntityDto"; "struct_where_clause_instr")]
#[test_case(quote! {
    #[map(EnumDto)]
    #[where_clause(EnumDto| T: Clone)]
    #[where_clause(EnumDto| T: Clone)]
    enum Enun {}
}, "where_clause", "EnumDto"; "enum_where_clause_instr")]
#[test_case(quote! {
    #[map(EntityDto)]
    #[ghosts(EntityDto| field: { Clone })]
    #[ghosts(EntityDto| field: { Clone })]
    struct Entity {}
}, "ghosts", "EntityDto"; "struct_ghosts_instr")]
fn dedicated_instruction_defined_twice(code_fragment: TokenStream, err_instr: &str, err_ty: &str) {
    let input: DeriveInput = syn::parse2(code_fragment).unwrap();
    let output = derive(&input);
    let message = get_error(output, true);

    assert_eq!(message, format!("Dedicated #[{}(...)] instruction for type {} is already defined.", err_instr, err_ty));
}

// endregion: dedicated_instruction_defined_twice

// region: dedicated_member_instruction_defined_twice

#[test_case(quote! {
    #[map(EntityDto)]
    struct Entity {
        #[parent(EntityDto)]
        #[parent(EntityDto)]
        x: i32
    }
}, "parent", "EntityDto"; "struct_parent_instr")]
#[test_case(quote! {
    #[map(EnumDto)]
    enum Enum {
        #[literal(EnumDto| 200)]
        #[literal(EnumDto| 200)]
        Var
    }
}, "literal", "EnumDto"; "enum_literal_instr")]
#[test_case(quote! {
    #[map(EnumDto)]
    enum Enum {
        #[pattern(EnumDto| 200)]
        #[pattern(EnumDto| 200)]
        Var
    }
}, "pattern", "EnumDto"; "enum_pattern_instr")]
#[test_case(quote! {
    #[map(EnumDto)]
    enum Enum {
        #[type_hint(EnumDto| as {})]
        #[type_hint(EnumDto| as {})]
        Var
    }
}, "type_hint", "EnumDto"; "enum_type_hint_instr")]
fn dedicated_member_instruction_defined_twice(code_fragment: TokenStream, err_instr: &str, err_ty: &str) {
    let input: DeriveInput = syn::parse2(code_fragment).unwrap();
    let output = derive(&input);
    let message = get_error(output, true);

    assert_eq!(message, format!("Dedicated #[{}(...)] instruction for type {} is already defined.", err_instr, err_ty));
}

// endregion: dedicated_member_instruction_defined_twice

// region: dedicated_field_instruction_mismatch

#[test_case(quote! {
    #[into(EntityDto)]
    #[ghosts(EntityDto123| ghost: {123})]
    struct Entity123 {
        test: i32
    }
}, vec!["EntityDto123"]; "into_ghosts_instr")]
#[test_case(quote! {
    #[try_into(EntityDto, SomeError)]
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
    #[try_map_owned(EntityDto, SomeError)]
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
    #[try_into(EntityDto, SomeError)]
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
    #[try_map(EnumDto, SomeError)]
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
    #[map(EnumDto)]
    enum Enum {
        #[type_hint(EnumDto123| as ())]
        Variant
    }
}, vec!["EnumDto123"]; "enum_map_type_hint_instr")]
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

// endregion: dedicated_field_instruction_mismatch

#[test_case(quote!{
    #[map(TestDto| vars(test: {123}), vars(test2: {123}))]
    struct Test;
}, "Instruction parameter 'vars' was already set."; "1")]
#[test_case(quote!{
    #[map(TestDto| vars(test: {123}), repeat(), vars(test2: {123}))]
    struct Test;
}, "Instruction parameter 'vars' was already set."; "2")]
#[test_case(quote!{
    #[map(TestDto| repeat(), repeat())]
    struct Test;
}, "Instruction parameter 'repeat' was already set."; "3")]
#[test_case(quote!{
    #[map(TestDto| repeat(), vars(test: {123}), repeat())]
    struct Test;
}, "Instruction parameter 'repeat' was already set."; "4")]
#[test_case(quote!{
    #[map(TestDto| skip_repeat, skip_repeat)]
    struct Test;
}, "Instruction parameter 'skip_repeat' was already set."; "5")]
#[test_case(quote!{
    #[map(TestDto| skip_repeat, repeat(), skip_repeat)]
    struct Test;
}, "Instruction parameter 'skip_repeat' was already set."; "6")]
#[test_case(quote!{
    #[map(TestDto| stop_repeat, stop_repeat)]
    struct Test;
}, "Instruction parameter 'stop_repeat' was already set."; "7")]
#[test_case(quote!{
    #[map(TestDto| stop_repeat, repeat(), stop_repeat)]
    struct Test;
}, "Instruction parameter 'stop_repeat' was already set."; "8")]
#[test_case(quote!{
    #[map(TestDto| skip_repeat, skip_repeat)]
    struct Test;
}, "Instruction parameter 'skip_repeat' was already set."; "9")]
#[test_case(quote!{
    #[map(TestDto| skip_repeat, repeat(), skip_repeat)]
    struct Test;
}, "Instruction parameter 'skip_repeat' was already set."; "10")]
#[test_case(quote!{
    #[map(TestDto| attribute(test), attribute(test))]
    struct Test;
}, "Instruction parameter 'attribute' was already set."; "11")]
#[test_case(quote!{
    #[map(TestDto| attribute(test), skip_repeat, attribute(test))]
    struct Test;
}, "Instruction parameter 'attribute' was already set."; "12")]
#[test_case(quote!{
    #[map(TestDto| impl_attribute(test), impl_attribute(test))]
    struct Test;
}, "Instruction parameter 'impl_attribute' was already set."; "13")]
#[test_case(quote!{
    #[map(TestDto| impl_attribute(test), skip_repeat, impl_attribute(test))]
    struct Test;
}, "Instruction parameter 'impl_attribute' was already set."; "14")]
#[test_case(quote!{
    #[map(TestDto| inner_attribute(test), inner_attribute(test))]
    struct Test;
}, "Instruction parameter 'inner_attribute' was already set."; "15")]
#[test_case(quote!{
    #[map(TestDto| inner_attribute(test), skip_repeat, inner_attribute(test))]
    struct Test;
}, "Instruction parameter 'inner_attribute' was already set."; "16")]
fn trait_instruction_defined_twice(code_fragment: TokenStream, err: &str) {
    let input: DeriveInput = syn::parse2(code_fragment).unwrap();
    let output = derive(&input);
    let message = get_error(output, false);

    assert_eq!(message, err);
}

// region: missing_children_instruction

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
    #[try_map(EntityDto, SomeError)]
    #[try_map(EntityModel, SomeError)]
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
    #[try_map(EntityModel, SomeError)]
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

// endregion: missing_children_instruction

// region: member_instr_on_wrong_member

#[test_case(quote! {
    #[map_owned(i32| _ => todo!())]
    enum Test {
        #[parent]
        #[literal(123)]
        Var
    }
}, vec!["parent"]; "1")]
#[test_case(quote! {
    #[map(TestDto)]
    struct Test {
        #[literal(123)]
        x: i32
    }
}, vec!["literal"]; "2")]
#[test_case(quote! {
    #[map(TestDto)]
    struct Test {
        #[pattern(123..321)]
        x: i32
    }
}, vec!["pattern"]; "3")]
#[test_case(quote! {
    #[map(TestDto)]
    struct Test {
        #[type_hint(as {})]
        x: i32
    }
}, vec!["type_hint"]; "4")]
#[test_case(quote! {
    #[map(TestDto)]
    struct Test {
        #[ghosts(x: {123})]
        x: i32
    }
}, vec!["ghosts"]; "5")]
#[test_case(quote! {
    #[map(TestDto)]
    struct Test {
        #[ghosts_owned(x: {123})]
        x: i32
    }
}, vec!["ghosts_owned"]; "6")]
#[test_case(quote! {
    #[map(TestDto)]
    struct Test {
        #[ghosts_ref(x: {123})]
        x: i32
    }
}, vec!["ghosts_ref"]; "7")]
#[test_case(quote! {
    #[map(TestDto)]
    struct Test {
        #[literal(123)]
        #[pattern(123..321)]
        #[type_hint(as {})]
        #[ghosts(x: {123})]
        x: i32
    }
}, vec!["literal", "pattern", "type_hint", "ghosts"]; "8")]
fn member_instr_on_wrong_member(code_fragment: TokenStream, errs: Vec<&str>) {
    let input: DeriveInput = syn::parse2(code_fragment).unwrap();
    let output = derive(&input);

    if errs.len() > 0 {
        let errors: Vec<Error> = get_error_iter(output).collect();

        assert_eq!(errs.len(), errors.len());

        for err in errs {
            assert!(errors.iter().any(|x| x.to_string() == format!("Instruction #[{}(...)] is not supported for this member.", err)))
        }
    } else {
        assert!(output.is_ok())
    }
}

// endregion: member_instr_on_wrong_member

// region: incomplete_children_instruction

#[test_case(quote! {
    #[map(EntityDto)]
    #[try_map(EntityModel, SomeError)]
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
    #[try_map(EntityModel, SomeError)]
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

// endregion: incomplete_children_instruction

// region: incomplete_ghost_instruction

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
    #[try_map(Entity, SomeError)]
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
    #[try_from(Entity2, SomeError)]
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

// endregion: incomplete_ghost_instruction

// region: incomplete_field_attr_instruction

#[test_case(quote! {
    #[into(Entity as {})]
    struct EntityDto (i32);
}, vec![
    ("owned_into", "Entity", false),
    ("ref_into", "Entity", false)
]; "1")]
#[test_case(quote! {
    #[try_into(Entity as {}, SomeError)]
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
    #[try_from(Entity2 as {}, SomeError)]
    struct EntityDto (#[from(Entity2| {123})]i32);
}, vec![
    ("from_owned", "Entity", true),
    ("from_ref", "Entity", true),
    ("owned_into", "Entity", false),
    ("ref_into", "Entity", false),
]; "7")]
#[test_case(quote! {
    #[owned_try_into(StuffWrapper, SomeError| return StuffWrapper { payload: @ })]
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

// endregion: incomplete_field_attr_instruction

// region: incomplete_variant_field_attr_instruction

#[test_case(quote! {from_owned}, TokenStream::new(), vec![
    (0, "Var3", "from_owned", "TestDto", true),
    (1, "Var3", "from_owned", "TestDto", true),
    (0, "Var4", "from_owned", "TestDto", true),
]; "1")]
#[test_case(quote! {from_ref}, TokenStream::new(), vec![
    (0, "Var3", "from_ref", "TestDto", true),
    (1, "Var3", "from_ref", "TestDto", true),
    (0, "Var4", "from_ref", "TestDto", true),
]; "2")]
#[test_case(quote! {from}, TokenStream::new(), vec![
    (0, "Var3", "from_owned", "TestDto", true),
    (1, "Var3", "from_owned", "TestDto", true),
    (0, "Var4", "from_owned", "TestDto", true),
    (0, "Var3", "from_ref", "TestDto", true),
    (1, "Var3", "from_ref", "TestDto", true),
    (0, "Var4", "from_ref", "TestDto", true),
]; "3")]
#[test_case(quote! {owned_into}, TokenStream::new(), vec![
    (0, "Var3", "owned_into", "TestDto", false),
    (1, "Var3", "owned_into", "TestDto", false),
    (0, "Var4", "owned_into", "TestDto", false),
]; "4")]
#[test_case(quote! {ref_into}, TokenStream::new(), vec![
    (0, "Var3", "ref_into", "TestDto", false),
    (1, "Var3", "ref_into", "TestDto", false),
    (0, "Var4", "ref_into", "TestDto", false),
]; "5")]
#[test_case(quote! {into}, TokenStream::new(), vec![
    (0, "Var3", "owned_into", "TestDto", false),
    (1, "Var3", "owned_into", "TestDto", false),
    (0, "Var4", "owned_into", "TestDto", false),
    (0, "Var3", "ref_into", "TestDto", false),
    (1, "Var3", "ref_into", "TestDto", false),
    (0, "Var4", "ref_into", "TestDto", false),
]; "6")]
#[test_case(quote! {try_from_owned}, quote!(, String), vec![
    (0, "Var3", "try_from_owned", "TestDto", true),
    (1, "Var3", "try_from_owned", "TestDto", true),
    (0, "Var4", "try_from_owned", "TestDto", true),
]; "7")]
#[test_case(quote! {try_from_ref}, quote!(, String), vec![
    (0, "Var3", "try_from_ref", "TestDto", true),
    (1, "Var3", "try_from_ref", "TestDto", true),
    (0, "Var4", "try_from_ref", "TestDto", true),
]; "8")]
#[test_case(quote! {try_from}, quote!(, String), vec![
    (0, "Var3", "try_from_owned", "TestDto", true),
    (1, "Var3", "try_from_owned", "TestDto", true),
    (0, "Var4", "try_from_owned", "TestDto", true),
    (0, "Var3", "try_from_ref", "TestDto", true),
    (1, "Var3", "try_from_ref", "TestDto", true),
    (0, "Var4", "try_from_ref", "TestDto", true),
]; "9")]
#[test_case(quote! {owned_try_into}, quote!(, String), vec![
    (0, "Var3", "owned_try_into", "TestDto", false),
    (1, "Var3", "owned_try_into", "TestDto", false),
    (0, "Var4", "owned_try_into", "TestDto", false),
]; "10")]
#[test_case(quote! {ref_try_into}, quote!(, String), vec![
    (0, "Var3", "ref_try_into", "TestDto", false),
    (1, "Var3", "ref_try_into", "TestDto", false),
    (0, "Var4", "ref_try_into", "TestDto", false),
]; "11")]
#[test_case(quote! {try_into}, quote!(, String), vec![
    (0, "Var3", "owned_try_into", "TestDto", false),
    (1, "Var3", "owned_try_into", "TestDto", false),
    (0, "Var4", "owned_try_into", "TestDto", false),
    (0, "Var3", "ref_try_into", "TestDto", false),
    (1, "Var3", "ref_try_into", "TestDto", false),
    (0, "Var4", "ref_try_into", "TestDto", false),
]; "12")]
#[test_case(quote! {map_owned}, TokenStream::new(), vec![
    (0, "Var3", "owned_into", "TestDto", false),
    (1, "Var3", "owned_into", "TestDto", false),
    (0, "Var4", "owned_into", "TestDto", false),
    (0, "Var3", "from_owned", "TestDto", true),
    (1, "Var3", "from_owned", "TestDto", true),
    (0, "Var4", "from_owned", "TestDto", true),
]; "13")]
#[test_case(quote! {map_ref}, TokenStream::new(), vec![
    (0, "Var3", "ref_into", "TestDto", false),
    (1, "Var3", "ref_into", "TestDto", false),
    (0, "Var4", "ref_into", "TestDto", false),
    (0, "Var3", "from_ref", "TestDto", true),
    (1, "Var3", "from_ref", "TestDto", true),
    (0, "Var4", "from_ref", "TestDto", true),
]; "14")]
#[test_case(quote! {map}, TokenStream::new(), vec![
    (0, "Var3", "owned_into", "TestDto", false),
    (1, "Var3", "owned_into", "TestDto", false),
    (0, "Var4", "owned_into", "TestDto", false),
    (0, "Var3", "from_owned", "TestDto", true),
    (1, "Var3", "from_owned", "TestDto", true),
    (0, "Var4", "from_owned", "TestDto", true),
    (0, "Var3", "ref_into", "TestDto", false),
    (1, "Var3", "ref_into", "TestDto", false),
    (0, "Var4", "ref_into", "TestDto", false),
    (0, "Var3", "from_ref", "TestDto", true),
    (1, "Var3", "from_ref", "TestDto", true),
    (0, "Var4", "from_ref", "TestDto", true),
]; "15")]
#[test_case(quote! {try_map_owned}, quote!(, String), vec![
    (0, "Var3", "owned_try_into", "TestDto", false),
    (1, "Var3", "owned_try_into", "TestDto", false),
    (0, "Var4", "owned_try_into", "TestDto", false),
    (0, "Var3", "try_from_owned", "TestDto", true),
    (1, "Var3", "try_from_owned", "TestDto", true),
    (0, "Var4", "try_from_owned", "TestDto", true),
]; "16")]
#[test_case(quote! {try_map_ref}, quote!(, String), vec![
    (0, "Var3", "ref_try_into", "TestDto", false),
    (1, "Var3", "ref_try_into", "TestDto", false),
    (0, "Var4", "ref_try_into", "TestDto", false),
    (0, "Var3", "try_from_ref", "TestDto", true),
    (1, "Var3", "try_from_ref", "TestDto", true),
    (0, "Var4", "try_from_ref", "TestDto", true),
]; "17")]
#[test_case(quote! {try_map}, quote!(, String), vec![
    (0, "Var3", "owned_try_into", "TestDto", false),
    (1, "Var3", "owned_try_into", "TestDto", false),
    (0, "Var4", "owned_try_into", "TestDto", false),
    (0, "Var3", "try_from_owned", "TestDto", true),
    (1, "Var3", "try_from_owned", "TestDto", true),
    (0, "Var4", "try_from_owned", "TestDto", true),
    (0, "Var3", "ref_try_into", "TestDto", false),
    (1, "Var3", "ref_try_into", "TestDto", false),
    (0, "Var4", "ref_try_into", "TestDto", false),
    (0, "Var3", "try_from_ref", "TestDto", true),
    (1, "Var3", "try_from_ref", "TestDto", true),
    (0, "Var4", "try_from_ref", "TestDto", true),
]; "18")]
#[test_case(quote! {owned_into_existing}, TokenStream::new(), vec![
    (0, "Var3", "owned_into_existing", "TestDto", false),
    (1, "Var3", "owned_into_existing", "TestDto", false),
    (0, "Var4", "owned_into_existing", "TestDto", false),
]; "19")]
#[test_case(quote! {ref_into_existing}, TokenStream::new(), vec![
    (0, "Var3", "ref_into_existing", "TestDto", false),
    (1, "Var3", "ref_into_existing", "TestDto", false),
    (0, "Var4", "ref_into_existing", "TestDto", false),
]; "20")]
#[test_case(quote! {into_existing}, TokenStream::new(), vec![
    (0, "Var3", "owned_into_existing", "TestDto", false),
    (1, "Var3", "owned_into_existing", "TestDto", false),
    (0, "Var4", "owned_into_existing", "TestDto", false),
    (0, "Var3", "ref_into_existing", "TestDto", false),
    (1, "Var3", "ref_into_existing", "TestDto", false),
    (0, "Var4", "ref_into_existing", "TestDto", false),
]; "21")]
#[test_case(quote! {owned_try_into_existing}, quote!(, String), vec![
    (0, "Var3", "owned_try_into_existing", "TestDto", false),
    (1, "Var3", "owned_try_into_existing", "TestDto", false),
    (0, "Var4", "owned_try_into_existing", "TestDto", false),
]; "22")]
#[test_case(quote! {ref_try_into_existing}, quote!(, String), vec![
    (0, "Var3", "ref_try_into_existing", "TestDto", false),
    (1, "Var3", "ref_try_into_existing", "TestDto", false),
    (0, "Var4", "ref_try_into_existing", "TestDto", false),
]; "23")]
#[test_case(quote! {try_into_existing}, quote!(, String), vec![
    (0, "Var3", "owned_try_into_existing", "TestDto", false),
    (1, "Var3", "owned_try_into_existing", "TestDto", false),
    (0, "Var4", "owned_try_into_existing", "TestDto", false),
    (0, "Var3", "ref_try_into_existing", "TestDto", false),
    (1, "Var3", "ref_try_into_existing", "TestDto", false),
    (0, "Var4", "ref_try_into_existing", "TestDto", false),
]; "24")]
fn incomplete_variant_field_attr_instruction(code_fragment: TokenStream, err_ty: TokenStream, errs: Vec<(u32, &str, &str, &str, bool)>) {
    let code_fragment = quote! {
        #[#code_fragment(TestDto #err_ty)]
        enum Test2Dto {
            Var1,
            Var2,
            #[type_hint(as {})] Var3(i32, i32),
            #[type_hint(as {})] Var4(i32),
        }
    };

    let input: DeriveInput = syn::parse2(code_fragment).unwrap();
    let output = derive(&input);

    if errs.len() > 0 {
        let errors: Vec<Error> = get_error_iter(output).collect();

        assert_eq!(errs.len(), errors.len());

        for (idx, variant, field, ty, or_action) in errs {
            assert!(errors.iter().any(|x| x.to_string() == format!("Member {} of a variant {} should have member trait instruction with field name{}, that corresponds to #[{}({}...)] trait instruction", idx, variant, if or_action {" or an action"} else {""}, field, ty)))
        }
    } else {
        assert!(output.is_ok())
    }
}

// endregion: incomplete_variant_field_attr_instruction

// region: incomplete_field_attr_instruction_2

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
    #[try_into_existing(Entity as {}, SomeError)]
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

// endregion: incomplete_field_attr_instruction_2

// region: fallible_map_instruction_no_error_type

#[test_case(quote!(try_map), None; "try_map")]
#[test_case(quote!(try_map_owned), None; "try_map_owned")]
#[test_case(quote!(try_map_ref), None; "try_map_ref")]
#[test_case(quote!(try_from), None; "try_from")]
#[test_case(quote!(try_into), None; "try_into")]
#[test_case(quote!(try_from_owned), None; "try_from_owned")]
#[test_case(quote!(try_from_ref), None; "try_from_ref")]
#[test_case(quote!(owned_try_into), None; "owned_try_into")]
#[test_case(quote!(ref_try_into), None; "ref_try_into")]
#[test_case(quote!(try_into_existing), None; "try_into_existing")]
#[test_case(quote!(owned_try_into_existing), None; "owned_try_into_existing")]
#[test_case(quote!(ref_try_into_existing), None; "ref_try_into_existing")]

#[test_case(quote!(try_map), Some(quote!(as {})); "try_map_as")]
#[test_case(quote!(try_map_owned), Some(quote!(as {})); "try_map_owned_as")]
#[test_case(quote!(try_map_ref), Some(quote!(as {})); "try_map_ref_as")]
#[test_case(quote!(try_from), Some(quote!(as {})); "try_from_as")]
#[test_case(quote!(try_into), Some(quote!(as {})); "try_into_as")]
#[test_case(quote!(try_from_owned), Some(quote!(as {})); "try_from_owned_as")]
#[test_case(quote!(try_from_ref), Some(quote!(as {})); "try_from_ref_as")]
#[test_case(quote!(owned_try_into), Some(quote!(as {})); "owned_try_into_as")]
#[test_case(quote!(ref_try_into), Some(quote!(as {})); "ref_try_into_as")]
#[test_case(quote!(try_into_existing), Some(quote!(as {})); "try_into_existing_as")]
#[test_case(quote!(owned_try_into_existing), Some(quote!(as {})); "owned_try_into_existing_as")]
#[test_case(quote!(ref_try_into_existing), Some(quote!(as {})); "ref_try_into_existing_as")]

#[test_case(quote!(try_map), Some(quote!(| return true)); "try_map_return")]
#[test_case(quote!(try_map_owned), Some(quote!(| return true)); "try_map_owned_return")]
#[test_case(quote!(try_map_ref), Some(quote!(| return true)); "try_map_ref_return")]
#[test_case(quote!(try_from), Some(quote!(| return true)); "try_from_return")]
#[test_case(quote!(try_into), Some(quote!(| return true)); "try_into_return")]
#[test_case(quote!(try_from_owned), Some(quote!(| return true)); "try_from_owned_return")]
#[test_case(quote!(try_from_ref), Some(quote!(| return true)); "try_from_ref_return")]
#[test_case(quote!(owned_try_into), Some(quote!(| return true)); "owned_try_into_return")]
#[test_case(quote!(ref_try_into), Some(quote!(| return true)); "ref_try_into_return")]
#[test_case(quote!(try_into_existing), Some(quote!(| return true)); "try_into_existing_return")]
#[test_case(quote!(owned_try_into_existing), Some(quote!(| return true)); "owned_try_into_existing_return")]
#[test_case(quote!(ref_try_into_existing), Some(quote!(| return true)); "ref_try_into_existing_return")]
fn fallible_map_instruction_no_error_type(instr: TokenStream, postfix: Option<TokenStream>) {
    let code_fragment = quote! {
        #[#instr(EntityDto #postfix)]
        struct Entity {
            test: i32
        }
    };

    let input: DeriveInput = syn::parse2(code_fragment).unwrap();
    let output = derive(&input);
    let message = get_error(output, true);

    assert_eq!(message, "Error type should be specified for fallible instruction.");
}

// endregion: fallible_map_instruction_no_error_type

// region: infallible_map_instruction_error_type

#[test_case(quote!(map), Some(quote!(, ErrorType)); "map")]
#[test_case(quote!(map_owned), Some(quote!(, ErrorType)); "map_owned")]
#[test_case(quote!(map_ref), Some(quote!(, ErrorType)); "map_ref")]
#[test_case(quote!(from), Some(quote!(, ErrorType)); "from")]
#[test_case(quote!(into), Some(quote!(, ErrorType)); "into")]
#[test_case(quote!(from_owned), Some(quote!(, ErrorType)); "from_owned")]
#[test_case(quote!(from_ref), Some(quote!(, ErrorType)); "from_ref")]
#[test_case(quote!(owned_into), Some(quote!(, ErrorType)); "owned_into")]
#[test_case(quote!(ref_into), Some(quote!(, ErrorType)); "ref_into")]
#[test_case(quote!(into_existing), Some(quote!(, ErrorType)); "into_existing")]
#[test_case(quote!(owned_into_existing), Some(quote!(, ErrorType)); "owned_into_existing")]
#[test_case(quote!(ref_into_existing), Some(quote!(, ErrorType)); "ref_into_existing")]

#[test_case(quote!(map), Some(quote!(as {}, ErrorType)); "map_as")]
#[test_case(quote!(map_owned), Some(quote!(as {}, ErrorType)); "map_owned_as")]
#[test_case(quote!(map_ref), Some(quote!(as {}, ErrorType)); "map_ref_as")]
#[test_case(quote!(from), Some(quote!(as {}, ErrorType)); "from_as")]
#[test_case(quote!(into), Some(quote!(as {}, ErrorType)); "into_as")]
#[test_case(quote!(from_owned), Some(quote!(as {}, ErrorType)); "from_owned_as")]
#[test_case(quote!(from_ref), Some(quote!(as {}, ErrorType)); "from_ref_as")]
#[test_case(quote!(owned_into), Some(quote!(as {}, ErrorType)); "owned_into_as")]
#[test_case(quote!(ref_into), Some(quote!(as {}, ErrorType)); "ref_into_as")]
#[test_case(quote!(into_existing), Some(quote!(as {}, ErrorType)); "into_existing_as")]
#[test_case(quote!(owned_into_existing), Some(quote!(as {}, ErrorType)); "owned_into_existing_as")]
#[test_case(quote!(ref_into_existing), Some(quote!(as {}, ErrorType)); "ref_into_existing_as")]

#[test_case(quote!(map), Some(quote!(, ErrorType| return true)); "map_return")]
#[test_case(quote!(map_owned), Some(quote!(, ErrorType| return true)); "map_owned_return")]
#[test_case(quote!(map_ref), Some(quote!(, ErrorType| return true)); "map_ref_return")]
#[test_case(quote!(from), Some(quote!(, ErrorType| return true)); "from_return")]
#[test_case(quote!(into), Some(quote!(, ErrorType| return true)); "into_return")]
#[test_case(quote!(from_owned), Some(quote!(, ErrorType| return true)); "from_owned_return")]
#[test_case(quote!(from_ref), Some(quote!(, ErrorType| return true)); "from_ref_return")]
#[test_case(quote!(owned_into), Some(quote!(, ErrorType| return true)); "owned_into_return")]
#[test_case(quote!(ref_into), Some(quote!(, ErrorType| return true)); "ref_into_return")]
#[test_case(quote!(into_existing), Some(quote!(, ErrorType| return true)); "into_existing_return")]
#[test_case(quote!(owned_into_existing), Some(quote!(, ErrorType| return true)); "owned_into_existing_return")]
#[test_case(quote!(ref_into_existing), Some(quote!(, ErrorType| return true)); "ref_into_existing_return")]
fn infallible_map_instruction_error_type(instr: TokenStream, postfix: Option<TokenStream>) {
    let code_fragment = quote! {
        #[#instr(EntityDto #postfix)]
        struct Entity {
            test: i32
        }
    };

    let input: DeriveInput = syn::parse2(code_fragment).unwrap();
    let output = derive(&input);
    let message = get_error(output, true);

    assert_eq!(message, "Error type should not be specified for infallible instruction.");
}

// endregion: infallible_map_instruction_error_type

// region: trait_attr_repeat

#[test_case(quote! {
    #[from_owned(i64| repeat(), return Self(@.to_string()))]
    #[from_owned(i32| repeat(), return Self(@.to_string()))]
    struct Wrapper(String);
}, "Previous repeat() instruction must be terminated with 'stop_repeat'"; "1")]
#[test_case(quote! {
    #[from_owned(i64| repeat(), return Self(@.to_string()))]
    #[from_owned(i32| return Self(@.to_string()))]
    struct Wrapper(String);
}, "Quick Return statement will be overriden. Did you forget to use 'skip_repeat'?"; "2")]
#[test_case(quote! {
    #[from_owned(i64| repeat(), vars(msg: {"test".into()}), return Self(msg))]
    #[from_owned(i32| vars(msg: {"123".into()}))]
    struct Wrapper(String);
}, "Vars will be overriden. Did you forget to use 'skip_repeat'?"; "3")]
#[test_case(quote! {
    #[from_owned(i64| repeat(vars), vars(msg: {"test".into()}), return Self(msg))]
    #[from_owned(i32| vars(msg: {"123".into()}))]
    struct Wrapper(String);
}, "Vars will be overriden. Did you forget to use 'skip_repeat'?"; "4")]
#[test_case(quote! {
    #[from_owned(i64| repeat(quick_return), vars(msg: {"test".into()}), return Self(msg))]
    #[from_owned(i32| vars(msg: {"123".into()}), return Self(msg))]
    struct Wrapper(String);
}, "Quick Return statement will be overriden. Did you forget to use 'skip_repeat'?"; "5")]
#[test_case(quote! {
    #[from_owned(i64| repeat(test), vars(msg: {"test".into()}), return Self(msg))]
    #[from_owned(i32)]
    struct Wrapper(String);
}, "#[repeat] of instruction type 'test' is not supported. Supported types are: vars, update, quick_return, default_case"; "6")]
fn trait_attr_repeat(code_fragment: TokenStream, err: &str) {
    let input: DeriveInput = syn::parse2(code_fragment).unwrap();
    let output = derive(&input);
    let message = get_error(output, false);

    assert_eq!(message, err);
}

// endregion: trait_attr_repeat

// region: permeating_repeat

#[test_case(quote!{
    #[map(TestDto)]
    struct Test {
        #[o2o(repeat(permeate()))]
        x1: i32,
    }
})]
fn permeating_repeat(code_fragment: TokenStream) {
    let input: DeriveInput = syn::parse2(code_fragment).unwrap();
    let output = derive(&input);
    let message = get_error(output, true);

    assert_eq!(message, "Permeating repeat instruction is only applicable to enum variant fields.");
}

// endregion: permeating_repeat

// region: item_attributes

#[test_case(quote!{
    #[map(TestDto| 
        impl_attribute(impl_attribute(param)), 
        attribute(attribute(param)), 
        inner_attribute(inner_param(param))
    )]
    #[into_existing(TestDto| 
        impl_attribute(impl_attribute(param)), 
        attribute(attribute(param)), 
        inner_attribute(inner_param(param))
    )]
    struct Test {
        x: i32,
    }
},
quote!{
    #[impl_attribute(param)]
    impl ::core::convert::From<TestDto> for Test {
        #[attribute(param)]
        fn from(value: TestDto) -> Test {
            #![inner_param(param)]
            Test { x: value.x, }
        }
    }

    #[impl_attribute(param)]
    impl ::core::convert::From<&TestDto> for Test {
        #[attribute(param)]
        fn from(value: &TestDto) -> Test {
            #![inner_param(param)]
            Test { x: value.x, }
        }
    }
    #[impl_attribute(param)]
    impl ::core::convert::Into<TestDto> for Test {
        #[attribute(param)]
        fn into(self) -> TestDto {
            #![inner_param(param)]
            TestDto { x: self.x, }
        }
    }
    #[impl_attribute(param)]
    impl ::core::convert::Into<TestDto> for &Test {
        #[attribute(param)]
        fn into(self) -> TestDto {
            #![inner_param(param)]
            TestDto { x: self.x, }
        }
    }
    #[impl_attribute(param)]
    impl o2o::traits::IntoExisting<TestDto> for Test {
        #[attribute(param)]
        fn into_existing(self, other: &mut TestDto) {
            #![inner_param(param)]
            other.x = self.x;
        }
    }
    #[impl_attribute(param)]
    impl o2o::traits::IntoExisting<TestDto> for &Test {
        #[attribute(param)]
        fn into_existing(self, other: &mut TestDto) {
            #![inner_param(param)]
            other.x = self.x;
        }
    }
}; "1")]
#[test_case(quote!{
    #[try_map(TestDto, String| 
        impl_attribute(impl_attribute(param)), 
        attribute(attribute(param)), 
        inner_attribute(inner_param(param))
    )]
    #[try_into_existing(TestDto, String| 
        impl_attribute(impl_attribute(param)), 
        attribute(attribute(param)), 
        inner_attribute(inner_param(param))
    )]
    struct Test {
        x: i32,
    }
},
quote!{
    #[impl_attribute(param)]
    impl ::core::convert::TryFrom<TestDto> for Test {
        type Error = String;
        #[attribute(param)]
        fn try_from(value: TestDto) -> Result<Test, String> {
            #![inner_param(param)]
            Ok(Test { x: value.x, })
        }
    }
    #[impl_attribute(param)]
    impl ::core::convert::TryFrom<&TestDto> for Test {
        type Error = String;
        #[attribute(param)]
        fn try_from(value: &TestDto) -> Result<Test, String> {
            #![inner_param(param)]
            Ok(Test { x: value.x, })
        }
    }
    #[impl_attribute(param)]
    impl ::core::convert::TryInto<TestDto> for Test {
        type Error = String;
        #[attribute(param)]
        fn try_into(self) -> Result<TestDto, String> {
            #![inner_param(param)]
            Ok(TestDto { x: self.x, })
        }
    }
    #[impl_attribute(param)]
    impl ::core::convert::TryInto<TestDto> for &Test {
        type Error = String;
        #[attribute(param)]
        fn try_into(self) -> Result<TestDto, String> {
            #![inner_param(param)]
            Ok(TestDto { x: self.x, })
        }
    }
    #[impl_attribute(param)]
    impl o2o::traits::TryIntoExisting<TestDto> for Test {
        type Error = String;
        #[attribute(param)]
        fn try_into_existing(self, other: &mut TestDto) -> Result<(), String> {
            #![inner_param(param)]
            other.x = self.x;
            Ok(())
        }
    }
    #[impl_attribute(param)]
    impl o2o::traits::TryIntoExisting<TestDto> for &Test {
        type Error = String;
        #[attribute(param)]
        fn try_into_existing(self, other: &mut TestDto) -> Result<(), String> {
            #![inner_param(param)]
            other.x = self.x;
            Ok(())
        }
    }
}; "2")]
fn item_attributes(code_fragment: TokenStream, expected_output: TokenStream) {
    let input: DeriveInput = syn::parse2(code_fragment).unwrap();
    let output = derive(&input);

    assert!(output.is_ok());
    assert_eq!(output.unwrap().to_string().trim(), expected_output.to_string().trim());
}

// endregion: item_attributes

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
