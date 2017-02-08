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
        let varident = variant.ident.clone();

        match variant.data {
            VariantData::Struct(ref body) => {
                let mut refs = Vec::new();

                let fields: Vec<_> = body
                    .iter()
                    .map(|field| field.ident.clone())
                    .map(|ident| {
                        refs.push(quote! { ref #ident });

                        quote! { BitEncode::encode(#ident, e); }
                    })
                    .collect();

                quote! {
                    #ident::#varident {#( #refs ),*} => {
                        BitEncode::encode(&#index, e);
                        #( #fields )*
                    },
                }
            },
            VariantData::Tuple(ref body) => {
                let mut refs = Vec::new();

                let fields: Vec<_> = body
                    .iter()
                    .enumerate()
                    .map(|(i, _)| Ident::from(format!("ref{}", i)))
                    .map(|ident| {
                        refs.push(quote! { ref #ident });

                        quote! { BitEncode::encode(#ident, e); }
                    })
                    .collect();

                quote! {
                    #ident::#varident(#( #refs ),*) => {
                        BitEncode::encode(&#index, e);
                        #( #fields )*
                    },
                }
            },
            VariantData::Unit => quote! {
                #ident::#varident => BitEncode::encode(&#index, e),
            },
        }
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

            quote! { #ident{ #( #fields )* } }
        },
        VariantData::Tuple(ref body) => {
            let fields = body.iter().map(|_| quote! { BitDecode::decode(d)? });

            quote! { #ident( #( #fields )* ) }
        },
        VariantData::Unit => quote! { #ident }
    }
}

fn decode_enum(ident: &Ident, variants: Vec<Variant>) -> Tokens {
    let matches = variants.into_iter().enumerate().map(|(index, variant)| {
        let varident = variant.ident.clone();
        let varstruct = decode_struct(&varident, variant.data);

        quote! { #index => #ident::#varstruct, }
    });

    quote! {
        match BitDecode::decode(d)? {
            #( #matches )*
            _ => return Err(Error::InvalidData)
        }
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
        Body::Enum(variants) => decode_enum(&ident, variants),
    };

    let tokens = quote! {
        impl BitDecode for #ident {
            fn decode(d: &mut Decoder) -> Result<Self, Error> {
                Ok(#body)
            }
        }
    };

    tokens.parse().unwrap()
}
