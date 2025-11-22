use proc_macro::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::token::Brace;
use syn::{Expr, Ident, Token, braced, parenthesized, parse_macro_input};

struct Query {
    context: Expr,
    module: QueryModule,
    predicate: Ident,
    args: Punctuated<Option<Expr>, Token![,]>,
}

#[allow(clippy::large_enum_variant)]
enum QueryModule {
    None,
    Ident(Ident),
    Expr(Expr),
}

impl Parse for Query {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let context = input.parse()?;

        input.parse::<Token![=>]>()?;

        let mut module = QueryModule::None;

        if input.peek(Ident) && input.peek2(Token![:]) {
            module = QueryModule::Ident(input.parse()?);
            input.parse::<Token![:]>()?;
        } else if input.peek(Brace) && input.peek2(Token![:]) {
            let name;
            braced!(name in input);
            module = QueryModule::Expr(name.parse()?);
            input.parse::<Token![:]>()?;
        }

        let predicate = input.parse()?;
        let args;
        parenthesized!(args in input);
        let args = args.parse_terminated(parse_expr_or_wildcard, Token![,])?;

        Ok(Self {
            context,
            module,
            predicate,
            args,
        })
    }
}

fn parse_expr_or_wildcard(input: syn::parse::ParseStream) -> syn::Result<Option<Expr>> {
    input
        .parse::<Token![_]>()
        .map(|_| None)
        .or_else(|_| input.parse().map(Some))
}

pub fn open_query_macro_impl(input: TokenStream) -> TokenStream {
    let Query {
        context,
        module,
        predicate,
        args,
    } = parse_macro_input!(input as Query);

    let predicate = predicate.to_string();
    let module = match module {
        QueryModule::None => quote!(None),
        QueryModule::Ident(ident) => ident.to_string().to_token_stream(),
        QueryModule::Expr(expr) => quote!(::std::convert::AsRef::<str>::as_ref(#expr)),
    };

    let (arg_names, arg_initializers): (Vec<_>, Vec<_>) = args
        .into_iter()
        .enumerate()
        .map(|(i, arg)| {
            let name = format_ident!("arg{i}");
            let initializer = arg.map(|value| {
                quote! {
                    #name.put(::factbook_swipl::term! { c => #value });
                }
            });
            (name, initializer)
        })
        .unzip();

    quote! {
        #context.open_query(
            #predicate,
            |c, [#(#arg_names),*]| {
                #(#arg_initializers)*
            },
            #module
        )
    }
    .into()
}
