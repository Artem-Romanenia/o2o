use crate::{kw, model::*};

#[derive(Clone)]
pub(crate) struct MemberRepeatAttr {
    pub permeate: bool,
    pub repeat_for: MemberRepeatFor,
}

impl Parse for MemberRepeatAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let permeate = input.peek(kw::permeate);
        if permeate {
            input.parse::<kw::permeate>()?;
            let _content;
            parenthesized!(_content in input);
        }

        if permeate && !input.is_empty() {
            input.parse::<Token![,]>()?;
        }

        let types: Punctuated<Ident, Token![,]> = Punctuated::parse_terminated(input)?;
        if types.is_empty() {
            return Ok(MemberRepeatAttr { permeate, repeat_for: [true, true, true, true, true] });
        }

        let mut repeat_for: MemberRepeatFor = [false, false, false, false, false];

        for ty in types {
            let str = ty.to_token_stream().to_string();

            match MEMBER_REPEAT_TYPES.iter().position(|x| *x == str.as_str()) {
                Some(idx) => repeat_for[idx] = true,
                None => return Err(Error::new(ty.span(), format!("#[repeat] of instruction type '{}' is not supported. Supported types are: {}", str, MEMBER_REPEAT_TYPES.join(", ")))),
            };
        }

        Ok(MemberRepeatAttr { permeate, repeat_for })
    }
}