use crate::model::*;

#[derive(Clone)]
pub(crate) struct GhostsAttr {
    pub attr: StructGhostAttrCore,
    pub applicable_to: ApplicableTo,
}

#[derive(Clone)]
pub(crate) struct StructGhostAttrCore {
    pub container_ty: Option<TypePath>,
    pub ghost_data: Punctuated<GhostData, Token![,]>,
}

impl Parse for StructGhostAttrCore {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(StructGhostAttrCore {
            container_ty: try_parse_container_ident(input, false),
            ghost_data: Punctuated::parse_terminated(input)?,
        })
    }
}

#[derive(Clone)]
pub(crate) struct GhostData {
    pub child_path: Option<ChildPath>,
    pub ghost_ident: GhostIdent,
    pub action: InlineExpression,
}

impl Parse for GhostData {
    fn parse(input: ParseStream) -> Result<Self> {
        let child_path = if !peek_ghost_field_name(input) {
            let child_path = Some(Punctuated::parse_separated_nonempty(input)?).map(|child_path| {
                let child_path_str = build_child_path_str(&child_path);
                ChildPath { child_path, child_path_str }
            });
            input.parse::<Token![@]>()?;
            child_path
        } else { None };
        let ghost_ident = if input.peek2(Token![:]) {
            GhostIdent::Member(input.parse()?)
        } else if input.peek2(Brace) {
            let ident: Ident = input.parse()?;
            let content;
            braced!(content in input);
            let destr: TokenStream = content.parse()?;
            GhostIdent::Destruction(quote!(#ident {#destr}))
        } else {
            let ident: Ident = input.parse()?;
            let content;
            parenthesized!(content in input);
            let destr: TokenStream = content.parse()?;
            GhostIdent::Destruction(quote!(#ident (#destr)))
        };

        input.parse::<Token![:]>()?;

        Ok(GhostData { child_path, ghost_ident, action: input.parse()? })
    }
}

#[derive(Clone)]
pub(crate) enum GhostIdent {
    Member(Member),
    Destruction(TokenStream),
}

impl GhostIdent {
    pub(crate) fn get_ident(&self) -> &Member {
        match self {
            GhostIdent::Member(member) => member,
            GhostIdent::Destruction(_) => unreachable!("16"),
        }
    }
}

impl GhostData {
    pub(crate) fn get_child_path_str(&self, depth: Option<usize>) -> &str {
        self.child_path.as_ref().map(|x| x.get_child_path_str(depth)).unwrap_or("")
    }
}