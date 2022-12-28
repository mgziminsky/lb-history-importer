extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{
    parse_macro_input,
    DeriveInput,
};

mod payload;
use payload::derive_payload;


#[proc_macro_derive(IntoPayload, attributes(track, artist, release, payload))]
pub fn derive_into_payload(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_payload(input).unwrap_or_else(|e| e.to_compile_error()).into()
}
