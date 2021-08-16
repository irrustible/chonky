use core::ptr::NonNull;
use heapless::Vec;

extern crate alloc;

mod link;
use link::*;

mod list;
pub use list::*;

mod pointer_chonks;
#[cfg(feature="recycling")]
mod local_pointer_chonks;

mod recycling;

/// Allocates a value with the global allocator according to the type's layout.
///
/// ## Safety
///
/// Basically everything about calling the global allocator as usual
/// applies, except it will allocate it with `T`s layout.
pub unsafe fn alloc<T>() -> *mut T {
    let layout = alloc::alloc::Layout::new::<T>();
    alloc::alloc::alloc(layout).cast()
}

/// ## Safety
///
/// Basically everything about calling the global allocator as usual
/// applies, except it will deallocate it with `T`s layout.
pub unsafe fn dealloc<T>(ptr: *mut T) {
    let layout = alloc::alloc::Layout::new::<T>();
    alloc::alloc::dealloc(ptr.cast(), layout);
}

#[repr(C)] // Force the layout to be consistent for any given H and
           // size of T so we can safely write PointerChonkList
pub struct Chonk<H, T, const N: usize> {
    pub header: H,
    pub data:   Vec<T, N>,
}

impl<H, T, const N: usize> Chonk<H, T, N> {
    #[inline(always)]
    pub fn push(&mut self, item: T) -> Result<(), T> { self.data.push(item) }
    #[inline(always)]
    pub fn len(&self) -> usize { self.data.len() }
    #[inline(always)]
    pub fn is_empty(&self) -> bool { self.data.is_empty() }
    #[inline(always)]
    pub fn is_full(&self) -> bool { self.data.is_full() }
    #[inline(always)]
    pub fn space(&self) -> usize { N - self.len() }
}

impl<H, T, const N: usize> From<H> for Chonk<H, T, N> {
    #[inline(always)]
    fn from(header: H) -> Self { Chonk { header, data: Vec::new() } }
}
