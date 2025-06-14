use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Item};

/// Add this macro to all structs or enums used inside procedure arguments or return types.
/// This macro is necessary for serialization and TS type generation.
#[proc_macro_attribute]
pub fn ipc_type(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_item = parse_macro_input!(item as Item);

    let output = match input_item {
        Item::Struct(input_struct) => {
            quote! {
                #[derive(serde::Serialize, serde::Deserialize, specta::Type, Debug, Clone)]
                #input_struct
            }
        }
        Item::Enum(input_enum) => {
            quote! {
                #[derive(serde::Serialize, serde::Deserialize, specta::Type, Debug, Clone)]
                #input_enum
            }
        }
        _ => {
            return syn::Error::new_spanned(
                input_item,
                "ipc_type can only be applied to structs or enums",
            )
            .to_compile_error()
            .into();
        }
    };

    output.into()
}
