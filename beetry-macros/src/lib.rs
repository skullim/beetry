use proc_macro::TokenStream;
use proc_macro_error::{abort, abort_call_site, proc_macro_error};
use quote::{format_ident, quote};
use syn::{DeriveInput, Lit, LitStr, parse_macro_input};

#[proc_macro_derive(Message)]
pub fn derive_message(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    quote! {
        impl Message for #name {}
    }
    .into()
}

/// Derive macro to automatically implement ProvideSchema for parameter structs.
///
/// This macro generates a ProvideSchema implementation by analyzing the struct fields
/// and creating appropriate parameter definitions based on field types.
///
/// # Supported Field Types
/// - `f32`, `f64` -> `parameter::Type::Float`
/// - `i32`, `i64`, `u32`, `u64` -> `parameter::Type::Integer`
/// - `bool` -> `parameter::Type::Boolean`
/// - `String` -> `parameter::Type::String`
///
/// # Attributes
/// - `#[param(description = "...")]` - Set field description
/// - `#[param(min = value, max = value)]` - Set bounds for numeric types
#[proc_macro_derive(ProvideSchema, attributes(param))]
#[proc_macro_error]
pub fn derive_provide_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        syn::Data::Struct(data_struct) => match &data_struct.fields {
            syn::Fields::Named(fields_named) => &fields_named.named,
            unsupported => abort!(
                unsupported,
                "can only be derived for structs with named fields"
            ),
        },
        _ => abort_call_site!("can only be derived for structs"),
    };

    let definitions = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        let field_type = &field.ty;

        let mut description: Option<LitStr> = None;
        let mut min_bound: Option<i64> = None;
        let mut max_bound: Option<i64> = None;

        for attr in &field.attrs {
            if attr.path().is_ident("param")
                && attr
                    .parse_nested_meta(|meta| {
                        if meta.path.is_ident("description") {
                            let value: LitStr = meta.value()?.parse()?;
                            description = Some(value);
                        } else if meta.path.is_ident("min") {
                            let value: Lit = meta.value()?.parse()?;
                            match value {
                                Lit::Int(lit_int) => {
                                    min_bound = Some(lit_int.base10_parse::<i64>()?);
                                }
                                Lit::Float(lit_float) => {
                                    min_bound = Some(lit_float.base10_parse::<f32>()? as i64);
                                }
                                _ => {}
                            }
                        } else if meta.path.is_ident("max") {
                            let value: Lit = meta.value()?.parse()?;
                            match value {
                                Lit::Int(lit_int) => {
                                    max_bound = Some(lit_int.base10_parse::<i64>()?);
                                }
                                Lit::Float(lit_float) => {
                                    max_bound = Some(lit_float.base10_parse::<f32>()? as i64);
                                }
                                _ => {}
                            }
                        }
                        Ok(())
                    })
                    .is_ok()
            {}
        }

        let generate_bounds = || match (min_bound, max_bound) {
            (Some(min), Some(max)) => {
                quote! { Some(Bounds::builder().min(#min).max(#max).build()) }
            }
            _ => quote! { None },
        };

        let param_type = match field_type {
            syn::Type::Path(type_path) => {
                let type_name = type_path
                    .path
                    .get_ident()
                    .map(|ident| ident.to_string())
                    .unwrap_or_default();

                match type_name.as_str() {
                    "f32" | "f64" => {
                        let bounds = generate_bounds();
                        quote! { parameter::Type::Float { bounds: #bounds } }
                    }
                    "i32" | "i64" | "u32" | "u64" | "usize" => {
                        let bounds = generate_bounds();
                        quote! { parameter::Type::Integer { bounds: #bounds } }
                    }
                    "bool" => quote! { parameter::Type::Boolean },
                    "String" => quote! { parameter::Type::String { max_length: None } },
                    unsupported => {
                        abort!(
                            unsupported,
                            "unsupported parameter type: {}: {}",
                            field_name,
                            unsupported
                        )
                    }
                }
            }
            unsupported => abort!(unsupported, "expected field type"),
        };

        // Build the definition with optional description
        let mut builder = quote! {
            parameter::Definition::builder()
                .name(#field_name_str)
                .ty(#param_type)
        };

        if let Some(desc) = description {
            builder = quote! { #builder.description(#desc) };
        }

        quote! { #builder.build() }
    });

    quote! {
        impl ProvideSchema for #name {
            fn provide() -> parameter::Schema {
                let definitions = vec![#(#definitions),*];
                Schema::new(definitions.into_iter())
            }
        }
    }
    .into()
}

/// Derive macro to automatically implement ChannelPlugin for message types.
///
/// This macro generates a ChannelPlugin implementation that:
/// - Creates a ChannelSpec using the type
/// - Creates a Factory using Factory::from_msg_type
/// - Returns the factory when requested
#[proc_macro_derive(ChannelPlugin)]
pub fn derive_channel_plugin(input: TokenStream) -> TokenStream {
    let (_, impl_tokens) = generate_channel_plugin_impl(parse_macro_input!(input as DeriveInput));
    impl_tokens
}

/// Attribute macro to generate both the channel plugin and register it.
///
/// This macro:
/// 1. Generates the ``ChannelPlugin`` implementation (like the derive macro)
/// 2. Automatically registers the plugin
#[proc_macro_attribute]
pub fn submit_as_channel_plugin(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_parsed = parse_macro_input!(input as DeriveInput);

    let (channel_name, channel_impl) = generate_channel_plugin_impl(input_parsed.clone());
    let channel_impl: proc_macro2::TokenStream = channel_impl.into();

    quote! {
        #input_parsed
        #channel_impl
        beetry_plugin::submit!(ChannelPluginConstructor::new::<#channel_name>());
    }
    .into()
}

fn generate_channel_plugin_impl(input: DeriveInput) -> (syn::Ident, TokenStream) {
    let message_type = &input.ident;
    let channel_name = syn::Ident::new(&format!("{}Channel", message_type), message_type.span());

    let expanded = quote! {
        pub struct #channel_name {
            factory: Factory,
        }

        impl Plugin for #channel_name {
            type Spec = ChannelSpec;
            type Factory = Factory;
            fn spec(&self) -> ChannelSpec {
                ChannelSpec::new::<#message_type>()
            }

            fn new() -> Self
            where
                Self: Sized,
            {
                Self {
                    factory: Factory::from_msg_type::<#message_type>(),
                }
            }

            fn factory(self: Box<Self>) -> Factory {
                self.factory
            }
        }
    };

    (channel_name, TokenStream::from(expanded))
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
    colon_token: Token![:],
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
            colon_token: input.parse()?,
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
    let getter_methods = fields.iter().zip(type_params.iter()).map(|(f, tname)| {
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
