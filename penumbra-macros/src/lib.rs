use proc_macro::TokenStream;

mod bytes;

#[proc_macro_derive(ToBytes)]
pub fn derive_to_bytes(input: TokenStream) -> TokenStream {
    bytes::derive_to_bytes(input)
}

#[proc_macro_derive(FromBytes)]
pub fn derive_from_bytes(input: TokenStream) -> TokenStream {
    bytes::derive_from_bytes(input)
}
