use std::collections::HashMap;

use darling::{
    ast::{
        Data,
        Fields,
    },
    util::SpannedValue,
    Error,
    FromDeriveInput,
    FromField,
    Result,
};

use crate::helper_attr::HelperAttr;

pub(crate) type FieldValue = SpannedValue<PayloadDeriveField>;

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_any))]
pub(crate) struct PayloadDerive {
    pub ident: syn::Ident,
    pub data: Data<(), FieldValue>,
}

#[derive(Debug, FromField)]
#[darling(forward_attrs(track, artist, release))]
pub(crate) struct PayloadDeriveField {
    pub ident: Option<syn::Ident>,
    pub ty: syn::Type,
    pub attrs: Vec<syn::Attribute>,
}


#[derive(Debug)]
pub(crate) struct PayloadFields<'f> {
    pub track: Option<&'f FieldValue>,
    pub artist: Option<&'f FieldValue>,
    pub release: Option<&'f FieldValue>,
}


impl PayloadDerive {
    pub fn process_fields(&self) -> Result<PayloadFields> {
        #[derive(PartialEq, Eq)]
        enum Source {
            Name,
            Attr,
        }

        let mut errors = Error::accumulator();

        let mut tagged_fields: HashMap<HelperAttr, (&FieldValue, Source)> = HashMap::with_capacity(3);
        if let Data::Struct(Fields { ref fields, .. }) = self.data {
            for field in fields {
                if let Some(attr) = field.ident.as_ref().and_then(|i| HelperAttr::try_from(i).ok()) {
                    tagged_fields.entry(attr).or_insert((field, Source::Name));
                }
                field.attrs.iter().for_each(|attr| {
                    if let Some(ha) = attr.path.get_ident().and_then(|i| HelperAttr::try_from(i).ok()) {
                        // Check for duplicate attributes
                        match *tagged_fields.entry(ha).or_insert((field, Source::Attr)) {
                            (f, Source::Attr) if !std::ptr::eq(f, field) => {
                                let msg = format!(r#"#[{}] only allowed on one field"#, ha.as_str());
                                errors.push(Error::custom(&msg).with_span(&f.span()));
                                errors.push(Error::custom(&msg).with_span(&field.span()));
                            },
                            _ => { /* Replacing implicit name or same tag */ },
                        }
                    }
                });
                let count = tagged_fields
                    .values()
                    .filter(|(f, a)| *a == Source::Attr && std::ptr::eq(*f, field))
                    .count();
                if count > 1 {
                    errors.push(
                        Error::custom("Only one of track, artist, or release allowed on a single field")
                            .with_span(&field.span()),
                    );
                }
            }
        }

        errors.finish_with(PayloadFields {
            track: tagged_fields.get(&HelperAttr::Track).map(|(f, _)| *f),
            artist: tagged_fields.get(&HelperAttr::Artist).map(|(f, _)| *f),
            release: tagged_fields.get(&HelperAttr::Release).map(|(f, _)| *f),
        })
    }
}
