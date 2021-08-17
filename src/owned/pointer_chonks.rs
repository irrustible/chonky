use super::*;
use core::ptr::NonNull;

pub type PointerChonk<const N: usize> = ListChonk<*mut u8, N>;

/// A list of chonks for `*mut T where T: Sized` (i.e. any non-fat pointer type).
///
/// Intended to be used as part of a thread-local freelist.
///
/// Has an additional optimisation over the regular list in that it
/// can use the chonks themselves to store more chonks, thus making it
/// totally allocation free so long as you keep providing chonks to
/// it. This also means you get an extra free entry per chonk in the
/// chonk itself.
///
/// ## Notes
///
/// * Panics if N is zero.
/// * The best values of `N` are powers of 2 to hasten maths.
/// * The best values of capacity are multiples of `N+1` to minimise memory waste.
pub struct PointerChonks<const N: usize> {
    /// Start pointer
    head: Link<PointerChonk<N>>,
    /// End pointer
    tail: Link<PointerChonk<N>>,
    /// All chunks, stored and internal, since they're interchangeable.
    length: usize,
    /// Maximum size we are allowed to grow to.
    capacity: usize,
}

impl<const N: usize> PointerChonks<N> {

    /// Creates a [`PointerChonks`] that will not store more than
    /// `capacity` chonks. This does not change how allocation happens
    /// at all, it merely imposes a limit on maximum length.
    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> Self {
        assert!(N > 0, "You may not create a zero-sized PointerChonks");
        PointerChonks {
            head: Link::default(),
            tail: Link::default(),
            length: 0,
            capacity,
        }
    }

    #[inline(always)]
    pub fn len(&self) -> usize { self.length }

    #[inline(always)]
    pub fn capacity(&self) -> usize { self.capacity }

    #[inline(always)]
    pub fn is_empty(&self) -> bool { self.length == 0}

    #[inline(always)]
    pub fn is_full(&self) -> bool { self.length == self.capacity }

    pub fn pop<T>(&mut self) -> Option<*mut ListChonk<* mut T, N>> {
        if let Some(tail) = self.tail.as_mut() {
            // We are guaranteed to shrink because worst case, we can
            // give them our empty chonk.
            self.length -= 1;
            // Try and pop an item off the tail
            if let Some(item) = tail.0.data.pop() {
                // Success. 
                return Some(Self::init(item.cast()))
            }
            // No? Give them the block itself.
            // The new tail will be the old tail's prev pointer.
            let mut new_tail = tail.0.header.prev.take();
            // and it will not point to another block.
            new_tail.as_mut().unwrap().0.header.next.take();
            let old_tail = {
                self.tail.swap(&mut new_tail);
                new_tail
            };
            // If that was the last block, we have to fix up the head pointer.
            if self.length == 0 { self.head.take(); }
            return Some(Self::init(old_tail.0.unwrap().as_ptr().cast()));
        }
        None
    }

    /// Push a dropped (but not freed) [`PointerChonk`] onto the [`PointerChonks`]
    pub fn push<T>(
        &mut self,
        chonk_ptr: *mut ListChonk<* mut T, N>
    ) -> Result<(), *mut ListChonk<* mut T, N>> {
        // Check we wouldn't go over our capacity.
        if self.length == self.capacity { return Err(chonk_ptr); }
        // No? Then we're guaranteed to grow because worst case we can
        // reuse the chonk ourselves.
        self.length += 1;
        // Cast to the internal pointer type. This is safe because:
        //
        //  * `ListChonk` is `#[repr(transparent)]` and contains a `Chonk`
        //  * `Chonk` is #[repr(c)]`
        //  * All non-fat pointers are the same size.
        // This is safe because it ultimately boils down to a repr(C)
        // data structure and all (non-fat) pointers are the same size.
        let chonk_ptr: *mut PointerChonk<N> = chonk_ptr.cast();
        if let Some(tail) = self.tail.as_mut() {
            // Try and push an item onto the tail
            if tail.0.data.push(chonk_ptr.cast()).is_err() {
                // No? it can be the next block. Doesn't make us longer though.
                let mut chonk = Link(Some(unsafe { NonNull::new_unchecked(Self::init(chonk_ptr)) }));
                tail.0.header.next.replace(chonk);                        // The existing tail should point to us.
                chonk.as_mut().unwrap().0.header.prev.replace(self.tail); // And we should point to the existing tail
                self.tail.replace(chonk);                                 // We are the new tail.
            } 
        } else {
            // It can be the first block.
            let chonk = Link(Some(unsafe { NonNull::new_unchecked(Self::init(chonk_ptr)) }));
            // That is to say we are head andn  tail.
            self.tail.replace(chonk);
            self.head.replace(chonk);
        }
        Ok(())
    }

    #[inline(always)]
    /// Casts an uninit internal chunk to an external chunk and
    /// initialises it appropriately for use.
    fn init<T>(ptr: *mut PointerChonk<N>) -> *mut ListChonk<*mut T, N> {
        // Cast to the generic pointer type. This is safe because:
        //
        // * `ListChonk` is `#[repr(transparent)]` and contains a `Chonk`
        // * `Chonk` is #[repr(c)]`
        // * All non-fat pointers are the same size.
        let ptr = ptr.cast::<ListChonk<*mut T, N>>();
        // Write the default value
        unsafe { ptr.write(ListChonk::default()); }
        ptr
    }
}

impl<const N: usize> Default for PointerChonks<N> {
    /// The default size is 8 * (N+1).
    /// For N=32: 528, enough chonks to store 8448 items.
    /// For N=16, 272, enough chonks to store 4352 items.
    #[inline(always)]
    fn default() -> Self { Self::with_capacity(8 * (N + 1)) }
}
