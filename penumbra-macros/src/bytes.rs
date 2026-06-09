use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

pub fn derive_to_bytes(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics crate::core::ToBytes for #name #ty_generics #where_clause {
            const SIZE: usize = size_of::<Self>();
            type Output = [u8; size_of::<Self>()];

            fn to_bytes(&self) -> Self::Output {
                let mut buf = [0u8; size_of::<Self>()];
                wincode::serialize_into(&mut buf[..], self).unwrap();
                buf
            }
        }
    };
    TokenStream::from(expanded)
}

pub fn derive_from_bytes(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics crate::core::FromBytes for #name #ty_generics #where_clause {
            const SIZE: usize = size_of::<Self>();

            fn from_bytes(raw: &[u8]) -> Option<Self> {
                if raw.len() < Self::SIZE {
                    return None;
                }
                Self::deserialize(raw).ok()
            }
        }
    };
    TokenStream::from(expanded)
}
