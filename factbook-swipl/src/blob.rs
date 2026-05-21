use crate::Atom;
use crate::term::{FromTerm, Term, ToTerm};
pub use factbook_swipl_macros::{BlobData, CopyBlobData, ScopedBlobData};
use std::ffi::{CStr, CString};
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
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
/// lifetime. It may be put in a term by reference. While it's held, shared
/// references to the data may obtained via [`Atom::scoped_blob`] and mutable
/// references via [`Atom::scoped_blob_mut`]. Borrow rules are dynamically
/// checked using an [`RwLock`].
///
/// It's not safe to share the terms holding references to scoped blobs between
/// threads. Dropping the original scoped blob while borrowed in another thread
/// will panic.
///
/// Unlike a [`Blob`], the inner type is dropped when this handle is dropped,
/// not when the blob is released by Prolog.
pub struct ScopedBlob<'a, T: ScopedBlobData> {
    // We store the raw heap allocation, because the handle needs to be held and potentially give
    // out more than one reference to it. We cannot own a `Box` since we can't leak it multiple
    // times.
    data: *const ScopedBlobAlloc<'a, T>,
}

// SAFETY: `RwLock` is `Send + Sync`, we're just using a raw pointer instead of
// a reference
unsafe impl<T: ScopedBlobData + Sync> Send for ScopedBlob<'_, T> {}
unsafe impl<T: ScopedBlobData + Sync> Sync for ScopedBlob<'_, T> {}

type ScopedBlobAlloc<'a, T> = RwLock<Option<&'a mut T>>;

/// A reference to a value stored in a [`ScopedBlob`]. When held, the blob data
/// is guaranteed to be alive.
pub struct ScopedBlobRef<'a, T: ScopedBlobData> {
    // The `Option` is ensured to be `Some` before construction
    // TODO: Would be nice to use https://doc.rust-lang.org/std/sync/struct.MappedRwLockReadGuard.html
    guard: RwLockReadGuard<'a, Option<&'a mut T>>,
}

/// Same as [`ScopedBlobRef`], but mutable
pub struct ScopedBlobMut<'a, T: ScopedBlobData> {
    // The `Option` is ensured to be `Some` before construction
    // TODO: Would be nice to use https://doc.rust-lang.org/std/sync/struct.MappedRwLockWriteGuard.html
    guard: RwLockWriteGuard<'a, Option<&'a mut T>>,
}

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

impl<'a, T: ScopedBlobData> ScopedBlob<'a, T> {
    pub fn new(value: &'a mut T) -> Self {
        Self {
            data: Box::leak(Box::new(RwLock::new(Some(value)))),
        }
    }
}

impl<T: ScopedBlobData> Deref for ScopedBlobRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.guard.as_ref().unwrap()
    }
}

impl<T: ScopedBlobData> Deref for ScopedBlobMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.guard.as_ref().unwrap()
    }
}

impl<T: ScopedBlobData> DerefMut for ScopedBlobMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.as_mut().unwrap()
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
        if !unsafe {
            pl::PL_put_blob(
                term.ptr.get(),
                Box::leak(self.0) as *mut _ as _,
                std::mem::size_of::<T>(),
                T::SPEC as _,
            )
        } {
            panic!("PL_put_blob failed");
        }
    }
}

impl<T: CopyBlobData> ToTerm for CopyBlob<T> {
    fn put_in(mut self, term: Term) {
        if !unsafe {
            pl::PL_put_blob(
                term.ptr.get(),
                &raw mut self.0 as _,
                std::mem::size_of::<T>(),
                T::SPEC as _,
            )
        } {
            panic!("PL_put_blob failed");
        }
    }
}

impl<T: ScopedBlobData> ToTerm for &ScopedBlob<'_, T> {
    fn put_in(self, term: Term) {
        if !unsafe {
            pl::PL_put_blob(
                term.ptr.get(),
                self.data as _,
                std::mem::size_of::<ScopedBlobAlloc<T>>(),
                T::SPEC as _,
            )
        } {
            panic!("PL_put_blob failed");
        }
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

    pub fn scoped_blob<'a, T: ScopedBlobData>(&'a self) -> Option<ScopedBlobRef<'a, T>> {
        let mut spec: *mut pl::PL_blob_t = std::ptr::null_mut();
        let blob_ptr = unsafe { pl::PL_blob_data(self.ptr, std::ptr::null_mut(), &raw mut spec) };

        if std::ptr::eq(T::SPEC, spec as _) {
            let lock = unsafe { &*(blob_ptr as *const ScopedBlobAlloc<T>) };
            let guard = lock
                .try_read()
                .expect("ScopedBlob borrowed from another thread");

            guard.as_ref()?; // ensure the reference is still there

            Some(ScopedBlobRef { guard })
        } else {
            None
        }
    }

    pub fn scoped_blob_mut<'a, T: ScopedBlobData>(&'a self) -> Option<ScopedBlobMut<'a, T>> {
        let mut spec: *mut pl::PL_blob_t = std::ptr::null_mut();
        let blob_ptr = unsafe { pl::PL_blob_data(self.ptr, std::ptr::null_mut(), &raw mut spec) };

        if std::ptr::eq(T::SPEC, spec as _) {
            let lock = unsafe { &*(blob_ptr as *const ScopedBlobAlloc<T>) };
            let guard = lock
                .try_write()
                .expect("ScopedBlob borrowed from another thread");

            guard.as_ref()?; // ensure the reference is still there

            Some(ScopedBlobMut { guard })
        } else {
            None
        }
    }
}

impl<T: ScopedBlobData> Drop for ScopedBlob<'_, T> {
    fn drop(&mut self) {
        unsafe { &*self.data }
            .try_write()
            .expect("ScopedBlob borrowed from another thread")
            .take();
    }
}

extern "C" fn blob_write<T: BlobData>(
    stream: *mut pl::IOSTREAM,
    atom: pl::atom_t,
    _flags: std::ffi::c_int,
) -> std::ffi::c_int {
    let blob_ptr = unsafe { pl::PL_blob_data(atom, std::ptr::null_mut(), std::ptr::null_mut()) };
    let blob = unsafe { &*(blob_ptr as *mut T) };

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
    let blob = unsafe { *(blob_ptr as *mut T) };

    let string = CString::new(format!("<{blob:?}>")).unwrap();
    unsafe { pl::Sfputs(string.as_ptr(), stream) };

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

    let blob_lock = unsafe { &*(blob_ptr as *mut ScopedBlobAlloc<T>) };

    let valid = match blob_lock.try_read() {
        Ok(data) => data.is_some(),
        Err(_) => true, // if it's borrowed then it must be valid
    };

    let valid_msg = if valid { "" } else { " (invalid)" };
    let string = CString::new(format!("<{type_name}{valid_msg}>")).unwrap();

    unsafe { pl::Sfputs(string.as_ptr(), stream) };

    pl::TRUE as _
}
