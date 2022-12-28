use std::{
    fmt::Display,
    ops::{
        Index,
        IndexMut,
    },
};

use proc_macro2::{
    Span,
    TokenStream,
};
use quote::{
    quote,
    ToTokens,
};
use syn::{
    spanned::Spanned,
    Data,
    DataStruct,
    DeriveInput,
    Error,
    Fields,
    Result,
    Type,
};

use self::attributes::HelperAttr;

mod attributes;
mod fields;

pub(super) fn derive_payload(input: DeriveInput) -> Result<TokenStream> {
    let fields = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => fields.named,
        _ => return Err(syn::Error::new(input.ident.span(), "this derive macro only works on structs with named fields")),
    };
    let target = &input.ident;

    let outer_attrs = attributes::parse(&input.attrs)?;
    let tagged = attributes::process(&outer_attrs)?.merge(fields::process(&fields)?);
    let move_impl = if tagged.valid() {
        let PayloadField {
            member: track_name,
            ty: track_type,
            ..
        } = tagged.track.unwrap();
        let PayloadField {
            member: artist_name,
            ty: artist_type,
            ..
        } = tagged.artist.unwrap();
        let (release_name, release_type) = tagged.release.map_or((quote!(::core::option::Option::None), None), |r| {
            let path = r.member;
            (quote!(listen.#path), Some(r.ty))
        });
        quote! {
            #[automatically_derived]
            impl From<#target> for ::listenbrainz::raw::request::Payload<#track_type, #artist_type, #release_type> {
                fn from(listen: #target) -> Self {
                    ::listenbrainz::raw::request::Payload {
                        listened_at: ::core::option::Option::Some(listen.listened_at()),
                        track_metadata: ::listenbrainz::raw::request::TrackMetadata {
                            additional_info: listen.track_metadata().as_ref().and_then(crate::service::additional_info),
                            track_name: listen.#track_name,
                            artist_name: listen.#artist_name,
                            release_name: #release_name,
                        },
                    }
                }
            }
        }
    } else {
        Default::default()
    };

    Ok(quote! {
        impl ::lb_importer_core::IntoPayloadDerive for #target { }
        #[automatically_derived]
        impl<'l: 'p, 'p> From<&'l #target> for ::listenbrainz::raw::request::Payload<&'p str> {
            fn from(listen: &'l #target) -> Self {
                ::listenbrainz::raw::request::Payload {
                    listened_at: ::core::option::Option::Some(listen.listened_at()),
                    track_metadata: ::listenbrainz::raw::request::TrackMetadata {
                        additional_info: listen.track_metadata().as_ref().and_then(crate::service::additional_info),
                        track_name: listen.track_name(),
                        artist_name: listen.artist_name(),
                        release_name: listen.release_name(),
                    },
                }
            }
        }
        #move_impl
    })
}


pub(self) struct PayloadField<'f> {
    pub member: &'f dyn ToTokens,
    pub ty: &'f Type,
    pub source: Option<&'f dyn Spanned>,
}

#[derive(Default)]
pub(self) struct PayloadFields<'f> {
    pub track: Option<PayloadField<'f>>,
    pub artist: Option<PayloadField<'f>>,
    pub release: Option<PayloadField<'f>>,
}

impl<'f> PayloadFields<'f> {
    fn valid(&self) -> bool { self.track.is_some() && self.artist.is_some() }

    fn assign(&mut self, fta: HelperAttr, member: &'f dyn ToTokens, ty: &'f Type, source: Option<&'f dyn Spanned>) -> Option<PayloadField<'_>> {
        self[fta].replace(PayloadField { member, ty, source })
    }

    fn try_assign(mut self, fta: HelperAttr, member: &'f dyn ToTokens, ty: &'f Type, source: Option<&'f dyn Spanned>) -> Result<Self> {
        fn ptr_eq<A: ?Sized, B: ?Sized>(a: *const A, b: *const B) -> bool { std::ptr::eq(a as *const (), b as *const ()) }

        match self.assign(fta, member, ty, source) {
            Some(PayloadField {
                member: prev_mem,
                source: prev_source,
                ..
            }) => {
                let span = source.map(Spanned::span).unwrap_or_else(Span::call_site);
                Err(if ptr_eq(member, prev_mem) {
                    const MSG: &str = "Duplicate attribute";
                    let mut e = Error::new(span, MSG);
                    if let Some(prev) = prev_source {
                        e.combine(Error::new(prev.span(), MSG));
                    }
                    e
                } else {
                    fn msg(id: impl Display) -> String { format!("Not allowed on multiple fields. Also present on '{id}'") }
                    let mut e = Error::new(span, msg(prev_mem.to_token_stream()));
                    if let Some(source) = prev_source {
                        e.combine(Error::new(source.span(), msg(member.to_token_stream())));
                    }
                    e
                })
            },
            None => Ok(self),
        }
    }

    fn merge(self, other: Self) -> Self {
        Self {
            track: other.track.or(self.track),
            artist: other.artist.or(self.artist),
            release: other.release.or(self.release),
        }
    }
}

impl<'f> Index<HelperAttr> for PayloadFields<'f> {
    type Output = Option<PayloadField<'f>>;

    fn index(&self, index: HelperAttr) -> &Self::Output {
        match index {
            HelperAttr::Track => &self.track,
            HelperAttr::Artist => &self.artist,
            HelperAttr::Release => &self.release,
            HelperAttr::Payload => unimplemented!(),
        }
    }
}
impl IndexMut<HelperAttr> for PayloadFields<'_> {
    fn index_mut(&mut self, index: HelperAttr) -> &mut Self::Output {
        match index {
            HelperAttr::Track => &mut self.track,
            HelperAttr::Artist => &mut self.artist,
            HelperAttr::Release => &mut self.release,
            HelperAttr::Payload => unimplemented!(),
        }
    }
}
