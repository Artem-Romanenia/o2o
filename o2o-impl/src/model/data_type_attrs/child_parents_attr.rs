use std::hash::Hash;

use crate::{model::*};

pub(crate) struct ChildParentsAttr {
    pub container_ty: Option<TypePath>,
    pub child_parents: Punctuated<ChildParentData, Token![,]>,
}

impl Parse for ChildParentsAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ChildParentsAttr {
            container_ty: try_parse_container_ident(input, false),
            child_parents: try_parse_child_parents(input)?,
        })
    }
}

pub(crate) struct ChildParentData {
    pub ty: syn::Path,
    pub type_hint: TypeHint,
    pub actions: Punctuated<ChildParentAction, Token![,]>,
    pub field_path: Punctuated<Member, Token![.]>,
    field_path_str: String,
}

impl ChildParentData {
    pub(crate) fn check_match(&self, path: &str) -> bool {
        self.field_path_str == path
    }
}

impl PartialEq for ChildParentData {
    fn eq(&self, other: &Self) -> bool {
        self.field_path_str == other.field_path_str
    }
}
impl Eq for ChildParentData {}
impl Hash for ChildParentData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.field_path_str.hash(state)
    }
}

pub(crate) struct ChildParentAction {
    pub action: InlineExpression,
    pub applicable_to: ApplicableTo
}

impl Parse for ChildParentAction{
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse::<Ident>()?;
        let kind_str = ident.to_string();

        input.parse::<Token![:]>()?;

        Ok(ChildParentAction { 
            action: input.parse()?, 
            applicable_to: appl_to(&kind_str)
        })
    }
}

impl ChildParentAction {
    pub fn is_applicable(&self, kind: &Kind) -> bool {
        self.applicable_to[kind]
    }
}

#[cfg(feature = "syn")]
fn try_parse_child_parents(input: ParseStream) -> Result<Punctuated<ChildParentData, Token![,]>> {
    input.parse_terminated(|x| {
        let child_path: Punctuated<Member, Token![.]> = Punctuated::parse_separated_nonempty(x)?;
        x.parse::<Token![:]>()?;
        let ty = x.parse::<syn::Path>()?;
        Ok(ChildParentData {
            ty,
            type_hint: x.parse()?,
            actions: try_parse_child_parent_actions(x)?,
            field_path: child_path.clone(),
            field_path_str: child_path.to_token_stream().to_string().chars().filter(|c| !c.is_whitespace()).collect(),
        })
    })
}

#[cfg(feature = "syn2")]
fn try_parse_child_parents(input: ParseStream) -> Result<Punctuated<ChildParentData, Token![,]>> {
    input.parse_terminated(|x| {
        use quote::ToTokens;

        let child_path: Punctuated<Member, Token![.]> = Punctuated::parse_separated_nonempty(x)?;
        x.parse::<Token![:]>()?;
        let ty = x.parse::<syn::Path>()?;
        Ok(ChildParentData {
            ty,
            type_hint: x.parse()?,
            actions: try_parse_child_parent_actions(x)?,
            field_path: child_path.clone(),
            field_path_str: child_path.to_token_stream().to_string().chars().filter(|c| !c.is_whitespace()).collect(),
        })
    }, Token![,])
}

fn try_parse_child_parent_actions(input: ParseStream) -> Result<Punctuated<ChildParentAction, Token![,]>>{
    if !input.peek(Token![=>]) {
        return Ok(Punctuated::new());
    }

    input.parse::<Token![=>]>()?; 

    let content;
    parenthesized!(content in input);

    Ok(Punctuated::parse_terminated(&content)?)
}