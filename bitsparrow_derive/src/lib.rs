// The `quote!` macro requires deep recursion.
#![recursion_limit = "192"]

extern crate syn;
#[macro_use]
extern crate quote;

extern crate proc_macro;
use proc_macro::TokenStream;

use syn::{Body, Ident, VariantData};

#[proc_macro_derive(BitEncode)]
pub fn derive_encode(input: TokenStream) -> TokenStream {
    let input = syn::parse_derive_input(&input.to_string()).unwrap();

    let ident = input.ident;

    let body = match input.body {
        Body::Struct(body) => body,
        _ => unimplemented!(),
    };

    let fields = body.fields().iter().enumerate().map(|(i, field)| {
        let index = Ident::from(i.to_string());
        let ident = field.ident.as_ref().unwrap_or(&index);

        quote! { BitEncode::encode(&self.#ident, e); }
    }).collect::<Vec<_>>();

    let size_hint = 8 * fields.len();

    let tokens = quote! {
        impl BitEncode for #ident {
            fn encode(&self, e: &mut Encoder) {
                #( #fields )*
            }

            #[inline]
            fn size_hint() -> usize {
                #size_hint
            }
        }

        impl<'a> BitEncode for &'a #ident {
            #[inline]
            fn encode(&self, e: &mut Encoder) {
                BitEncode::encode(*self, e)
            }

            #[inline]
            fn size_hint() -> usize {
                #size_hint
            }
        }
    };

    tokens.parse().unwrap()
}

#[proc_macro_derive(BitDecode)]
pub fn derive_decode(input: TokenStream) -> TokenStream {
    let input = syn::parse_derive_input(&input.to_string()).unwrap();

    let ident = input.ident;

    let body = match input.body {
        Body::Struct(body) => body,
        _ => unimplemented!(),
    };

    let fields = body.fields().iter().map(|field| {
        match field.ident {
            Some(ref ident) => quote! { #ident: BitDecode::decode(d)?, },
            None            => quote! { BitDecode::decode(d)?, }
        }
    });

    let body = match body {
        VariantData::Struct(..) => quote! { Ok(#ident{ #( #fields )* }) },
        VariantData::Tuple(..)  => quote! { Ok(#ident( #( #fields )* )) },
        VariantData::Unit       => quote! { Ok(#ident) }
    };

    let tokens = quote! {
        impl BitDecode for #ident {
            fn decode(d: &mut Decoder) -> Result<Self, Error> {
                #body
            }
        }
    };

    tokens.parse().unwrap()
}
