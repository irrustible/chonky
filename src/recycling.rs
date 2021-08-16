use crate::{*, local_pointer_chonks::*, pointer_chonks::*};

pub struct RecyclingList<T> {
    list: List<*mut T, CHONK_SIZE>,
}

impl<T> RecyclingList<T> {
    #[inline(always)]
    pub fn len(&self) -> usize { self.list.len() }

    #[inline(always)]
    pub fn push(&mut self, item: Box<T>) {
        let item = Box::leak(item);
        unsafe {
            self.list.push_custom(item, || LocalPointerChonks::pop())
        }
    }

    #[inline(always)]
    pub fn pop(&mut self) -> Box<T> {
        unsafe {
            Box::from_raw(self.list.pop_custom(|i| LocalPointerChonks::push(i)).unwrap())
        }
    }
}
