use crate::Atom;
use crate::term::{FromTerm, Term, ToTerm};
pub use factbook_swipl_macros::{BlobData, CopyBlobData, ScopedBlobData};
use std::ffi::{CStr, CString};
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::sync::{Arc, Weak};
use swipl_fli as pl;

/// A static set of metadata used by Prolog to distinguish blob types and call
/// lifecycle callbacks.
#[repr(transparent)]
pub struct BlobSpec(pl::PL_blob_t);

impl BlobSpec {
    pub const fn new<T: BlobData>(name: &'static CStr) -> Self {
        Self(pl::PL_blob_t {
            magic: pl::PL_BLOB_MAGIC as _,
            flags: pl::PL_BLOB_NOCOPY as _,
            name: name.as_ptr(),
            release: Some(blob_release::<T>),
            write: Some(blob_write::<T>),
            ..Self::zeroed()
        })
    }

    pub const fn copy<T: CopyBlobData>(name: &'static CStr) -> Self {
        Self(pl::PL_blob_t {
            magic: pl::PL_BLOB_MAGIC as _,
            name: name.as_ptr(),
            write: Some(copy_blob_write::<T>),
            ..Self::zeroed()
        })
    }

    pub const fn scoped<T: ScopedBlobData>(name: &'static CStr) -> Self {
        // No `release`, because we only want to drop it when we `ScopedBlob` is
        // dropped, not when the references are dropped
        Self(pl::PL_blob_t {
            magic: pl::PL_BLOB_MAGIC as _,
            flags: pl::PL_BLOB_NOCOPY as _,
            name: name.as_ptr(),
            release: Some(scoped_blob_release::<T>),
            write: Some(scoped_blob_write::<T>),
            ..Self::zeroed()
        })
    }

    const fn zeroed() -> pl::PL_blob_t {
        unsafe { MaybeUninit::<pl::PL_blob_t>::zeroed().assume_init() }
    }
}

/// A non-copyable blob which owns `T`
#[derive(Debug, Clone)]
pub struct Blob<T: BlobData>(Box<T>);

/// A reference to a value stored in a [`Blob`]. It lives as long as the atom
/// holding the blob.
#[derive(Debug, Clone, PartialEq)]
pub struct BlobRef<'a, T: BlobData>(&'a T);

/// A copyable blob which owns `T`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CopyBlob<T: CopyBlobData>(pub T);

/// A non-copyable blob which borrows non-`'static` data for the duration of its
/// lifetime. The de-facto way to borrow data into the Prolog runtime. It may be
/// put in a term by reference. While it's held, shared references to
/// the data may obtained via [`Atom::scoped_blob`]. Unlike a [`Blob`], the
/// inner type is dropped when the last handle is dropped, not when the blob is
/// released by Prolog.
///
/// It may be shared between threads, provided the inner type is `Send + Sync`.
/// References become invalid after this handle is dropped and calling
/// [`Atom::scoped_blob`] will return `None`.
///
/// To satisfy the lifetime constraints, there must be no live references to the
/// blob when the handle is dropped and the inner type's lifetime ends. Dropping
/// this handle while the blob is borrowed in another thread will panic.
pub struct ScopedBlob<T: ScopedBlobData>(
    /// The blob handle holds a strong `Arc`, while all Prolog instances (terms
    /// referencing this blob) hold a `Weak` raw pointer, until they are
    /// upgraded to a [`ScopedBlobRef`]
    MaybeUninit<Arc<T>>,
);

/// A reference to a [`ScopedBlob`]
///
/// This does not free the allocation and does not require to be an exclusive
/// reference on drop. It must not be kept alive when the [`ScopedBlob`] is
/// dropped. It should generally only be held within the scope of a foreign
/// predicate call.
pub struct ScopedBlobRef<T: ScopedBlobData>(Arc<T>);

impl<T: BlobData> Blob<T> {
    pub fn new(value: T) -> Self {
        Self(Box::new(value))
    }
}

impl<T: BlobData> Deref for BlobRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<T: ScopedBlobData> ScopedBlob<T> {
    pub fn new(value: T) -> Self {
        Self(MaybeUninit::new(Arc::new(value)))
    }

    pub fn into_inner(self) -> Option<T> {
        // SAFETY: Always init until after `ScopedBlob` is dropped
        let arc = unsafe { self.0.assume_init_read() };
        std::mem::forget(self);
        Arc::into_inner(arc)
    }
}

