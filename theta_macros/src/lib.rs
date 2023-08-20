use proc_macro::TokenStream;
use syn::{Ident, ExprClosure, LitInt, parse::Parse, LitStr, Token, parse_macro_input, Error, Expr};
use quote::{quote, quote_spanned};

struct E2ETest {
    name: Ident,
    code_fragment: LitStr,
    init: Option<ExprClosure>,
    iteration_extension: Option<E2EExtension>,
    top_of_stack: Option<Expr>,
}

impl Parse for E2ETest {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        let code_frag: LitStr = input.parse()?;
        let mut init = None;
        let mut iteration_extension = None;
        let mut top_of_stack = None;

        while !input.is_empty() {
            let k = input.parse::<Ident>()?;
            input.parse::<Token![:]>()?;

            match k.to_string().as_str() {
                "init" => {
                    init = Some(input.parse()?);
                },
                "iter" => {
                    iteration_extension = Some(input.parse()?);
                },
                "stack" => {

                },
                _ => {
                    return Err(Error::new(k.span(), "Invalid k for macro"));
                    
                },
            };

            if input.is_empty() {
                break;
            }
            input.parse::<Token![,]>()?;
        };

        Ok(E2ETest { name, code_fragment: code_frag, init, iteration_extension, top_of_stack })

    }
}

struct E2EExtension {
    iter_count: LitInt,
    iter_check: ExprClosure,
}

impl Parse for E2EExtension {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        todo!()
    }
}

#[proc_macro]
pub fn make_answer(item: TokenStream) -> TokenStream {
    let E2ETest { name, code_fragment, init, iteration_extension: extension, top_of_stack } = parse_macro_input!(item as E2ETest);

    let func_name = format!("test_{}", name);
    let func_ident = Ident::new(&func_name, name.span());

    let expanded = quote! {
        #[test]
        pub fn #func_ident() {
            
        }
    };

    TokenStream::from(expanded)
}