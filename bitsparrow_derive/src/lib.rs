// The `quote!` macro requires deep recursion.
#![recursion_limit = "192"]

extern crate syn;
#[macro_use]
extern crate quote;

extern crate proc_macro;
use proc_macro::TokenStream;

use quote::Tokens;
use syn::{Body, Ident, VariantData, Variant};

fn encode_ident(ident: Ident) -> Tokens {
    quote! { BitEncode::encode(&self.#ident, e); }
}

fn encode_struct(mut body: VariantData) -> (usize, Tokens) {
    let fields = match body {
        VariantData::Struct(ref mut body) => {
            body.iter_mut()
                .map(|field| field.ident.take().unwrap())
                .map(encode_ident)
                .collect()
        },
        VariantData::Tuple(ref body) => {
            body.iter()
                .enumerate()
                .map(|(i, _)| i.to_string().into())
                .map(encode_ident)
                .collect()
        },
        VariantData::Unit => Vec::new(),
    };

    (8 * fields.len(), quote! { #( #fields )* })
}

fn encode_enum(ident: &Ident, variants: Vec<Variant>) -> (usize, Tokens) {
    let matches = variants.iter().enumerate().map(|(index, variant)| {
        quote! { #ident::#variant => BitEncode::encode(&#index, e), }
    });

    (1, quote! { match *self { #( #matches )* }; })
}

fn decode_struct(ident: &Ident, body: VariantData) -> Tokens {
    match body {
        VariantData::Struct(ref body) => {
            let fields = body
                .iter()
                .map(|field| &field.ident)
                .map(|ident| quote! { #ident: BitDecode::decode(d)?, });

            quote! { Ok(#ident{ #( #fields )* }) }
        },
        VariantData::Tuple(ref b) => {
            let fields = b.iter().map(|_| quote! { BitDecode::decode(d)? });

            quote! { Ok(#ident( #( #fields )* )) }
        },
        VariantData::Unit => quote! { Ok(#ident) }
    }
}

#[proc_macro_derive(BitEncode)]
pub fn derive_encode(input: TokenStream) -> TokenStream {
    let input = syn::parse_derive_input(&input.to_string()).unwrap();

    let ident = input.ident;

    let (size_hint, body) = match input.body {
        Body::Struct(body) => encode_struct(body),
        Body::Enum(variants) => encode_enum(&ident, variants),
    };

    let tokens = quote! {
        impl BitEncode for #ident {
            fn encode(&self, e: &mut Encoder) {
                #body
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
        Body::Struct(body) => decode_struct(&ident, body),
        Body::Enum(_variants) => unimplemented!(),
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
