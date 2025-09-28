use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::quote;
use serde::{Deserialize, Serialize};
use syn::{ItemEnum, parse_macro_input};

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
struct KeySpecRange {
    lastkey: i64,
    step: i64,
    limit: i64,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
struct BeginSearchIndex {
    pos: i64,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
struct BeginSearch {
    index: BeginSearchIndex,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
struct FindKeys {
    range: KeySpecRange,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
struct KeySpec {
    flags: Vec<String>,
    begin_search: Option<BeginSearch>,
    find_keys: Option<FindKeys>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
struct ArgumentSpec {
    name: String,
    #[serde(rename = "type")]
    r#type: String,
    #[serde(default)]
    multiple: bool,
    key_spec_index: Option<i8>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
struct Command {
    summary: Option<String>,
    arity: i8,
    #[serde(default)]
    key_specs: Vec<KeySpec>,
    #[serde(default)]
    arguments: Vec<ArgumentSpec>,
}

use std::fs;
use std::path::Path;

fn load_commands_from_folder(folder_path: &Path) -> Vec<(String, Command)> {
    let mut all_commands = Vec::new();

    let entries = fs::read_dir(folder_path)
        .unwrap_or_else(|_| panic!("Failed to read commands folder: {:?}", folder_path));

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        let data = fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("Failed to read JSON file: {:?}", path));

        // Parse directly into HashMap<String, Command> instead of Value
        let obj: std::collections::HashMap<String, Command> = serde_json::from_str(&data)
            .unwrap_or_else(|_| panic!("Failed to parse JSON in file {:?}", path));

        // Extend instead of pushing one by one
        all_commands.extend(obj.into_iter());
    }

    all_commands
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn generate_command_kind(_: TokenStream, item: TokenStream) -> TokenStream {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let folder_path = std::path::Path::new(&manifest_dir).parent().unwrap().join("commands");

    let commands = load_commands_from_folder(&folder_path);
    let mut variants = Vec::new();
    let mut arity_matches = Vec::new();
    let mut desc_matches = Vec::new();

    for (cmd_name, cmd) in commands {
        let ident_name = cmd_name.to_case(Case::Pascal);
        let ident = syn::Ident::new(&ident_name, proc_macro2::Span::call_site());

        let arity = cmd.arity;
        let desc_lit =
            syn::LitStr::new(cmd.summary.as_deref().unwrap_or(""), proc_macro2::Span::call_site());

        variants.push(quote! { #ident });
        arity_matches.push(quote! { Self::#ident => #arity, });
        desc_matches.push(quote! { Self::#ident => #desc_lit, });
    }

    let input_enum = parse_macro_input!(item as ItemEnum);
    let enum_ident = &input_enum.ident;

    let expanded = quote! {
        use strum_macros::EnumString;
        #[derive(Debug, Clone, Copy,EnumString, Hash, Eq, PartialEq)]
        pub enum #enum_ident {
            #(#variants),*
        }

        impl #enum_ident {
            pub fn arity(&self) -> i8 {
                match self {
                    #(#arity_matches)*
                }
            }

            pub fn desc(&self) -> &'static str {
                match self {
                    #(#desc_matches)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
