use proc_macro::TokenStream;

mod derive_blob;
mod predicate;
mod query;

/// Implements [`Predicate`](factbook_swipl::Predicate) for a type and generates
/// the necessary `extern "C" fn` item. This does not generate predicate
/// implementations. You must implement [`Semidet`] or [`Nondet`] for
/// the type explicitly.
///
/// Each predicate can be declared as _semi-deterministic_ (`semidet`) or
/// _non-deterministic_ (`nondet`). _Deterministic_ predicates are a subset of
/// _semi-deterministic_ predicates and are trivial to implement in terms of the
/// `Semidet` trait, and are therefore left out.
///
/// * https://www.swi-prolog.org/pldoc/man?section=determinism
///
/// ```ignore
/// #[predicate(my_predicate(t1, t2) semidet)]
/// struct MyPredicate;
///
/// impl Semidet for MyPredicate {
///     // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn predicate(attr: TokenStream, item: TokenStream) -> TokenStream {
    crate::predicate::predicate_macro_impl(attr, item)
}

#[proc_macro_derive(BlobData)]
pub fn derive_blob_data(item: TokenStream) -> TokenStream {
    crate::derive_blob::derive_blob_data_macro_impl(item)
}

#[proc_macro_derive(CopyBlobData)]
pub fn derive_copy_blob_data(item: TokenStream) -> TokenStream {
    crate::derive_blob::derive_copy_blob_data_macro_impl(item)
}

#[proc_macro_derive(ScopedBlobData)]
pub fn derive_scoped_blob_data(item: TokenStream) -> TokenStream {
    crate::derive_blob::derive_scoped_blob_data_macro_impl(item)
}

#[proc_macro]
pub fn open_query(input: TokenStream) -> TokenStream {
    crate::query::open_query_macro_impl(input)
}
