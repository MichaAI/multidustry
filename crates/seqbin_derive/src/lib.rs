extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, GenericParam, parse_macro_input, spanned::Spanned};

fn add_seqbin_bounds(mut generics: syn::Generics) -> syn::Generics {
    for gp in generics.params.iter_mut() {
        if let GenericParam::Type(tp) = gp {
            tp.bounds.push(syn::parse_quote!(SeqBin));
        }
    }
    generics
}

#[proc_macro_derive(SeqBin, attributes(packet_id))]
pub fn seqbin_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let gen_with_bounds = add_seqbin_bounds(input.generics.clone());
    let (impl_generics, ty_generics, where_clause) = gen_with_bounds.split_for_impl();

    let write_stmts;
    let read_bindings_and_construct;

    match input.data {
        Data::Struct(ds) => match &ds.fields {
            Fields::Named(fields_named) => {
                let write_tokens = fields_named.named.iter().map(|f| {
                    let ident = &f.ident;
                    quote! { SeqBin::write_to(&self.#ident, writer)?; }
                });

                let read_lets = fields_named.named.iter().map(|f| {
                    let ident = &f.ident;
                    let ty = &f.ty;
                    quote! { let #ident: #ty = <#ty as SeqBin>::read_from(reader)?; }
                });

                let read_names = fields_named.named.iter().map(|f| &f.ident);

                write_stmts = quote! { #(#write_tokens)* };
                read_bindings_and_construct = quote! {
                    #(#read_lets)*
                    Ok(Self { #(#read_names),* })
                };
            }
            Fields::Unnamed(fields_unnamed) => {
                let mut idx = 0usize;
                let write_tokens = fields_unnamed.unnamed.iter().map(|_| {
                    let i = syn::Index::from(idx);
                    idx += 1;
                    quote! { SeqBin::write_to(&self.#i, writer)?; }
                });

                let mut idx = 0usize;
                let read_lets = fields_unnamed.unnamed.iter().map(|f| {
                    let ty = &f.ty;
                    let ident = format_ident!("_field{}", idx);
                    idx += 1;
                    quote! { let #ident: #ty = <#ty as SeqBin>::read_from(reader)?; }
                });

                let construct_fields =
                    (0..fields_unnamed.unnamed.len()).map(|i| format_ident!("_field{}", i));

                write_stmts = quote! { #(#write_tokens)* };
                read_bindings_and_construct = quote! {
                    #(#read_lets)*
                    Ok(Self(#(#construct_fields),*))
                };
            }
            Fields::Unit => {
                write_stmts = quote! {};
                read_bindings_and_construct = quote! { Ok(Self) };
            }
        },
        _ => {
            return syn::Error::new(input.span(), "SeqBin can only be derived for structs")
                .to_compile_error()
                .into();
        }
    }

    let expanded = quote! {
        impl #impl_generics SeqBin for #name #ty_generics #where_clause {
            fn write_to<W: std::io::Write>(&self, writer: &mut W) -> Result<(), SeqBinError> {
                #write_stmts
                Ok(())
            }

            fn read_from<R: std::io::Read>(reader: &mut R) -> Result<Self, SeqBinError> {
                #read_bindings_and_construct
            }
        }
    };

    TokenStream::from(expanded)
}
