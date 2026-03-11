use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(Message)]
pub fn derive_message(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    quote! {
        impl Message for #name {}
    }
    .into()
}

use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Ident, Result, Token, Type, braced};

struct InputMacro {
    name: Ident,
    fields: Punctuated<Field, Token![,]>,
}

struct Field {
    ident: Ident,
    _colon_token: Token![:],
    ty: Type,
}

impl Parse for InputMacro {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        let content;
        braced!(content in input);
        let fields = content.parse_terminated(Field::parse, Token![,])?;
        Ok(InputMacro { name, fields })
    }
}
impl Parse for Field {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Field {
            ident: input.parse()?,
            _colon_token: input.parse()?,
            ty: input.parse()?,
        })
    }
}

#[proc_macro]
pub fn receivers(item: TokenStream) -> TokenStream {
    let InputMacro { name, fields } = parse_macro_input!(item as InputMacro);

    // Generate dummy type names: R1, R2, R3, ...
    let type_params: Vec<Ident> = (1..=fields.len())
        .map(|i| format_ident!("R{}", i))
        .collect();

    // Used for new() fn and struct fields
    let field_names: Vec<_> = fields.iter().map(|f| &f.ident).collect();

    // For where clauses
    let receiver_type_bounds: Vec<_> = fields
        .iter()
        .zip(type_params.iter())
        .map(|(f, tname)| {
            let fty = &f.ty;
            quote! { #tname: beetry_core::Receiver<#fty> }
        })
        .collect();

    // For actual struct field types
    let input_types = fields.iter().zip(type_params.iter()).map(|(f, tname)| {
        let fname = &f.ident; // Extract outside
        let fty = &f.ty; // Extract outside
        quote! { #fname: beetry_channel::Input<#tname, #fty> }
    });

    // Generate each getter method
    let getter_methods = fields.iter().zip(type_params.iter()).map(|(f, _tname)| {
        let fname = &f.ident;
        let fty = &f.ty;
        quote! {
            pub fn #fname(&mut self) -> beetry_core::TryRecvResult<#fty> {
                self.#fname.get()
            }
        }
    });

    // Compose all code
    let output = quote! {
        pub struct #name<#(#type_params),*>
        {
            #(#input_types,)*
        }

        #[bon]
        impl<#(#type_params),*> #name<#(#type_params),*>
        where
            #(#receiver_type_bounds),*
        {
            #[builder]
            pub fn new(#(#field_names: #type_params),*) -> Self {
                Self {
                    #(#field_names: beetry_channel::Input::new(#field_names),)*
                }
            }

            pub fn drain(&mut self) {
                #(self.#field_names.drain();)*
            }

            #(#getter_methods)*
        }
    };
    output.into()
}
