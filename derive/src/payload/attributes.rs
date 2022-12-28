use std::str::FromStr;

use syn::{
    parse::{
        Parse,
        ParseStream,
    },
    punctuated::Punctuated,
    Attribute,
    ExprType,
    Ident,
    Token,
};

use super::PayloadFields;

#[derive(Clone, Copy)]
pub enum HelperAttr {
    Track,
    Artist,
    Release,
    Payload,
}

impl FromStr for HelperAttr {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "track" => Ok(Self::Track),
            "artist" => Ok(Self::Artist),
            "release" => Ok(Self::Release),
            "payload" => Ok(Self::Payload),
            _ => Err(()),
        }
    }
}

impl TryFrom<&Ident> for HelperAttr {
    type Error = ();

    fn try_from(value: &Ident) -> Result<Self, Self::Error> { Self::from_str(&value.to_string()) }
}


pub(super) struct PayloadAttr {
    field_type: HelperAttr,
    id: Ident,
    _eq: Token![=],
    value: ExprType,
}

impl Parse for PayloadAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let id: Ident = input.parse()?;
        let field_type =
            HelperAttr::try_from(&id).map_err(|_| syn::Error::new(id.span(), "Invalid field name; expected `track`, `artist`, or `release`"))?;

        Ok(PayloadAttr {
            field_type,
            id,
            _eq: input.parse()?,
            value: input.parse()?,
        })
    }
}


pub(super) fn parse(attrs: &[Attribute]) -> syn::Result<Vec<PayloadAttr>> {
    attrs
        .iter()
        .filter(|a| a.path.is_ident("payload"))
        .map(|a| a.parse_args_with(Punctuated::<PayloadAttr, Token![,]>::parse_terminated))
        .try_fold(Vec::new(), |mut v, pr| pr.map(|x| v.extend(x)).and(Ok(v)))
}

pub(super) fn process(attrs: &[PayloadAttr]) -> syn::Result<PayloadFields<'_>> {
    attrs.iter().try_fold(PayloadFields::default(), |pf, a| {
        pf.try_assign(a.field_type, &a.value.expr, &a.value.ty, Some(&a.id))
    })
}
