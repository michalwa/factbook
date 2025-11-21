use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use std::ffi::CString;
use syn::{DeriveInput, Ident, parse_macro_input};

pub fn derive_blob_data_macro_impl(item: TokenStream) -> TokenStream {
    derive_blob_generic_macro_impl(item, "UNIQUE", "new", "BlobData")
}

pub fn derive_copy_blob_data_macro_impl(item: TokenStream) -> TokenStream {
    derive_blob_generic_macro_impl(item, "COPY", "copy", "CopyBlobData")
}

pub fn derive_scoped_blob_data_macro_impl(item: TokenStream) -> TokenStream {
    derive_blob_generic_macro_impl(item, "SCOPED", "scoped", "ScopedBlobData")
}

fn derive_blob_generic_macro_impl(
    item: TokenStream,
    static_prefix: &'static str,
    blob_spec_constructor: &'static str,
    trait_name: &'static str,
) -> TokenStream {
    let item = parse_macro_input!(item as DeriveInput);
    let typename = &item.ident;
    let blob_type_name = CString::new(typename.to_string()).unwrap();
    let static_name = format_ident!("BLOB_SPEC_{static_prefix}_{}", typename.to_string());
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    let blob_spec_constructor = Ident::new(blob_spec_constructor, Span::call_site());
    let trait_name = Ident::new(trait_name, Span::call_site());

    quote! {
        static mut #static_name: ::factbook_swipl::blob::BlobSpec =
            ::factbook_swipl::blob::BlobSpec::#blob_spec_constructor::<#typename>(#blob_type_name);

        unsafe impl #impl_generics ::factbook_swipl::blob::#trait_name for #typename #ty_generics
            #where_clause
        {
            const SPEC: *mut ::factbook_swipl::blob::BlobSpec = &raw mut #static_name;
        }
    }
    .into()
}
