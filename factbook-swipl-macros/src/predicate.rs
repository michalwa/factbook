use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::ffi::CString;
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::{DeriveInput, Ident, Token, parenthesized, parse_macro_input};

mod keyword {
    syn::custom_keyword!(semidet);
    syn::custom_keyword!(nondet);
}

struct PredicateAttr {
    name: Ident,
    args: Punctuated<Ident, Token![,]>,
    det: Det,
}

impl Parse for PredicateAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        let args;
        parenthesized!(args in input);
        let args = args.parse_terminated(Ident::parse, Token![,])?;
        let det = input.parse()?;

        Ok(Self { name, args, det })
    }
}

enum Det {
    Semidet,
    Nondet,
}

impl Parse for Det {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(keyword::semidet) {
            input.parse::<keyword::semidet>()?;
            Ok(Self::Semidet)
        } else if lookahead.peek(keyword::nondet) {
            input.parse::<keyword::nondet>()?;
            Ok(Self::Nondet)
        } else {
            Err(lookahead.error())
        }
    }
}

pub fn predicate_macro_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as PredicateAttr);
    let item = parse_macro_input!(item as DeriveInput);

    let typename = &item.ident;
    let arity = attr.args.len();
    let name_cstr = CString::new(attr.name.to_string()).unwrap();
    let (extern_name, extern_fn) = generate_extern_fn(&attr, &item);
    let flags = match attr.det {
        Det::Semidet => quote! { 0 },
        Det::Nondet => quote! { ::factbook_swipl::foreign::ffi::PL_FA_NONDETERMINISTIC },
    };

    quote! {
        #item

        unsafe impl ::factbook_swipl::foreign::Predicate for #typename {
            type Args<'a> = [::factbook_swipl::term::Term<'a>; #arity];

            const NAME: &'static ::std::ffi::CStr = #name_cstr;
            const EXTERN_FN: *const () = #extern_name as _;
            const FLAGS: u32 = #flags;
        }

        #extern_fn
    }
    .into()
}

fn generate_extern_fn(
    attr: &PredicateAttr,
    item: &DeriveInput,
) -> (Ident, proc_macro2::TokenStream) {
    let name = format_ident!("{}_{}", attr.name, attr.args.len());

    let typename = &item.ident;
    let params = attr.args.iter();
    let args = attr.args.iter();

    let extern_fn = match attr.det {
        Det::Semidet => quote! {
            extern "C" fn #name(
                #(#params: ::factbook_swipl::foreign::ffi::term_t),*,
            ) -> ::factbook_swipl::foreign::ffi::foreign_t {
                unsafe { ::factbook_swipl::foreign::semidet_impl::<#typename>([#(#args),*]) }
            }
        },
        Det::Nondet => quote! {
            extern "C" fn #name(
                #(#params: ::factbook_swipl::foreign::ffi::term_t),*,
                ctrl: ::factbook_swipl::foreign::ffi::control_t,
            ) -> ::factbook_swipl::foreign::ffi::foreign_t {
                unsafe { ::factbook_swipl::foreign::nondet_impl::<#typename>([#(#args),*], ctrl) }
            }
        },
    };

    (name, extern_fn)
}
