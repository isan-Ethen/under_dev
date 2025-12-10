use proc_macro::TokenStream;
use quote::quote;
use std::fs::OpenOptions;
use std::io::Write;
use syn::{
    Expr, Ident, ItemFn, Lit, Token,
    parse::{Parse, ParseStream, Parser, Result},
    parse_macro_input,
    punctuated::Punctuated,
};

fn log_unimplemented_function(func_name: &str, comment: &str, file_info: &str) {
    let out_dir = match std::env::var("OUT_DIR") {
        Ok(dir) => dir,
        Err(_) => {
            eprintln!("Warning: OUT_DIR not set. Cannot write to unimplemented_symbols.txt");
            return;
        }
    };
    let dest_path = std::path::Path::new(&out_dir).join("under_devs.txt");

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&dest_path)
        .expect("Failed to open unimplemented_symbols.txt");

    if comment.is_empty() {
        writeln!(file, "{} ({})", func_name, file_info).expect("Failed to write to file");
    } else {
        writeln!(file, "{} # {} ({})", func_name, comment, file_info)
            .expect("Failed to write to file");
    }
}

#[proc_macro_attribute]
pub fn wip(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args_result = Punctuated::<Expr, Token![,]>::parse_terminated.parse(attr);
    let args = match args_result {
        Ok(args) => args,
        Err(e) => return e.to_compile_error().into(),
    };

    let mut func = parse_macro_input!(item as ItemFn);
    let mut comment = String::new();
    let mut is_ffi = false;

    for arg in args {
        match arg {
            Expr::Lit(expr_lit) => {
                if let Lit::Str(lit_str) = &expr_lit.lit {
                    comment = lit_str.value();
                }
            }
            Expr::Assign(expr_assign) => {
                let left = &expr_assign.left;
                let right = &expr_assign.right;

                if let Expr::Path(expr_path) = &**left {
                    if expr_path.path.is_ident("ffi") {
                        if let Expr::Lit(expr_lit) = &**right {
                            if let Lit::Bool(lit_bool) = &expr_lit.lit {
                                is_ffi = lit_bool.value;
                            } else {
                                return syn::Error::new_spanned(
                                    right,
                                    "Expected boolean literal (true or false) for ffi",
                                )
                                .to_compile_error()
                                .into();
                            }
                        } else {
                            return syn::Error::new_spanned(
                                right,
                                "Expected boolean literal (true or false) for ffi",
                            )
                            .to_compile_error()
                            .into();
                        }
                    }
                }
            }
            _ => {
                return syn::Error::new_spanned(
                    arg,
                    "Unsupported attribute argument. Use \"comment\" or ffi = true/false",
                )
                .to_compile_error()
                .into();
            }
        }
    }

    let func_name = func.sig.ident.to_string();
    let func_ident = &func.sig.ident;
    let span = func_ident.span();
    let start = span.start();
    let file_info = format!("{}:line {}:col {}", span.file(), start.line, start.column);

    log_unimplemented_function(&func_name, &comment, &file_info);

    let vis = &func.vis;
    let sig = &func.sig;

    let result = if is_ffi {
        func.sig.abi = None;
        func.sig.unsafety = None;
        let cleaned_sig = &func.sig;

        quote! {
            #[doc = #comment]
            #[unsafe(no_mangle)]
            #vis unsafe extern "C" #cleaned_sig {
                unimplemented!()
            }
        }
    } else {
        quote! {
            #[doc = #comment]
            #vis #sig {
                unimplemented!()
            }
        }
    };

    result.into()
}

struct UnimplementedInput {
    is_ffi: bool,
    functions: Vec<ItemFn>,
}

impl Parse for UnimplementedInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut is_ffi = false;

        if input.peek(Ident) && input.peek2(Token![=]) {
            let ident: Ident = input.parse()?;
            if ident == "ffi" {
                input.parse::<Token![=]>()?;
                if let Lit::Bool(lit_bool) = input.parse::<Lit>()? {
                    is_ffi = lit_bool.value;
                } else {
                    return Err(syn::Error::new(
                        ident.span(),
                        "Expected boolean literal (true or false) after 'ffi ='",
                    ));
                }

                if input.peek(Token![,]) {
                    input.parse::<Token![,]>()?;
                }
            } else {
                return Err(syn::Error::new(
                    ident.span(),
                    "Expected 'ffi = true' or function definitions",
                ));
            }
        }

        let mut functions = Vec::new();
        while !input.is_empty() {
            functions.push(input.parse()?);
        }

        Ok(UnimplementedInput { is_ffi, functions })
    }
}

#[proc_macro]
pub fn unimplemented_functions(input: TokenStream) -> TokenStream {
    let UnimplementedInput { is_ffi, functions } = parse_macro_input!(input as UnimplementedInput);

    let generated_functions = functions.iter().map(|func| {
        let func_name = func.sig.ident.to_string();
        let func_ident = &func.sig.ident;
        let span = func_ident.span();
        let start = span.start();
        let file_info = format!("{}:line {}:col {}", span.file(), start.line, start.column);
        log_unimplemented_function(&func_name, "", &file_info);

        let vis = &func.vis;
        let mut sig = func.sig.clone();

        if is_ffi {
            sig.abi = None;
            sig.unsafety = None;

            quote! {
                #[unsafe(no_mangle)]
                #vis unsafe extern "C" #sig {
                    unimplemented!()
                }
            }
        } else {
            quote! {
                #vis #sig {
                    unimplemented!()
                }
            }
        }
    });

    let result = quote! {
        #(#generated_functions)*
    };

    result.into()
}
