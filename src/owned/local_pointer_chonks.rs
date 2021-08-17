use super::{*, pointer_chonks::*};
use std::thread::LocalKey;
use core::cell::UnsafeCell;

pub struct LocalPointerChonks<const N: usize> {
    chonks: UnsafeCell<PointerChonks<N>>,
}

impl<const N: usize> LocalPointerChonks<N> {
    /// Pushes the provided chonk to the [`LocalPointerChonks`],
    /// falling back to calling [`alloc::alloc::dealloc`] when full.
    ///
    /// ## Note
    ///
    /// Any items in the chonk should already have been dropped.
    #[inline(always)]
    fn do_push<T, F: FnOnce(*mut ListChonk<* mut T, N>)>(
        &self,
        chonk_ptr: *mut ListChonk<* mut T, N>,
        free: F
    ) {
        unsafe { self.chonks.get().as_mut() }.unwrap().push(chonk_ptr)
            .unwrap_or_else(|_| free(chonk_ptr))
    }

    /// Attempts to grab an empty chonk from the list, falling back to
    /// allocating a new one.
    #[inline(always)]
    fn do_pop<T, A>(
        &self,
        alloc: A
    ) -> *mut ListChonk<* mut T, N>
    where A: FnOnce() -> *mut ListChonk<* mut T, N> {
        unsafe { self.chonks.get().as_mut() }.unwrap().pop()
            .unwrap_or_else(|| alloc().cast())
    }

    #[inline(always)]
    fn do_len(&self) -> usize {
        unsafe { self.chonks.get().as_ref() }.unwrap().len()
    }
}

impl<const N: usize> LocalPointerChonks<N> {
    pub fn with_capacity(cap: usize) -> Self {
        assert!(N > 0, "You may not create a LocalPointerChonks with zero-sized chonks");
        assert!(cap > 0, "You may not create a zero-sized LocalPointerChonks");
        LocalPointerChonks { chonks: UnsafeCell::new(PointerChonks::with_capacity(cap)) }
    }

    #[inline(always)]
    pub fn push<T>(
        key: &'static LocalKey<LocalPointerChonks<N>>,
        chonk_ptr: *mut ListChonk<* mut T, N>
    ) {
        key.with(|lpc| {
            lpc.do_push(chonk_ptr, |ptr| unsafe { dealloc(ptr) })
        })
    }

    #[inline(always)]
    pub unsafe fn push_custom<T, F>(
        key: &'static LocalKey<LocalPointerChonks<N>>,
        chonk_ptr: *mut ListChonk<* mut T, N>,
        free: F
    )
    where F: FnOnce(*mut ListChonk<* mut T, N>) {
        key.with(|lpc| { lpc.do_push(chonk_ptr, free) })
    }

    #[inline(always)]
    pub fn pop<T>(key: &'static LocalKey<LocalPointerChonks<N>>) -> *mut ListChonk<* mut T, N> {
        key.with(|lpc| lpc.do_pop(|| unsafe { alloc() }))
    }

    #[inline(always)]
    pub unsafe fn pop_custom<T, A>(
        key: &'static LocalKey<LocalPointerChonks<N>>,
        alloc: A
    ) -> *mut ListChonk<* mut T, N>
    where A: FnOnce() -> *mut ListChonk<* mut T, N> {
        key.with(|lpc| { lpc.do_pop(alloc) })
    }

    #[inline(always)]
    pub fn len(key: &'static LocalKey<LocalPointerChonks<N>>) -> usize {
        key.with(|lpc| lpc.do_len())
    }
}

impl<const N: usize> Default for LocalPointerChonks<N> {
    #[inline(always)]
    fn default() -> Self { Self::with_capacity(8 * (N + 1)) }
}

