use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, ItemStruct, ItemEnum, ItemImpl, FnArg, Pat, Type, ReturnType, ImplItem, Visibility};


fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for (i, ch) in s.chars().enumerate() {
        if ch == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(ch.to_uppercase().next().unwrap());
            capitalize_next = false;
        } else if i == 0 {
            result.push(ch.to_lowercase().next().unwrap());
        } else {
            result.push(ch.to_lowercase().next().unwrap());
        }
    }

    result
}

#[proc_macro_attribute]
pub fn typescript_fn(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let fn_name_str = fn_name.to_string();

    // Extract parameters
    let mut params = Vec::new();
    for arg in &input.sig.inputs {
        if let FnArg::Typed(pat_type) = arg {
            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                let param_name = pat_ident.ident.to_string();
                let param_type = type_to_typescript(&pat_type.ty);
                params.push(format!("{}: {}", param_name, param_type));
            }
        }
    }

    // Extract return type
    let return_type = match &input.sig.output {
        ReturnType::Default => "void".to_string(),
        ReturnType::Type(_, ty) => type_to_typescript(ty),
    };

    // Convert snake_case to camelCase for TypeScript
    let ts_fn_name = to_camel_case(&fn_name_str);
    let params_str = params.join(", ");
    let signature = format!("export function {}({}): {};", ts_fn_name, params_str, return_type);

    // Create a unique static name to avoid conflicts
    let static_name = quote::format_ident!("__{}_TYPESCRIPT", fn_name.to_string().to_uppercase());

    let expanded = quote! {
        #input

        #[linkme::distributed_slice(crate::typescript::TYPESCRIPT_FUNCTIONS)]
        static #static_name: (&'static str, &'static str) = (stringify!(#fn_name), #signature);
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn typescript_type(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let struct_name = &input.ident;
    let struct_name_str = struct_name.to_string();

    // Create a unique static name
    let static_name = quote::format_ident!("__{}_TYPE", struct_name.to_string().to_uppercase());

    // Generate TypeScript interface
    let mut ts_fields = Vec::new();
    if let syn::Fields::Named(fields) = &input.fields {
        for field in &fields.named {
            // Only include public fields
            if matches!(field.vis, Visibility::Public(_)) {
                if let Some(field_name) = &field.ident {
                    let field_name_str = field_name.to_string();
                    let field_type = type_to_typescript(&field.ty);

                    // Check if field is optional
                    let is_optional = is_option_type(&field.ty);
                    let optional_marker = if is_optional { "?" } else { "" };

                    ts_fields.push(format!("    {}{}: {}", field_name_str, optional_marker, field_type));
                }
            }
        }
    }

    let ts_interface = format!(
        "export interface {} {{\n{}\n}}",
        struct_name_str,
        ts_fields.join(";\n")
    );

    let expanded = quote! {
        #[derive(Clone, Debug, PartialEq)]
        #input

        #[linkme::distributed_slice(crate::typescript::TYPESCRIPT_TYPES)]
        static #static_name: (&'static str, &'static str) = (stringify!(#struct_name), #ts_interface);
    };

    TokenStream::from(expanded)
}

fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

fn type_to_typescript(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                match segment.ident.to_string().as_str() {
                    "String" => "string".to_string(),
                    "bool" => "boolean".to_string(),
                    "i32" | "i64" | "u32" | "u64" | "f32" | "f64" => "number".to_string(),
                    "Vec" => {
                        // Handle Vec<T>
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                                return format!("{}[]", type_to_typescript(inner_ty));
                            }
                        }
                        "any[]".to_string()
                    },
                    "Option" => {
                        // Handle Option<T>
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                                return format!("{} | undefined", type_to_typescript(inner_ty));
                            }
                        }
                        "any | undefined".to_string()
                    },
                    "Result" => {
                        // Handle Result<T, E>
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let Some(syn::GenericArgument::Type(ok_ty)) = args.args.first() {
                                return type_to_typescript(ok_ty);
                            }
                        }
                        "any".to_string()
                    },
                    // For custom types, use the type name as-is
                    "Self" => "this".to_string(),
                    other => other.to_string(),
                }
            } else {
                "any".to_string()
            }
        },
        _ => "any".to_string(),
    }
}

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
            if let syn::Meta::List(meta_list) = &attr.meta {
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

#[proc_macro_attribute]
pub fn typescript_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemImpl);

    // Get the type name from the impl
    let type_name = match &*input.self_ty {
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                segment.ident.to_string()
            } else {
                panic!("Could not determine type name from impl");
            }
        }
        _ => panic!("typescript_impl only works with named types"),
    };

    // Create a unique static name
    let static_name = quote::format_ident!("__{}_METHODS", type_name.to_uppercase());

    // Collect method signatures
    let mut methods = Vec::new();

    for item in &input.items {
        if let ImplItem::Fn(method) = item {
            // Skip private methods and constructor
            if matches!(method.vis, Visibility::Public(_)) {
                let method_name = method.sig.ident.to_string();

                // Skip new and other constructors
                if method_name == "new" || method_name.starts_with("from_") {
                    continue;
                }

                // Convert method name to camelCase
                let ts_method_name = to_camel_case(&method_name);

                // Extract parameters (skip self)
                let mut params = Vec::new();

                for arg in &method.sig.inputs {
                    match arg {
                        FnArg::Receiver(_) => {
                            // Skip self parameter
                        }
                        FnArg::Typed(pat_type) => {
                            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                                let param_name = pat_ident.ident.to_string();
                                let param_type = type_to_typescript(&pat_type.ty);
                                params.push(format!("{}: {}", param_name, param_type));
                            }
                        }
                    }
                }

                // Extract return type
                let return_type = match &method.sig.output {
                    ReturnType::Default => "void".to_string(),
                    ReturnType::Type(_, ty) => type_to_typescript(ty),
                };

                let params_str = params.join(", ");
                let method_signature = format!("    {}({}): {}", ts_method_name, params_str, return_type);
                methods.push(method_signature);
            }
        }
    }

    if !methods.is_empty() {
        let methods_str = methods.join(";\n");
        let ts_methods = format!("_METHODS_{}:{{\n{}\n}}", type_name, methods_str);

        let expanded = quote! {
            #input

            #[linkme::distributed_slice(crate::typescript::TYPESCRIPT_METHODS)]
            static #static_name: (&'static str, &'static str) = (#type_name, #ts_methods);
        };

        TokenStream::from(expanded)
    } else {
        // If no public methods, just return the impl unchanged
        TokenStream::from(quote! { #input })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("hello_world"), "helloWorld");
        assert_eq!(to_camel_case("define_module"), "defineModule");
        assert_eq!(to_camel_case("a_b_c_d"), "aBCD");
        assert_eq!(to_camel_case("already_camel"), "alreadyCamel");
        assert_eq!(to_camel_case("single"), "single");
        assert_eq!(to_camel_case("UPPER_CASE"), "upperCase");
        assert_eq!(to_camel_case(""), "");
        assert_eq!(to_camel_case("_leading"), "Leading");
        assert_eq!(to_camel_case("trailing_"), "trailing");
        assert_eq!(to_camel_case("double__underscore"), "doubleUnderscore");
    }

    #[test]
    fn test_is_option_type() {
        use syn::parse_quote;

        let option_type: Type = parse_quote!(Option<String>);
        assert!(is_option_type(&option_type));

        let string_type: Type = parse_quote!(String);
        assert!(!is_option_type(&string_type));

        let vec_type: Type = parse_quote!(Vec<String>);
        assert!(!is_option_type(&vec_type));
    }

    #[test]
    fn test_type_to_typescript() {
        use syn::parse_quote;

        // Basic types
        let string_type: Type = parse_quote!(String);
        assert_eq!(type_to_typescript(&string_type), "string");

        let bool_type: Type = parse_quote!(bool);
        assert_eq!(type_to_typescript(&bool_type), "boolean");

        let i32_type: Type = parse_quote!(i32);
        assert_eq!(type_to_typescript(&i32_type), "number");

        let u64_type: Type = parse_quote!(u64);
        assert_eq!(type_to_typescript(&u64_type), "number");

        // Vec types
        let vec_string: Type = parse_quote!(Vec<String>);
        assert_eq!(type_to_typescript(&vec_string), "string[]");

        let vec_i32: Type = parse_quote!(Vec<i32>);
        assert_eq!(type_to_typescript(&vec_i32), "number[]");

        // Option types
        let option_string: Type = parse_quote!(Option<String>);
        assert_eq!(type_to_typescript(&option_string), "string | undefined");

        let option_bool: Type = parse_quote!(Option<bool>);
        assert_eq!(type_to_typescript(&option_bool), "boolean | undefined");

        // Result type
        let result_string: Type = parse_quote!(Result<String, Error>);
        assert_eq!(type_to_typescript(&result_string), "string");

        // Self type
        let self_type: Type = parse_quote!(Self);
        assert_eq!(type_to_typescript(&self_type), "this");

        // Custom types
        let custom_type: Type = parse_quote!(MyCustomType);
        assert_eq!(type_to_typescript(&custom_type), "MyCustomType");

        let action_type: Type = parse_quote!(ActionType);
        assert_eq!(type_to_typescript(&action_type), "ActionType");
    }
}