impl<T: ScopedBlobData> Drop for ScopedBlob<T> {
    fn drop(&mut self) {
        // SAFETY: Always init until after `ScopedBlob` is dropped
        Arc::into_inner(unsafe { self.0.assume_init_read() })
            .expect("ScopedBlob borrowed while dropped");
    }
}

impl<T: ScopedBlobData> Deref for ScopedBlobRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A type which can be stored in a [`Blob`]. Its not copied within Prolog and
/// its destructor gets called when the blob is freed. The type must be
/// `'static` because its memory is managed by Prolog.
///
/// # Safety
/// `SPEC` must be a valid pointer to a static `BlobSpec` instance for that
/// type.
pub unsafe trait BlobData: Sized + std::fmt::Debug + 'static {
    const SPEC: *mut BlobSpec;
}

/// A type which can be stored in a [`CopyBlob`]. Its destructor is not called
/// when the blob is freed. The type must be `'static` because its memory is
/// managed by Prolog.
///
/// # Safety
/// `SPEC` must be a valid pointer to a static `BlobSpec` instance for that
/// type.
pub unsafe trait CopyBlobData: Copy + std::fmt::Debug + 'static {
    const SPEC: *mut BlobSpec;
}

/// A type which can be borrowed via a [`ScopedBlob`]. Its destructor gets
/// called when the blob is freed.
///
/// # Safety
/// `SPEC` must be a valid pointer to a static `BlobSpec` instance for that
/// type.
pub unsafe trait ScopedBlobData {
    const SPEC: *mut BlobSpec;
}

impl<T: BlobData> ToTerm for Blob<T> {
    fn put_in(self, term: Term) {
        assert!(unsafe { put_blob(term, Box::leak(self.0) as _, T::SPEC) })
    }

    fn unify_with(self, term: Term) -> bool {
        unsafe { unify_blob(term, Box::leak(self.0) as _, T::SPEC) }
    }
}

impl<T: CopyBlobData> ToTerm for CopyBlob<T> {
    fn put_in(self, term: Term) {
        assert!(unsafe { put_blob(term, &raw const self.0, T::SPEC) });
    }

    fn unify_with(self, term: Term) -> bool {
        unsafe { unify_blob(term, &raw const self.0, T::SPEC) }
    }
}

impl<T: ScopedBlobData> ToTerm for &ScopedBlob<T> {
    fn put_in(self, term: Term) {
        assert!(unsafe {
            put_blob(
                term,
                // SAFETY: Always init until after `ScopedBlob` is dropped
                Arc::downgrade(self.0.assume_init_ref()).into_raw(),
                T::SPEC,
            )
        });
    }

    fn unify_with(self, term: Term) -> bool {
        unsafe {
            unify_blob(
                term,
                // SAFETY: Always init until after `ScopedBlob` is dropped
                Arc::downgrade(self.0.assume_init_ref()).into_raw(),
                T::SPEC,
            )
        }
    }
}

unsafe fn put_blob<T>(term: Term, ptr: *const T, spec: *mut BlobSpec) -> bool {
    unsafe {
        pl::PL_put_blob(
            term.ptr.get(),
            ptr as _,
            std::mem::size_of::<T>(),
            spec as _,
        )
    }
}

unsafe fn unify_blob<T>(term: Term, ptr: *const T, spec: *mut BlobSpec) -> bool {
    unsafe {
        pl::PL_unify_blob(
            term.ptr.get(),
            ptr as _,
            std::mem::size_of::<T>(),
            spec as _,
        )
    }
}

impl<T: CopyBlobData> FromTerm for CopyBlob<T> {
    fn from_term(term: Term) -> Option<Self> {
        let mut blob_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
        let mut spec: *mut pl::PL_blob_t = std::ptr::null_mut();

        if unsafe {
            pl::PL_get_blob(
                term.ptr.get(),
                &raw mut blob_ptr,
                std::ptr::null_mut(),
                &raw mut spec,
            )
        } && std::ptr::eq(T::SPEC, spec as _)
        {
            Some(Self(unsafe { *(blob_ptr as *const T) }))
        } else {
            None
        }
    }
}

