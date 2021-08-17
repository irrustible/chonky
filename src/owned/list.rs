use super::*;
use core::ptr::{NonNull, drop_in_place};

/// A chunked doubly-linked list. Efficient for the following operations:
///
/// * In-order chunked traversal and consumption
/// * Append (at tail)
/// * Pop    (at tail)
///
/// Allows you to plug in your own allocator via closures so you can
/// use a custom allocator on a stable rust.
///
/// ## Note
///
/// The best values of `N` will be powers of 2 as it makes the maths
/// quicker and probably is kinder to the allocator.
pub struct List<T, const N: usize> {
    head: Link<ListChonk<T, N>>,
    tail: Link<ListChonk<T, N>>,
    len:  usize,
    cap:  usize,
}

impl<T, const N: usize> Default for List<T, N> {
    #[inline(always)]
    fn default() -> Self { Self::with_capacity(usize::MAX) }
}
impl<T, const N: usize> List<T, N> {

    #[inline(always)]
    pub fn with_capacity(cap: usize) -> Self {
        List { head: Link::default(), tail: Link::default(), len: 0, cap }
    }

    #[inline(always)]
    pub fn len(&self) -> usize { self.len }

    #[inline(always)]
    pub fn capacity(&self) -> usize { self.cap }

    #[inline(always)]
    pub fn is_empty(&self) -> bool { self.len == 0 }

    #[inline(always)]
    pub fn is_full(&self) -> bool { self.len == self.cap }

    #[inline(always)]
    pub fn pop(&mut self) -> Option<T> {
        unsafe { self.pop_custom(|x| dealloc(x)) }
    }

    #[inline(always)]
    pub fn push(&mut self, item: T) -> Result<(), T> {
        unsafe { self.push_custom(item, || alloc::<ListChonk<T, N>>()) }
    }

    pub unsafe fn pop_custom<F>(&mut self, free: F) -> Option<T>
    where F: FnOnce(*mut ListChonk<T, N>) {
        if let Some(tail) = self.tail.as_mut() {
            if let Some(item) = tail.0.data.pop() { return Some(item); }
            let mut tail = self.tail;
            let mut prev = tail.as_mut().unwrap().0.header.prev.take();
            self.tail.swap(&mut prev);
            ListChonk::drop_in(tail.0.unwrap().as_ptr().cast(), free);
            return self.tail.as_mut()?.0.data.pop()
                .map(|r| { self.len -= 1; r })
        }
        None
    }

    pub unsafe fn push_custom<A>(&mut self, item: T, alloc: A) -> Result<(), T>
    where A: FnOnce() -> *mut ListChonk<T, N> {
        // First check we have capacity
        if self.len == self.cap { return Err(item); }
        if let Some(tail) = self.tail.as_mut() {
            // There's a block! Try push,fall back to fetching a new block.
            tail.0.data.push(item)
                .unwrap_or_else(|item| self.add_block(item, alloc))
        } else {
            // We will need a block.
            self.add_first_block(item, alloc)            
        }
        self.len += 1;
        Ok(())
    }

    unsafe fn add_first_block<A>(&mut self, item: T, alloc: A)
    where A: FnOnce() -> *mut ListChonk<T, N> {
        let mut chonk = ListChonk::new_in(alloc);
        // The chonk is promised to be empty. This mess is to avoid T: Debug.
        chonk.as_mut().unwrap().0.data.push(item).map_err(|_| ()).unwrap();
        // First chonk. Both head and tail should point to it.
        self.tail.replace(chonk);
        self.head.replace(chonk);
    }

    unsafe fn add_block<A>(&mut self, item: T, alloc: A)
    where A: FnOnce() -> *mut ListChonk<T, N> {
        let mut chonk = ListChonk::new_in(alloc);
        // Start out by copying the tail because we need it at the end.
        let mut old =  self.tail;
        // Our new tail is the tail and the old tail points to the new tail.
        self.tail.replace(chonk);
        old.as_mut().unwrap().0.header.next.replace(chonk);
        // Now we have to prepare the chonk.
        let ch = chonk.as_mut().unwrap();
        // The chonk is promised to be empty. This mess is to avoid T: Debug.
        ch.0.data.push(item).map_err(|_| ()).unwrap();
        // The new tail should point to the old tail
        ch.0.header.prev.replace(old);
    }

}

/// An opaque fixed-size chunk used by the [`List`] to store
/// entries. Only exposed so you can allocate/deallocate them
/// yourself.
#[repr(transparent)] // Force chonk's layout guarantees.
pub struct ListChonk<T, const N: usize>(pub(crate) Chonk<Links<Self>, T, N>);

impl<T, const N: usize> Default for ListChonk<T, N> {
    #[inline(always)]
    fn default() -> Self {
        assert!(N > 0, "You may not create a zero-sized chonk");
        ListChonk(Chonk::from(Links::default()))
    }
}

impl<T, const N: usize> ListChonk<T, N> {
    /// ## Safety
    ///
    /// The provided allocator function must return a valid and
    /// properly aligned pointer for the type `T`.
    #[inline(always)]
    unsafe fn new_in<A>(alloc: A) -> Link<ListChonk<T, N>>
    where A: FnOnce() -> *mut ListChonk<T, N> {
        assert!(N > 0, "You may not create a zero-sized chonk");
        let ptr = alloc();
        ptr.write(Self::default());
        Link(Some(NonNull::new_unchecked(ptr)))
    }

    /// ## Safety
    ///
    /// The provided pointer must be valid, properly aligned and
    /// have been allocated with the corresponding allocator.
    #[inline(always)]
    unsafe fn drop_in<F>(chonk: *mut Self, free: F)
    where F: FnOnce(*mut ListChonk<T, N>) {
        assert!(N > 0, "You may not drop a zero-sized chonk");
        drop_in_place(chonk);
        free(chonk.cast());
    }
}

pub(crate) struct Links<T> {
    pub(crate) prev: Link<T>,
    pub(crate) next: Link<T>,
}

impl<T> Default for Links<T> {
    #[inline(always)]
    fn default() -> Self { Links { prev: Link::default(), next: Link::default() } }
}
