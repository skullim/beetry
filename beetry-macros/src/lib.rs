use proc_macro::TokenStream;
use quote::quote;
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
pub fn derive_provide_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        syn::Data::Struct(data_struct) => match &data_struct.fields {
            syn::Fields::Named(fields_named) => &fields_named.named,
            _ => panic!("ProvideSchema can only be derived for structs with named fields"),
        },
        _ => panic!("ProvideSchema can only be derived for structs"),
    };

    let definitions = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        let field_type = &field.ty;

        let mut description: Option<LitStr> = None;
        let mut min_bound: Option<i32> = None;
        let mut max_bound: Option<i32> = None;

        for attr in &field.attrs {
            if attr.path().is_ident("param") {
                if let Ok(parsed) = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("description") {
                        let value: LitStr = meta.value()?.parse()?;
                        description = Some(value);
                    } else if meta.path.is_ident("min") {
                        let value: Lit = meta.value()?.parse()?;
                        match value {
                            Lit::Int(lit_int) => {
                                min_bound = Some(lit_int.base10_parse::<i32>()?);
                            }
                            Lit::Float(lit_float) => {
                                min_bound = Some(lit_float.base10_parse::<f32>()? as i32);
                            }
                            _ => {}
                        }
                    } else if meta.path.is_ident("max") {
                        let value: Lit = meta.value()?.parse()?;
                        match value {
                            Lit::Int(lit_int) => {
                                max_bound = Some(lit_int.base10_parse::<i32>()?);
                            }
                            Lit::Float(lit_float) => {
                                max_bound = Some(lit_float.base10_parse::<f32>()? as i32);
                            }
                            _ => {}
                        }
                    }
                    Ok(())
                }) {
                    let _ = parsed;
                }
            }
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
                    "i32" | "i64" | "u32" | "u64" => {
                        let bounds = generate_bounds();
                        quote! { parameter::Type::Integer { bounds: #bounds } }
                    }
                    "bool" => quote! { parameter::Type::Boolean },
                    "String" => quote! { parameter::Type::String { max_length: None } },
                    _ => quote! { parameter::Type::String { max_length: None } },
                }
            }
            _ => quote! { parameter::Type::String { max_length: None } },
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
/// - Creates a ChannelDescription using the type
/// - Creates a Factory using Factory::from_msg_type
/// - Returns the factory when requested
#[proc_macro_derive(ChannelPlugin)]
pub fn derive_channel_plugin(input: TokenStream) -> TokenStream {
    let (_, impl_tokens) = generate_channel_plugin_impl(parse_macro_input!(input as DeriveInput));
    impl_tokens
}

/// Attribute macro to generate both the channel plugin and register it with inventory.
///
/// This macro:
/// 1. Generates the ChannelPlugin implementation (like the derive macro)
/// 2. Automatically registers it with inventory
#[proc_macro_attribute]
pub fn submit_as_channel_plugin(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_parsed = parse_macro_input!(input as DeriveInput);

    let (channel_name, channel_impl) = generate_channel_plugin_impl(input_parsed.clone());
    let channel_impl: proc_macro2::TokenStream = channel_impl.into();

    quote! {
        #input_parsed
        #channel_impl
        inventory::submit!(ChannelPluginConstructor::new::<#channel_name>());
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

        impl ChannelPlugin for #channel_name {
            fn desc(&self) -> ChannelDescription {
                ChannelDescription::new::<#message_type>()
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
