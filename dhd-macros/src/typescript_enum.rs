use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemEnum};

use crate::type_to_typescript;

#[proc_macro_attribute]
pub fn typescript_enum(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemEnum);
    let enum_name = &input.ident;
    let enum_name_str = enum_name.to_string();

    // Create a unique static name
    let static_name = quote::format_ident!("__{}_TYPE", enum_name.to_string().to_uppercase());

    // Generate TypeScript type union
    let mut ts_variants = Vec::new();

    // Check if this is a tagged enum (with serde tag)
    let mut tag_name = "type".to_string();
    let mut content_name = None;

    for attr in &input.attrs {
        if attr.path().is_ident("serde") {
            // Parse serde attributes to find tag and content
            if let Ok(syn::Meta::List(meta_list)) = &attr.meta {
                let tokens = &meta_list.tokens;
                let tokens_str = tokens.to_string();

                // Simple parsing for tag and content
                if tokens_str.contains("tag") {
                    if let Some(tag_start) = tokens_str.find("tag = \"") {
                        let tag_start = tag_start + 7;
                        if let Some(tag_end) = tokens_str[tag_start..].find("\"") {
                            tag_name = tokens_str[tag_start..tag_start + tag_end].to_string();
                        }
                    }
                }

                if tokens_str.contains("content") {
                    if let Some(content_start) = tokens_str.find("content = \"") {
                        let content_start = content_start + 11;
                        if let Some(content_end) = tokens_str[content_start..].find("\"") {
                            content_name = Some(tokens_str[content_start..content_start + content_end].to_string());
                        }
                    }
                }
            }
        }
    }

    for variant in &input.variants {
        let variant_name = &variant.ident;
        let variant_name_str = variant_name.to_string();

        match &variant.fields {
            syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                // Tuple variant with one field
                let field_type = type_to_typescript(&fields.unnamed[0].ty);

                if let Some(ref content) = content_name {
                    // Tagged with content
                    ts_variants.push(format!(
                        "{{ {}: \"{}\", {}: {} }}",
                        tag_name, variant_name_str, content, field_type
                    ));
                } else {
                    // Simple variant
                    ts_variants.push(format!("{{ {}: \"{}\" }} & {}", tag_name, variant_name_str, field_type));
                }
            }
            syn::Fields::Unit => {
                // Unit variant
                ts_variants.push(format!("\"{}\"", variant_name_str));
            }
            _ => {
                // Other cases - just use the variant name
                ts_variants.push(format!("{{ {}: \"{}\" }}", tag_name, variant_name_str));
            }
        }
    }

    let ts_type = format!(
        "export type {} = \n    | {};",
        enum_name_str,
        ts_variants.join("\n    | ")
    );

    let expanded = quote! {
        #[derive(Clone, Debug, PartialEq)]
        #input

        #[linkme::distributed_slice(crate::typescript::TYPESCRIPT_TYPES)]
        static #static_name: (&'static str, &'static str) = (stringify!(#enum_name), #ts_type);
    };

    TokenStream::from(expanded)
}