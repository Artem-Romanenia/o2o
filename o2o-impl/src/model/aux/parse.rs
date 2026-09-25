use crate::model::*;

pub(crate) fn try_parse_container_ident(input: ParseStream, can_be_empty_after: bool) -> Option<TypePath> {
    if peek_container_path(input, can_be_empty_after) {
        let ident = input.parse::<syn::Path>();
        if input.peek(Token![|]) {
            input.parse::<Token![|]>().unwrap();
        }
        return ident.ok().map(|x| x.into());
    }
    None
}

pub(crate) fn peek_container_path(input: ParseStream, can_be_empty: bool) -> bool {
    let fork = input.fork();
    match fork.parse::<syn::Path>() {
        Ok(_) => (can_be_empty && fork.is_empty()) || fork.peek(Token![|]),
        Err(_) => false,
    }
}

pub(crate) fn peek_member(input: ParseStream) -> bool {
    if input.peek(Ident) {
        return true;
    }

    let fork = input.fork();
    fork.parse::<syn::Index>().is_ok()
}

pub(crate) fn peek_ghost_field_name(input: ParseStream) -> bool {
    peek_member(input) && (input.peek2(Token![:]) || input.peek2(Brace) || input.peek2(Paren))
}

pub(crate) fn try_parse_optional_ident(input: ParseStream) -> Option<Member> {
    if peek_member(input) && input.peek2(Token![,]) {
        let ident = input.parse::<Member>().ok();
        input.parse::<Token![,]>().unwrap();
        return ident;
    }
    if peek_member(input) {
        let fork = input.fork();
        fork.parse::<Member>().unwrap();
        if fork.is_empty() {
            return input.parse::<Member>().ok();
        }
    }
    None
}