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

/// Load command definitions from all `.json` files in the given folder and collect them as (name, Command) pairs.
///
/// The function reads each file with a `.json` extension, parses it as a `HashMap<String, Command>`, and aggregates all entries into a single `Vec<(String, Command)>`. The function will panic if the directory cannot be read, any JSON file cannot be read, or any file contains invalid JSON.
///
/// # Returns
///
/// A vector of `(String, Command)` pairs containing every command name and its parsed `Command`.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// // `commands` is a folder containing JSON files that map command names to `Command` definitions.
/// let pairs = load_commands_from_folder(Path::new("commands"));
/// for (name, cmd) in pairs {
///     println!("{} => arity {}", name, cmd.arity);
/// }
/// ```
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

/// Generates a public enum from JSON command definitions found in a sibling `commands` directory.
///
/// The attribute macro reads all `.json` files in `<CARGO_MANIFEST_DIR>/../commands`, parses them as command
/// definitions, and emits a public enum (using the input enum's identifier) with one variant per command.
/// Variant names are derived by converting command names from kebab-case to PascalCase. The generated enum
/// derives `Debug`, `Clone`, `Copy`, `EnumString`, `Hash`, `Eq`, and `PartialEq`, and provides:
/// - `arity(&self) -> i8` returning the command's arity
/// - `desc(&self) -> &'static str` returning the command's summary (empty string when absent)
///
/// The macro will panic at compile time if the manifest directory, commands folder, or JSON parsing cannot be read or parsed.
///
/// # Examples
///
/// ```no_run
/// use fractonkv_macros::commands;
///
/// #[commands]
/// enum CommandKind {}
///
/// // After expansion the generated enum `CommandKind` has variants for each command and methods:
/// // let a = CommandKind::SomeCommand.arity();
/// // let d = CommandKind::SomeCommand.desc();
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
pub fn commands(_: TokenStream, item: TokenStream) -> TokenStream {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let folder_path = std::path::Path::new(&manifest_dir).parent().unwrap().join("commands");

    let commands = load_commands_from_folder(&folder_path);
    let mut variants = Vec::new();
    let mut arity_matches = Vec::new();
    let mut desc_matches = Vec::new();

    for (cmd_name, cmd) in commands {
        let ident_name = cmd_name.from_case(Case::Kebab).to_case(Case::Pascal);
        let ident = syn::Ident::new(&ident_name, proc_macro2::Span::call_site());

        let arity = cmd.arity;
        let desc_lit =
            syn::LitStr::new(cmd.summary.as_deref().unwrap_or(""), proc_macro2::Span::call_site());

        variants.push(quote! { #ident });
        arity_matches.push(quote! { CommandKind::#ident => #arity, });
        desc_matches.push(quote! { CommandKind::#ident => #desc_lit, });
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
