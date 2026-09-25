use crate::model::*;

#[derive(Clone)]
pub(crate) struct ChildAttr {
    pub container_ty: Option<TypePath>,
    pub child_path: ChildPath,
}

impl ChildAttr {
    pub(crate) fn get_child_path_str(&self, depth: Option<usize>) -> &str {
        self.child_path.get_child_path_str(depth)
    }
}

impl Parse for ChildAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let container_ty = try_parse_container_ident(input, false);
        let child_path: Punctuated<Member, Token![.]> = Punctuated::parse_separated_nonempty(input)?;
        let child_path_str = build_child_path_str(&child_path);
        Ok(ChildAttr { container_ty, child_path: ChildPath { child_path, child_path_str } })
    }
}