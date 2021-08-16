use crate::{*, pointer_chonks::*};

use core::cell::UnsafeCell;
#[derive(Default)]
pub struct LocalPointerChonks {
    chonks: UnsafeCell<PointerChonks>,
}

impl LocalPointerChonks {
    /// Pushes the provided chonk to the [`LocalPointerChonks`],
    /// falling back to calling [`alloc::alloc::dealloc`] when full.
    ///
    /// ## Note
    ///
    /// Any items in the chonk should already have been dropped.
    #[inline(always)]
    fn do_push<T, F: FnOnce(*mut ListChonk<* mut T, CHONK_SIZE>)>(
        &self,
        chonk_ptr: *mut ListChonk<* mut T, CHONK_SIZE>,
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
    ) -> *mut ListChonk<* mut T, CHONK_SIZE>
    where A: FnOnce() -> *mut ListChonk<* mut T, CHONK_SIZE> {
        unsafe { self.chonks.get().as_mut() }.unwrap().pop()
            .unwrap_or_else(|| alloc().cast())
    }

    #[inline(always)]
    fn do_len(&self) -> usize {
        unsafe { self.chonks.get().as_ref() }.unwrap().len()
    }
}

impl LocalPointerChonks {
    #[inline(always)]
    pub fn push<T>(chonk_ptr: *mut ListChonk<* mut T, CHONK_SIZE>) {
        LOCAL_POINTER_CHONKS.with(|lpc| {
            lpc.do_push(chonk_ptr, |ptr| unsafe { dealloc(ptr) })
        })
    }

    #[inline(always)]
    pub unsafe fn push_custom<T, F>(chonk_ptr: *mut ListChonk<* mut T, CHONK_SIZE>, free: F)
    where F: FnOnce(*mut ListChonk<* mut T, CHONK_SIZE>) {
        LOCAL_POINTER_CHONKS.with(|lpc| {
            lpc.do_push(chonk_ptr, free)
        })
    }

    #[inline(always)]
    pub fn pop<T>() -> *mut ListChonk<* mut T, CHONK_SIZE> {
        LOCAL_POINTER_CHONKS.with(|lpc| lpc.do_pop(|| unsafe { alloc() }))
    }

    #[inline(always)]
    pub unsafe fn pop_custom<T, A>(alloc: A) -> *mut ListChonk<* mut T, CHONK_SIZE>
    where A: FnOnce() -> *mut ListChonk<* mut T, CHONK_SIZE> {
        LOCAL_POINTER_CHONKS.with(|lpc| { lpc.do_pop(alloc) })
    }

    #[inline(always)]
    pub fn len() -> usize {
        LOCAL_POINTER_CHONKS.with(|lpc| lpc.do_len())
    }
}

std::thread_local! {
    static LOCAL_POINTER_CHONKS: LocalPointerChonks = LocalPointerChonks::default();
}
