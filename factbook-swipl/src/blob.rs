use crate::term::{Term, ToTerm};
pub use factbook_swipl_macros::{BlobData, CopyBlobData};
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use swipl_fli as pl;

#[repr(transparent)]
pub struct BlobSpec<T> {
    _marker: PhantomData<T>,
    spec: pl::PL_blob_t,
}

impl<T> BlobSpec<T> {
    pub const fn new(name: &'static CStr) -> Self
    where
        T: BlobData,
    {
        let mut spec = unsafe { MaybeUninit::<pl::PL_blob_t>::zeroed().assume_init() };

        spec.magic = pl::PL_BLOB_MAGIC as _;
        spec.flags = (pl::PL_BLOB_UNIQUE | pl::PL_BLOB_NOCOPY) as _;
        spec.name = name.as_ptr();
        spec.release = Some(blob_release::<T>);
        spec.write = Some(blob_write::<T>);

        Self {
            _marker: PhantomData,
            spec,
        }
    }

    pub const fn copy(name: &'static CStr) -> Self
    where
        T: CopyBlobData,
    {
        let mut spec = unsafe { MaybeUninit::<pl::PL_blob_t>::zeroed().assume_init() };

        spec.magic = pl::PL_BLOB_MAGIC as _;
        spec.name = name.as_ptr();
        spec.write = Some(copy_blob_write::<T>);

        Self {
            _marker: PhantomData,
            spec,
        }
    }
}

/// A non-copyable blob which owns `T`
pub struct Blob<T: BlobData>(Box<T>);

/// A copyable blob which owns `T`
#[derive(Clone, Copy)]
pub struct CopyBlob<T: CopyBlobData>(pub T);

impl<T: BlobData> Blob<T> {
    pub fn new(value: T) -> Self {
        Self(Box::new(value))
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
    const SPEC: *mut BlobSpec<Self>;
}

/// A type which can be stored in a [`CopyBlob`]. Its destructor is not called
/// when the blob is freed. The type must be `'static` because its memory is
/// managed by Prolog.
///
/// # Safety
/// `SPEC` must be a valid pointer to a static `BlobSpec` instance for that
/// type.
pub unsafe trait CopyBlobData: Copy + std::fmt::Debug + 'static {
    const SPEC: *mut BlobSpec<Self>;
}

impl<T: BlobData> ToTerm for Blob<T> {
    fn put_in(self, term: Term) {
        if unsafe {
            pl::PL_put_blob(
                term.ptr,
                Box::leak(self.0) as *mut _ as _,
                std::mem::size_of::<T>(),
                T::SPEC as _,
            )
        } == 0
        {
            panic!("PL_put_blob failed");
        }
    }
}

impl<T: CopyBlobData> ToTerm for CopyBlob<T> {
    fn put_in(mut self, term: Term) {
        if unsafe {
            pl::PL_put_blob(
                term.ptr,
                &raw mut self.0 as _,
                std::mem::size_of::<T>(),
                T::SPEC as _,
            )
        } == 0
        {
            panic!("PL_put_blob failed");
        }
    }
}

extern "C" fn blob_write<T: BlobData>(
    stream: *mut pl::IOSTREAM,
    atom: pl::atom_t,
    _flags: std::ffi::c_int,
) -> std::ffi::c_int {
    let blob_ptr = unsafe { pl::PL_blob_data(atom, std::ptr::null_mut(), std::ptr::null_mut()) };
    let blob = unsafe { (blob_ptr as *mut T).as_ref().unwrap() };

    let string = CString::new(format!("{blob:?}")).unwrap();
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

    let string = CString::new(format!("{blob:?}")).unwrap();
    unsafe { pl::Sfputs(string.as_ptr(), stream) };

    pl::TRUE as _
}
