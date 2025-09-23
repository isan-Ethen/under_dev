use proc_macro::TokenStream;
use quote::quote;
use std::fs::OpenOptions;
use std::io::Write;
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input, ItemFn, LitStr,
};

fn log_unimplemented_function(func_name: &str, comment: &str) {
    let out_dir = "/home/ubuntu/".to_string();
    let dest_path = std::path::Path::new(&out_dir).join("unimplemented_symbols.txt");

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&dest_path)
        .expect("Failed to open unimplemented_symbols.txt");

    if comment.is_empty() {
        writeln!(file, "{}", func_name).expect("Failed to write to file");
    } else {
        writeln!(file, "{} # {}", func_name, comment).expect("Failed to write to file");
    }
}

#[proc_macro_attribute]
pub fn unimplemented_function(attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);

    let comment = if !attr.is_empty() {
        let lit_str = parse_macro_input!(attr as LitStr);
        lit_str.value()
    } else {
        String::new()
    };

    let func_name = func.sig.ident.to_string();
    log_unimplemented_function(&func_name, &comment);

    let vis = &func.vis;
    let sig = &func.sig;
    let result = quote! {
        #[doc = #comment]
        #[unsafe(no_mangle)]
        #vis unsafe #sig {
            unimplemented!()
        }
    };

    result.into()
}

struct UnimplementedInput {
    functions: Vec<ItemFn>,
}

impl Parse for UnimplementedInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut functions = Vec::new();
        while !input.is_empty() {
            functions.push(input.parse()?);
        }
        Ok(UnimplementedInput { functions })
    }
}

#[proc_macro]
pub fn unimplemented_functions(input: TokenStream) -> TokenStream {
    let UnimplementedInput { functions } = parse_macro_input!(input as UnimplementedInput);

    let generated_functions = functions.iter().map(|func| {
        let func_name = func.sig.ident.to_string();
        log_unimplemented_function(&func_name, "");

        let vis = &func.vis;
        let sig = &func.sig;
        quote! {
            #[unsafe(no_mangle)]
            #vis unsafe #sig {
                unimplemented!()
            }
        }
    });

    let result = quote! {
        #(#generated_functions)*
    };

    result.into()
}
