use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::ffi::CString;
use syn::{DeriveInput, parse_macro_input};

pub fn derive_blob_data_macro_impl(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as DeriveInput);
    let typename = &item.ident;
    let blob_type_name = CString::new(typename.to_string()).unwrap();
    let static_name = format_ident!("BLOB_SPEC_UNIQUE_{}", typename.to_string());

    quote! {
        static mut #static_name: ::factbook_swipl::blob::BlobSpec =
            ::factbook_swipl::blob::BlobSpec::new::<#typename>(#blob_type_name);

        unsafe impl ::factbook_swipl::blob::BlobData for #typename {
            const SPEC: *mut ::factbook_swipl::blob::BlobSpec = &raw mut #static_name;
        }
    }
    .into()
}

pub fn derive_copy_blob_data_macro_impl(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as DeriveInput);
    let typename = &item.ident;
    let blob_type_name = CString::new(typename.to_string()).unwrap();
    let static_name = format_ident!("BLOB_SPEC_COPY_{}", typename.to_string());

    quote! {
        static mut #static_name: ::factbook_swipl::blob::BlobSpec =
            ::factbook_swipl::blob::BlobSpec::copy::<#typename>(#blob_type_name);

        unsafe impl ::factbook_swipl::blob::CopyBlobData for #typename {
            const SPEC: *mut ::factbook_swipl::blob::BlobSpec = &raw mut #static_name;
        }
    }
    .into()
}

pub fn derive_scoped_blob_data_macro_impl(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as DeriveInput);
    let typename = &item.ident;
    let blob_type_name = CString::new(typename.to_string()).unwrap();
    let static_name = format_ident!("BLOB_SPEC_SCOPED_{}", typename.to_string());

    quote! {
        static mut #static_name: ::factbook_swipl::blob::BlobSpec =
            ::factbook_swipl::blob::BlobSpec::scoped::<#typename>(#blob_type_name);

        unsafe impl ::factbook_swipl::blob::ScopedBlobData for #typename {
            const SPEC: *mut ::factbook_swipl::blob::BlobSpec = &raw mut #static_name;
        }
    }
    .into()
}
