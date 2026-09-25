use std::ops::Not;

use crate::model::*;

#[derive(Clone)]
pub(crate) struct ParentAttr {
    pub container_ty: Option<TypePath>,
    pub child_fields: Option<Vec<ParentChildField>>,
}

impl Parse for ParentAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let container_ty = try_parse_container_ident(input, true);
        let child_fields: Option<Punctuated<ParentChildFieldAsParsed, Comma>> = input.is_empty().not().then(|| Punctuated::parse_terminated(input)).transpose()?;

        Ok(ParentAttr { container_ty, child_fields: child_fields.map(|x|convert_parent_child_field(x, vec![])) })
    }
}

fn convert_parent_child_field(child_fields_as_parsed: Punctuated<ParentChildFieldAsParsed, Comma>, sub_path: Vec<(Member, Option<syn::Path>)>) -> Vec<ParentChildField> {
    let mut child_fields_as_used = vec![];

    for child_field in child_fields_as_parsed {
        if let Some(parent_attr) = child_field.parent_attr {
            let mut path = sub_path.clone();
            path.push((child_field.this_member, child_field.ty));
            child_fields_as_used.extend(convert_parent_child_field(parent_attr, path));
        } else {
            let path_tokens = sub_path.iter().map(|x|x.0.to_token_stream()).fold(TokenStream::new(), |a,b| quote!(#a.#b));
            child_fields_as_used.push(ParentChildField { this_member: child_field.this_member, attrs: child_field.attrs, sub_path: sub_path.clone(), sub_path_tokens: path_tokens });
        }
    }

    child_fields_as_used
}

#[derive(Clone)]
struct ParentChildFieldAsParsed {
    pub this_member: Member,
    pub ty: Option<syn::Path>,
    pub attrs: Vec<ParentChildFieldAttr>,
    pub parent_attr: Option<Punctuated<ParentChildFieldAsParsed, Comma>>,
}

impl Parse for ParentChildFieldAsParsed {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut attrs = vec![];
        let mut parent_attr = None;

        while input.peek(Bracket) {
            let content;
            bracketed!(content in input);

            let instr = content.parse::<Ident>()?;
            let instr_str = &instr.to_string();

            let content_inner;
            parenthesized!(content_inner in content);

            match instr_str.as_ref() {
                "owned_into" | "ref_into" | "into" | "from_owned" | "from_ref" | "from" | "map_owned" | "map_ref" | "map" | "owned_into_existing" | "ref_into_existing" | "into_existing" => {
                    attrs.push(ParentChildFieldAttr { 
                        that_member: try_parse_optional_ident(&content_inner),
                        action: content_inner.is_empty().not().then(|| content_inner.parse()).transpose()?,
                        applicable_to: appl_to(instr_str)});
                },
                "parent" => {
                    if parent_attr.is_none() {
                        parent_attr = Some(Punctuated::parse_terminated(&content_inner)?)
                    } else {
                        Err(syn::Error::new(instr.span(), "Cannot have more than one [parent(...)] instruction here"))?
                    }
                }
                _ => Err(syn::Error::new(instr.span(), format!("Instruction '{}' is not recognized in this context", instr_str)))?
            }
        }

        let this_member: Member = input.parse()?;

        let ty: Option<syn::Path> = if input.peek(Token![:]) {
            input.parse::<Token![:]>()?;
            Some(input.parse()?)
        } else { None };

        Ok(ParentChildFieldAsParsed { this_member, ty, attrs, parent_attr })
    }
}

#[derive(Clone)]
pub(crate) struct ParentChildField {
    pub this_member: Member,
    pub attrs: Vec<ParentChildFieldAttr>,
    pub sub_path: Vec<(Member, Option<syn::Path>)>,
    pub sub_path_tokens: TokenStream,
}

impl<'a> ParentChildField {
    pub(crate) fn named_fields(&'a self) -> bool {
        match self.this_member {
            Member::Named(_) => true,
            Member::Unnamed(_) => false,
        }
    }

    pub(crate) fn get_for_kind(&'a self, kind: &'a Kind) -> Option<&'a ParentChildFieldAttr> {
        self.attrs.iter()
            .find(|x| x.applicable_to[kind])
            .or_else(|| if kind == &Kind::OwnedIntoExisting { self.get_for_kind(&Kind::OwnedInto) } else { None })
            .or_else(|| if kind == &Kind::RefIntoExisting { self.get_for_kind(&Kind::RefInto) } else { None })
    }
}

#[derive(Clone)]
pub(crate) struct ParentChildFieldAttr {
    pub that_member: Option<Member>,
    pub action: Option<InlineExpression>,
    pub applicable_to: ApplicableTo,
}