#![allow(clippy::needless_borrow)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

#[proc_macro_derive(SchemaInspector)]
pub fn derive_schema_inspector(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let fields_code = match &ast.data {
        Data::Struct(data_struct) => {
            let mut field_snippets = Vec::new();
            if let Fields::Named(fields_named) = &data_struct.fields {
                for field in &fields_named.named {
                    let field_name = field.ident.as_ref().unwrap().to_string();
                    let field_type = type_to_json(&field.ty);
                    field_snippets.push(quote! {
                        map.insert(
                            #field_name.to_string(),
                            #field_type
                        );
                    });
                }
            }
            quote! { #(#field_snippets)* }
        }
        _ => quote! {},
    };

    let gen = quote! {
        impl #name {
            pub fn get_schema() -> serde_json::Value {
                let mut map = serde_json::Map::new();
                #fields_code
                serde_json::Value::Object(map)
            }
        }
    };

    gen.into()
}

fn type_to_json(ty: &Type) -> proc_macro2::TokenStream {
    match ty {
        Type::Path(type_path) => {
            let segments = &type_path.path.segments;
            let type_ident = &segments.last().unwrap().ident;
            let type_str = type_ident.to_string();

            // Check if the type is a primitive or a nested struct
            if is_primitive(&type_str) {
                quote! { serde_json::Value::String(#type_str.to_string()) }
            } else {
                quote! { #type_ident::get_schema() }
            }
        }
        _ => quote! { serde_json::Value::String("UnsupportedType".to_string()) },
    }
}

fn is_primitive(type_str: &str) -> bool {
    matches!(
        type_str,
        "bool"
            | "char"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "f32"
            | "f64"
            | "String"
    )
}
