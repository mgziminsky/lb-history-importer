extern crate proc_macro;

use darling::{
    Error,
    FromDeriveInput,
};
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input,
    AngleBracketedGenericArguments,
    DeriveInput,
    PathArguments,
    Type::Path,
    TypePath,
};


mod data;
use data::*;

mod helper_attr;


#[proc_macro_derive(IntoPayload, attributes(track, artist, release))]
pub fn derive_into_payload(input: TokenStream) -> TokenStream {
    let input = PayloadDerive::from_derive_input(&parse_macro_input!(input as DeriveInput)).unwrap();
    let mut errors = Error::accumulator();

    let target = &input.ident;
    let move_into = errors
        .handle(input.process_fields())
        .filter(|fields| fields.track.is_some() && fields.artist.is_some())
        .map(|ref fields| {
            let (track_name, track_type) = {
                let track = fields.track.unwrap();
                (track.ident.as_ref().unwrap(), &track.ty)
            };
            let (artist_name, artist_type) = {
                let artist = fields.artist.unwrap();
                (artist.ident.as_ref().unwrap(), &artist.ty)
            };
            let (release_name, release_type) = {
                let release = fields.release;
                (
                    release
                        .map(|f| f.ident.as_ref().unwrap())
                        .map(|i| quote!(listen.#i))
                        .unwrap_or_else(|| quote!(::core::option::Option::None)),
                    release.map(|f| type_generic(&f.ty)),
                )
            };

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
        });

    match errors.finish() {
        Ok(_) => quote! {
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
            #move_into
        }
        .into(),
        Err(e) => e.write_errors().into(),
    }
}

fn type_generic(ty: &syn::Type) -> &syn::Type {
    if let Path(TypePath { path, .. }) = ty {
        let mut gen_arg = path
            .segments
            .iter()
            .skip_while(|s| !matches!(s.arguments, syn::PathArguments::AngleBracketed(_)))
            .map(|s| &s.arguments);
        match gen_arg.next() {
            Some(PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. })) => {
                match args.first() {
                    Some(syn::GenericArgument::Type(ty)) => ty,
                    _ => ty,
                }
            }
            _ => ty,
        }
    } else {
        ty
    }
}
