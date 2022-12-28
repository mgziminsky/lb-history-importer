use std::{
    self,
    str::FromStr,
};

use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    token,
    AngleBracketedGenericArguments,
    Field,
    GenericArgument,
    PathArguments,
    Result,
    Type,
    TypePath,
};

use super::{
    attributes::HelperAttr,
    PayloadFields,
};

pub(super) fn process(fields: &Punctuated<Field, token::Comma>) -> Result<PayloadFields<'_>> {
    let mut named = PayloadFields::default();
    fields
        .iter()
        .inspect(|&f| named.assign_implicit(f))
        .flat_map(|f| {
            f.attrs.iter().filter_map(move |a| {
                a.path
                    .get_ident()
                    .map(ToString::to_string)
                    .and_then(|i| HelperAttr::from_str(i.as_str()).ok())
                    .map(|fta| (fta, f, a))
            })
        })
        .try_fold(PayloadFields::default(), |tagged, (ha, field, a)| match ha {
            HelperAttr::Payload => Err(syn::Error::new(a.span(), "Not allowed on fields. Must be placed on outer struct")),
            _ => tagged.try_assign(ha, field.ident.as_ref().unwrap(), type_generic(&field.ty), Some(a)),
        })
        .map(|t| named.merge(t))
}


impl<'f> PayloadFields<'f> {
    fn assign_implicit(&mut self, field: &'f Field) {
        field
            .ident
            .as_ref()
            .map(ToString::to_string)
            .and_then(|i| HelperAttr::from_str(i.as_str()).ok())
            .and_then(|h| self.assign(h, field.ident.as_ref().unwrap(), type_generic(&field.ty), None));
    }
}

fn type_generic(ty: &Type) -> &Type {
    if let Type::Path(TypePath { path, .. }) = ty {
        let mut gen_arg = path
            .segments
            .iter()
            .skip_while(|s| !matches!(s.arguments, PathArguments::AngleBracketed(_)))
            .map(|s| &s.arguments);
        match gen_arg.next() {
            Some(PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. })) => match args.first() {
                Some(GenericArgument::Type(ty)) => ty,
                _ => ty,
            },
            _ => ty,
        }
    } else {
        ty
    }
}
