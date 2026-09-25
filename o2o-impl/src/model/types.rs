use std::collections::HashMap;

use crate::model::*;

pub(crate) enum DataType<'a> {
    Struct(&'a Struct<'a>),
    Enum(&'a Enum<'a>),
}

impl<'a> DataType<'a> {
    pub fn get_ident(&'a self) -> &'a Ident {
        match self {
            DataType::Struct(s) => s.ident,
            DataType::Enum(e) => e.ident,
        }
    }

    pub fn named_fields(&'a self) -> bool {
        match self {
            DataType::Struct(s) => s.named_fields,
            DataType::Enum(_) => panic!("Method 'named_fields' is not supposed to be called in the enum context."),
        }
    }

    pub fn get_attrs(&'a self) -> &'a DataTypeAttrs {
        match self {
            DataType::Struct(s) => &s.attrs,
            DataType::Enum(e) => &e.attrs,
        }
    }

    pub fn get_members(&'a self) -> Vec<DataTypeMember<'a>> {
        match self {
            DataType::Struct(s) => s.fields.iter().map(DataTypeMember::Field).collect(),
            DataType::Enum(e) => e.variants.iter().map(DataTypeMember::Variant).collect(),
        }
    }

    pub fn get_generics(&'a self) -> &'a Generics {
        match self {
            DataType::Struct(s) => s.generics,
            DataType::Enum(e) => e.generics,
        }
    }
}

pub(crate) struct Struct<'a> {
    pub attrs: DataTypeAttrs,
    pub ident: &'a Ident,
    pub generics: &'a Generics,
    pub fields: Vec<Field>,
    pub named_fields: bool,
    pub unit: bool,
}

impl<'a> Struct<'a> {
    pub fn from_syn(node: &'a DeriveInput, data: &'a DataStruct) -> Result<Self> {
        let (attrs, bark) = get_data_type_attrs(&node.attrs)?;
        let fields = Field::multiple_from_syn(&mut Default::default(), &data.fields, bark)?;
        Ok(Struct {
            attrs,
            ident: &node.ident,
            generics: &node.generics,
            fields,
            named_fields: matches!(&data.fields, Fields::Named(_)),
            unit: matches!(&data.fields, Fields::Unit),
        })
    }
}

pub(crate) struct Enum<'a> {
    pub attrs: DataTypeAttrs,
    pub ident: &'a Ident,
    pub generics: &'a Generics,
    pub variants: Vec<Variant>,
}

impl<'a> Enum<'a> {
    pub fn from_syn(node: &'a DeriveInput, data: &'a DataEnum) -> Result<Self> {
        let (attrs, bark) = get_data_type_attrs(&node.attrs)?;
        let variants = Variant::multiple_from_syn(&data.variants, bark)?;
        Ok(Enum { attrs, ident: &node.ident, generics: &node.generics, variants })
    }
}

fn get_data_type_attrs(input: &[Attribute]) -> Result<(DataTypeAttrs, bool)> {
    let mut bark = true;

    let mut instrs: Vec<DataTypeInstruction> = vec![];
    for x in input.iter() {
        #[cfg(feature = "syn")]
        let path = &x.path;

        #[cfg(feature = "syn2")]
        let path = x.meta.path();

        if path.is_ident("doc") {
            continue;
        } else if path.is_ident("o2o") {
            x.parse_args_with(|input: ParseStream| {
                let new_instrs: Punctuated<DataTypeInstruction, Token![,]> = Punctuated::parse_terminated_with(input, |input| {
                    let instr = input.parse::<Ident>()?;
                    let p: OptionalParenthesizedTokenStream = input.parse()?;
                    parse_data_type_instruction(&instr, p.content(), true, true)
                })?;

                if new_instrs.iter().any(|x| matches!(x, DataTypeInstruction::AllowUnknown)) {
                    bark = false;
                }

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

            instrs.push(parse_data_type_instruction(instr, tokens, false, bark)?);
        }
    }

    let mut attrs = DataTypeAttrs::default();

    let mut trait_attrs_to_repeat = HashMap::<(ApplicableTo, bool), TraitAttr>::new();

    for instr in instrs {
        match instr {
            DataTypeInstruction::Map(mut trait_attr) => {
                let k = (trait_attr.applicable_to, trait_attr.fallible);

                if trait_attr.core.stop_repeat {
                    trait_attrs_to_repeat.remove(&k);
                }

                let trait_attr_to_repeat = trait_attrs_to_repeat.get_mut(&k);

                if trait_attr.core.repeat.is_some() {
                    if trait_attr_to_repeat.is_some() && !trait_attr.core.stop_repeat {
                        Err(syn::Error::new(trait_attr.core.ty.span, "Previous repeat() instruction must be terminated with 'stop_repeat'"))?
                    }

                    trait_attrs_to_repeat.insert(k, trait_attr.clone());
                } else if let Some(trait_attr_to_repeat) = &trait_attr_to_repeat {
                    trait_attr.core.merge(trait_attr_to_repeat.core.clone())?;
                }

                attrs.attrs.push(trait_attr)
            },
            DataTypeInstruction::Ghosts(attr) => attrs.ghosts_attrs.push(attr),
            DataTypeInstruction::Where(attr) => attrs.where_attrs.push(attr),
            DataTypeInstruction::ChildParents(attr) => attrs.child_parents_attrs.push(attr),
            DataTypeInstruction::AllowUnknown | DataTypeInstruction::Unrecognized => (),
            _ => attrs.error_instrs.push(instr),
        };
    }
    Ok((attrs, bark))
}

fn parse_data_type_instruction(instr: &Ident, input: TokenStream, own_instr: bool, bark: bool) -> Result<DataTypeInstruction> {
    let instr_str = &instr.to_token_stream().to_string();
    match instr_str.as_ref() {
        "allow_unknown" if own_instr => Ok(DataTypeInstruction::AllowUnknown),
        "owned_into" | "ref_into" | "into" | "from_owned" | "from_ref" | "from" | "map_owned" | "map_ref" | "map" | "owned_into_existing" | "ref_into_existing" | "into_existing" => Ok(DataTypeInstruction::Map(TraitAttr {
            core: syn::parse2(input)?,
            fallible: false,
            applicable_to: appl_to(instr_str),
        })),
        "owned_try_into" | "ref_try_into" | "try_into" | "try_from_owned" | "try_from_ref" | "try_from" | "try_map_owned" | "try_map_ref" | "try_map" | "owned_try_into_existing" | "ref_try_into_existing" | "try_into_existing" => Ok(DataTypeInstruction::Map(TraitAttr {
            core: syn::parse2(input)?,
            fallible: true,
            applicable_to: appl_to(instr_str),
        })),
        "ghosts" | "ghosts_ref" | "ghosts_owned" => Ok(DataTypeInstruction::Ghosts(GhostsAttr {
            attr: syn::parse2(input)?,
            applicable_to: appl_to_ghosts(instr_str),
        })),
        "child_parents" => Ok(DataTypeInstruction::ChildParents(syn::parse2(input)?)),
        "where_clause" => Ok(DataTypeInstruction::Where(syn::parse2(input)?)),
        "children" => Ok(DataTypeInstruction::Misnamed { instr: "children", span: instr.span(), guess_name: "child_parents", own: own_instr }),
        "ghost" if bark => Ok(DataTypeInstruction::Misnamed { instr: "ghost", span: instr.span(), guess_name: "ghosts", own: own_instr }),
        "ghost_ref" if bark => Ok(DataTypeInstruction::Misnamed { instr: "ghost_ref", span: instr.span(), guess_name: "ghosts_ref", own: own_instr }),
        "ghost_owned" if bark => Ok(DataTypeInstruction::Misnamed { instr: "ghost_owned", span: instr.span(), guess_name: "ghosts_owned", own: own_instr }),
        "child" if bark => Ok(DataTypeInstruction::Misnamed { instr: "child", span: instr.span(), guess_name: "child_parents", own: own_instr }),
        "parent" if bark => Ok(DataTypeInstruction::Misplaced { instr: "parent", span: instr.span(), own: own_instr }),
        "as_type" if bark => Ok(DataTypeInstruction::Misplaced { instr: "as_type", span: instr.span(), own: own_instr }),
        "literal" if bark => Ok(DataTypeInstruction::Misplaced { instr: "literal", span: instr.span(), own: own_instr }),
        "pattern" if bark => Ok(DataTypeInstruction::Misplaced { instr: "pattern", span: instr.span(), own: own_instr }),
        "repeat" if bark => Ok(DataTypeInstruction::Misplaced { instr: "repeat", span: instr.span(), own: own_instr }),
        "skip_repeat" if bark => Ok(DataTypeInstruction::Misplaced { instr: "skip_repeat", span: instr.span(), own: own_instr }),
        "stop_repeat" if bark => Ok(DataTypeInstruction::Misplaced { instr: "stop_repeat", span: instr.span(), own: own_instr }),
        "type_hint" if bark => Ok(DataTypeInstruction::Misplaced { instr: "type_hint", span: instr.span(), own: own_instr }),
        _ if own_instr => Ok(DataTypeInstruction::UnrecognizedWithError { instr: instr_str.clone(), span: instr.span() }),
        _ => Ok(DataTypeInstruction::Unrecognized),
    }
}