use heapless::Vec;

mod link;
pub(crate) use link::*;

mod list;
pub use list::*;

pub mod pointer_chonks;

#[cfg(feature="std")]
pub mod local_pointer_chonks;

#[cfg(feature="std")]
pub mod recycling;

/// A fixed-size vector of values appended to a header.
///
/// ## Note
///
/// The best values of `N` are powers of 2 to hasten the maths
// This repr forces the layout to be consistent for any given H when T
// is any non-fat pointer, so we can safely write PointerChonkList. I
// don't think the compiler toolchain (currently) (ab)uses this
// possibility, but let's wear a tinfoil hat and be sure.
#[repr(C)]
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
    fn from(header: H) -> Self {
        assert!(N > 0, "You may not create a zero-sized chonk");
        Chonk { header, data: Vec::new() }
    }
}

/// Allocates a value with the global allocator according to the type's layout.
///
/// ## Safety
///
/// Basically everything about calling the global allocator as usual
/// applies, except it will allocate it with `T`s layout.
unsafe fn alloc<T>() -> *mut T {
    let layout = alloc::alloc::Layout::new::<T>();
    alloc::alloc::alloc(layout).cast()
}

/// ## Safety
///
/// Basically everything about calling the global allocator as usual
/// applies, except it will deallocate it with `T`s layout.
unsafe fn dealloc<T>(ptr: *mut T) {
    let layout = alloc::alloc::Layout::new::<T>();
    alloc::alloc::dealloc(ptr.cast(), layout);
}