impl Atom {
    /// Borrows the blob stored in the atom, or returns `None` if a blob of that
    /// type is not stored in the atom.
    pub fn blob<'a, T: BlobData>(&'a self) -> Option<BlobRef<'a, T>> {
        let mut spec: *mut pl::PL_blob_t = std::ptr::null_mut();
        let blob_ptr: *mut std::ffi::c_void =
            unsafe { pl::PL_blob_data(self.ptr, std::ptr::null_mut(), &raw mut spec) };

        if std::ptr::eq(T::SPEC, spec as _) {
            Some(BlobRef(unsafe { &*(blob_ptr as *const T) }))
        } else {
            None
        }
    }

    pub fn scoped_blob<T: ScopedBlobData>(&self) -> Option<ScopedBlobRef<T>> {
        let mut spec: *mut pl::PL_blob_t = std::ptr::null_mut();
        let blob_ptr = unsafe { pl::PL_blob_data(self.ptr, std::ptr::null_mut(), &raw mut spec) };

        if std::ptr::eq(T::SPEC, spec as _) {
            unsafe { upgrade_raw_weak(blob_ptr as *const T) }.map(ScopedBlobRef)
        } else {
            None
        }
    }
}

extern "C" fn blob_write<T: BlobData>(
    stream: *mut pl::IOSTREAM,
    atom: pl::atom_t,
    _flags: std::ffi::c_int,
) -> std::ffi::c_int {
    let blob_ptr = unsafe { pl::PL_blob_data(atom, std::ptr::null_mut(), std::ptr::null_mut()) };
    let blob = unsafe { &*(blob_ptr as *const T) };

    let string = CString::new(format!("<{blob:?}>")).unwrap();
    unsafe { pl::Sfputs(string.as_ptr(), stream) };

    pl::TRUE as _
}

extern "C" fn blob_release<T>(atom: pl::atom_t) -> std::ffi::c_int {
    let blob = unsafe { pl::PL_blob_data(atom, std::ptr::null_mut(), std::ptr::null_mut()) };
    let _boxed = unsafe { Box::from_raw(blob as *mut T) };

    pl::TRUE as _
}

extern "C" fn copy_blob_write<T: CopyBlobData>(
    stream: *mut pl::IOSTREAM,
    atom: pl::atom_t,
    _flags: std::ffi::c_int,
) -> std::ffi::c_int {
    let blob_ptr = unsafe { pl::PL_blob_data(atom, std::ptr::null_mut(), std::ptr::null_mut()) };
    let blob = unsafe { *(blob_ptr as *const T) };

    let string = CString::new(format!("<{blob:?}>")).unwrap();
    unsafe { pl::Sfputs(string.as_ptr(), stream) };

    pl::TRUE as _
}

extern "C" fn scoped_blob_release<T>(atom: pl::atom_t) -> std::ffi::c_int {
    let blob = unsafe { pl::PL_blob_data(atom, std::ptr::null_mut(), std::ptr::null_mut()) };
    // Only drop the `Weak` when the atom is being released
    let _weak = unsafe { Weak::from_raw(blob as *const T) };

    pl::TRUE as _
}

extern "C" fn scoped_blob_write<T>(
    stream: *mut pl::IOSTREAM,
    atom: pl::atom_t,
    _flags: std::ffi::c_int,
) -> std::ffi::c_int {
    let mut spec: *mut pl::PL_blob_t = std::ptr::null_mut();
    let blob_ptr = unsafe { pl::PL_blob_data(atom, std::ptr::null_mut(), &raw mut spec) };
    let type_name = unsafe { CStr::from_ptr((&*spec).name) }.to_str().unwrap();

    let valid = unsafe { upgrade_raw_weak(blob_ptr as *const T) }.is_some();
    let valid_msg = if valid { "" } else { " (invalid)" };
    let string = CString::new(format!("<{type_name}{valid_msg}>")).unwrap();

    unsafe { pl::Sfputs(string.as_ptr(), stream) };

    pl::TRUE as _
}

/// Tries to upgrade the raw [`Weak`] pointer to an [`Arc`] without changing the
/// weak count
///
/// SAFETY: The pointer must point to a valid [`Arc`] allocation as returned
/// from [`Weak::into_raw`]
unsafe fn upgrade_raw_weak<T>(ptr: *const T) -> Option<Arc<T>> {
    let weak = unsafe { Weak::from_raw(ptr) };
    let arc = weak.upgrade();
    // Don't drop the `Weak`, regardless of whether it still points to a valid
    // allocation or not to keep the `Weak` raw pointer valid and allow subsequent
    // checks
    std::mem::forget(weak);
    arc
}
