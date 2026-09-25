use crate::ast::SynDataTypeMember;
use crate::model::*;

#[derive(Clone, Copy)]
pub(crate) enum DataTypeMember<'a> {
    Field(&'a Field),
    Variant(&'a Variant),
}

impl<'a> DataTypeMember<'a> {
    pub fn get_attrs(&'a self) -> &'a MemberAttrs {
        match self {
            DataTypeMember::Field(f) => &f.attrs,
            DataTypeMember::Variant(v) => &v.attrs,
        }
    }

    pub fn get_span(&'a self) -> Span {
        match self {
            DataTypeMember::Field(f) => f.member.span(),
            DataTypeMember::Variant(v) => v.ident.span(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct Field {
    pub attrs: MemberAttrs,
    pub idx: usize,
    pub member: Member,
    pub member_str: String,
    pub ty: Option<Path>
}

impl<'a> Field {
    pub(super) fn multiple_from_syn(ctx: &mut Context, fields: &'a Fields, bark: bool) -> Result<Vec<Self>> {
        fields
            .iter()
            .enumerate()
            .map(move |(i, field)| {
                let mut field = Field::from_syn(i, field, bark)?;

                if field.attrs.stop_repeat {
                    ctx.field_attrs_to_repeat = None;
                }

                if let Some(repeat_attr) = &field.attrs.repeat {
                    if ctx.field_attrs_to_repeat.is_some() && !field.attrs.stop_repeat {
                        panic!("Previous #[repeat] instruction must be terminated with #[stop_repeat]")
                    }

                    ctx.field_attrs_to_repeat = Some((field.attrs.clone(), repeat_attr.permeate));
                } else if let Some(attrs_to_repeat) = &ctx.field_attrs_to_repeat {
                    field.attrs.merge(attrs_to_repeat.0.clone());
                }

                Ok(field)
            })
            .collect()
    }

    fn from_syn(idx: usize, node: &'a syn::Field, bark: bool) -> Result<Self> {
        let member = node.ident.clone().map(Member::Named).unwrap_or_else(|| {
            Member::Unnamed(Index {
                index: idx as u32,
                span: node.ty.span(),
            })
        });
        let member_str = member.to_token_stream().to_string();

        Ok(Field {
            attrs: get_member_attrs(SynDataTypeMember::Field(node), bark)?,
            idx,
            member,
            member_str,
            ty: match &node.ty {
                syn::Type::Path(p) => Some(ensure_colons(p.path.clone())),
                _ => None
            }
        })
    }
}

pub(crate) struct Variant {
    pub attrs: MemberAttrs,
    pub ident: Ident,
    _idx: usize,
    pub fields: Vec<Field>,
    pub named_fields: bool,
    pub unit: bool,
}

impl<'a> Variant {
    pub(super) fn multiple_from_syn(variants: &'a Punctuated<syn::Variant, Comma>, bark: bool) -> Result<Vec<Self>> {
        let mut ctx = Context { variant_attrs_to_repeat: None, field_attrs_to_repeat: None };

        variants
            .iter()
            .enumerate()
            .map(move |(i, variant)| {
                let mut variant = Variant::from_syn(&mut ctx, i, variant, bark)?;

                if variant.attrs.stop_repeat {
                    ctx.variant_attrs_to_repeat = None;
                }

                if variant.attrs.repeat.is_some() {
                    if ctx.variant_attrs_to_repeat.is_some() && !variant.attrs.stop_repeat {
                        panic!("Previous #[repeat] instruction must be terminated with #[stop_repeat]")
                    }

                    ctx.variant_attrs_to_repeat = Some(variant.attrs.clone());
                } else if let Some(attrs_to_repeat) = &ctx.variant_attrs_to_repeat {
                    variant.attrs.merge(attrs_to_repeat.clone());
                }

                Ok(variant)
            })
            .collect()
    }

    fn from_syn(ctx: &mut Context, i: usize, variant: &'a syn::Variant, bark: bool) -> Result<Self> {
        let fields = Field::multiple_from_syn(ctx, &variant.fields, bark)?;
        let attrs = get_member_attrs(SynDataTypeMember::Variant(variant), bark)?;

        if let Some((_, permeating)) = &ctx.field_attrs_to_repeat {
            if !permeating {
                ctx.field_attrs_to_repeat = None;
            }
        }

        Ok(Variant {
            attrs,
            ident: variant.ident.clone(),
            _idx: i,
            fields,
            named_fields: matches!(&variant.fields, Fields::Named(_)),
            unit: matches!(&variant.fields, Fields::Unit),
        })
    }
}

#[derive(Default)]
pub(super) struct Context {
    variant_attrs_to_repeat: Option<MemberAttrs>,
    field_attrs_to_repeat: Option<(MemberAttrs, bool)>,
}

fn ensure_colons(mut path: Path) -> Path {
    if let Some(segment) = path.segments.last_mut() {
        // if final segment is AngleBracketed, include `::` for ::<T> instead of <T>
        if let syn::PathArguments::AngleBracketed(ref mut args) = segment.arguments {
            args.colon2_token = Some(Default::default());
        }
    }
    path
}

fn get_member_attrs(input: SynDataTypeMember, bark: bool) -> Result<MemberAttrs> {
    let mut instrs: Vec<MemberInstruction> = vec![];
    for x in input.get_attrs().iter() {
        #[cfg(feature = "syn")]
        let path = &x.path;

        #[cfg(feature = "syn2")]
        let path = x.meta.path();

        if path.is_ident("doc") {
            continue;
        } else if path.is_ident("o2o") {
            x.parse_args_with(|input: ParseStream| {
                let new_instrs: Punctuated<MemberInstruction, Token![,]> = Punctuated::parse_terminated_with(input, |input| {
                    let instr = input.parse::<Ident>()?;
                    let p: OptionalParenthesizedTokenStream = input.parse()?;
                    parse_member_instruction(&instr, p.content(), true, true)
                })?;
                instrs.extend(new_instrs.into_iter());
                Ok(())
            })?;
        } else if let Some(instr) = path.get_ident() {
            #[cfg(feature = "syn")]
            let tokens = syn::parse2(x.tokens.clone()).map(|x: OptionalParenthesizedTokenStream|x.content())?;
            
            #[cfg(feature = "syn2")]
            let tokens = match &x.meta {
                syn2::Meta::Path(_) => TokenStream::new(),
                syn2::Meta::List(l) => l.tokens.clone(),
                syn2::Meta::NameValue(_) => Err(syn::Error::new(x.span(), "#[name = \"Value\"] syntax is not supported."))?,
            };

            instrs.push(parse_member_instruction(instr, tokens, false, bark)?);
        }
    }

    let mut attrs = MemberAttrs::default();

    for instr in instrs {
        match instr {
            MemberInstruction::Map(attr) => attrs.attrs.push(attr),
            MemberInstruction::Child(attr) => attrs.child_attrs.push(attr),
            MemberInstruction::Ghost(attr) => attrs.ghost_attrs.push(attr),
            MemberInstruction::Ghosts(attr) => attrs.ghosts_attrs.push(attr),
            MemberInstruction::Parent(attr) => attrs.parent_attrs.push(attr),
            MemberInstruction::As(attr) => {
                match input {
                    SynDataTypeMember::Field(f) => add_as_type_attrs(f, attr, &mut attrs.attrs),
                    SynDataTypeMember::Variant(_) => unreachable!("1"),
                };
            },
            MemberInstruction::Lit(attr) => attrs.lit_attrs.push(attr),
            MemberInstruction::Pat(attr) => attrs.pat_attrs.push(attr),
            MemberInstruction::Repeat(repeat_for) => attrs.repeat = Some(repeat_for),
            MemberInstruction::SkipRepeat => attrs.skip_repeat = true,
            MemberInstruction::StopRepeat => attrs.stop_repeat = true,
            MemberInstruction::VariantTypeHint(attr) => attrs.type_hint_attrs.push(attr),
            MemberInstruction::Unrecognized => (),
            _ => attrs.error_instrs.push(instr),
        };
    }
    Ok(attrs)
}

fn parse_member_instruction(instr: &Ident, input: TokenStream, own_instr: bool, bark: bool) -> Result<MemberInstruction> {
    let instr_str = &instr.to_string();
    match instr_str.as_ref() {
        "owned_into" | "ref_into" | "into" | "from_owned" | "from_ref" | "from" | "map_owned" | "map_ref" | "map" | "owned_into_existing" | "ref_into_existing" | "into_existing" => Ok(MemberInstruction::Map(MemberAttr {
            attr: syn::parse2(input)?,
            fallible: false,
            original_instr: instr_str.clone(),
            applicable_to: appl_to(instr_str),
        })),
        "owned_try_into" | "ref_try_into" | "try_into" | "try_from_owned" | "try_from_ref" | "try_from" | "try_map_owned" | "try_map_ref" | "try_map" => Ok(MemberInstruction::Map(MemberAttr {
            attr: syn::parse2(input)?,
            fallible: true,
            original_instr: instr_str.clone(),
            applicable_to: appl_to(instr_str),
        })),
        "ghost" | "ghost_ref" | "ghost_owned" => Ok(MemberInstruction::Ghost(GhostAttr {
            attr: syn::parse2(input)?,
            applicable_to: appl_to_ghost(instr_str),
        })),
        "ghosts" | "ghosts_ref" | "ghosts_owned" => Ok(MemberInstruction::Ghosts(GhostsAttr {
            attr: syn::parse2(input)?,
            applicable_to: appl_to_ghosts(instr_str),
        })),
        "child" => Ok(MemberInstruction::Child(syn::parse2(input)?)),
        "parent" => Ok(MemberInstruction::Parent(syn::parse2(input)?)),
        "as_type" => Ok(MemberInstruction::As(syn::parse2(input)?)),
        "literal" => Ok(MemberInstruction::Lit(syn::parse2(input)?)),
        "pattern" => Ok(MemberInstruction::Pat(syn::parse2(input)?)),
        "repeat" => Ok(MemberInstruction::Repeat(syn::parse2(input)?)),
        "skip_repeat" => Ok(MemberInstruction::SkipRepeat),
        "stop_repeat" => Ok(MemberInstruction::StopRepeat),
        "type_hint" => Ok(MemberInstruction::VariantTypeHint(syn::parse2(input)?)),
        "children" if bark => Ok(MemberInstruction::Misnamed { instr: "children", span: instr.span(), guess_name: "child", own: own_instr }),
        "child_parents" if bark => Ok(MemberInstruction::Misnamed { instr: "child_parents", span: instr.span(), guess_name: "child", own: own_instr }),
        "where_clause" if bark => Ok(MemberInstruction::Misplaced { instr: "where_clause", span: instr.span(), own: own_instr }),
        "allow_unknown" if bark => Ok(MemberInstruction::Misplaced { instr: "allow_unknown", span: instr.span(), own: own_instr }),
        _ if own_instr => Ok(MemberInstruction::UnrecognizedWithError { instr: instr_str.clone(), span: instr.span() }),
        _ => Ok(MemberInstruction::Unrecognized),
    }
}

fn add_as_type_attrs(input: &syn::Field, attr: AsAttr, attrs: &mut Vec<MemberAttr>) {
    let this_ty = input.ty.to_token_stream();
    let that_ty = attr.tokens;
    attrs.push(MemberAttr {
        attr: MemberAttrCore {
            container_ty: attr.container_ty.clone(),
            member: attr.member.clone(),
            action: Some(InlineExpression { expr: quote!(~ as #this_ty) }),
        },
        fallible: false,
        original_instr: "as_type".into(),
        applicable_to: [false, false, true, true, false, false],
    });
    attrs.push(MemberAttr {
        attr: MemberAttrCore {
            container_ty: attr.container_ty,
            member: attr.member,
            action: Some(InlineExpression { expr: quote!(~ as #that_ty) }),
        },
        fallible: false,
        original_instr: "as_type".into(),
        applicable_to: [true, true, false, false, true, true],
    });
}