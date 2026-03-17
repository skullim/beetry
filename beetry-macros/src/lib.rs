use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    DeriveInput, Ident, Result, Token, Type, braced,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

#[proc_macro_derive(Message)]
pub fn derive_message(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    quote! {
        impl Message for #name {}
    }
    .into()
}

struct ReceiversInput {
    name: Ident,
    fields: Punctuated<Field, Token![,]>,
}

struct Field {
    ident: Ident,
    ty: Type,
}

impl Parse for ReceiversInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        let content;
        braced!(content in input);
        let fields = content.parse_terminated(Field::parse, Token![,])?;
        Ok(Self { name, fields })
    }
}

impl Parse for Field {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        Ok(Self { ident, ty })
    }
}

#[proc_macro]
pub fn receivers(input: TokenStream) -> TokenStream {
    let ReceiversInput { name, fields } = parse_macro_input!(input as ReceiversInput);

    let receiver_types: Vec<Ident> = (1..=fields.len()).map(|i| format_ident!("R{i}")).collect();
    let field_names: Vec<_> = fields.iter().map(|field| &field.ident).collect();

    let type_bounds: Vec<_> = fields
        .iter()
        .zip(&receiver_types)
        .map(|(field, receiver_ty)| {
            let ty = &field.ty;
            quote! { #receiver_ty: beetry::Receiver<#ty> }
        })
        .collect();

    let field_types = fields
        .iter()
        .zip(receiver_types.iter())
        .map(|(field, tname)| {
            let Field { ident, ty } = field;
            quote! { #ident: beetry::Input<#tname, #ty> }
        });

    let getter_methods = fields.iter().map(|field| {
        let ident = &field.ident;
        let ty = &field.ty;
        quote! {
            pub fn #ident(&mut self) -> beetry::TryRecvResult<#ty> {
                self.#ident.get()
            }
        }
    });

    quote! {
        pub struct #name<#(#receiver_types),*>
        {
            #(#field_types,)*
        }

        #[bon]
        impl<#(#receiver_types),*> #name<#(#receiver_types),*>
        where
            #(#type_bounds),*
        {
            #[builder]
            pub fn new(#(#field_names: #receiver_types),*) -> Self {
                Self {
                    #(#field_names: beetry::Input::new(#field_names),)*
                }
            }

            pub fn drain(&mut self) {
                #(self.#field_names.drain();)*
            }

            #(#getter_methods)*
        }
    }
    .into()
}
