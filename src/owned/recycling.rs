use super::{*, local_pointer_chonks::*};
use std::thread::LocalKey;

pub struct RecyclingList<T, const N: usize> {
    key:  &'static LocalKey<LocalPointerChonks<N>>,
    list: List<*mut T, N>,
}

impl<T, const N: usize> RecyclingList<T, N> {
    #[inline(always)]
    pub fn len(&self) -> usize { self.list.len() }

    #[inline(always)]
    pub fn is_empty(&self) -> bool { self.list.is_empty() }

    #[inline(always)]
    pub fn is_full(&self) -> bool { self.list.is_full() }

    #[inline(always)]
    pub fn push(&mut self, item: Box<T>) -> Result<(), Box<T>> {
        let item = Box::leak(item);
        let key = self.key;
        unsafe {
            self.list.push_custom(item, || { LocalPointerChonks::pop(key) })
                .map_err(|x| Box::from_raw(x))
        }
    }

   #[inline(always)]
    pub unsafe fn push_custom<A>(&mut self, item: Box<T>, alloc: A) -> Result<(), Box<T>>
    where A: FnOnce() -> *mut ListChonk<* mut T, N> {
        let item = Box::leak(item);
        let key = self.key;
        self.list.push_custom(item, || {
            LocalPointerChonks::pop_custom(key, alloc)
        }).map_err(|x| Box::from_raw(x))
    }

    #[inline(always)]
    pub fn pop(&mut self) -> Box<T> {
        let key = self.key;
        unsafe {
            Box::from_raw(self.list.pop_custom(|i| {
                LocalPointerChonks::push(key, i)
            }).unwrap())
        }
    }

    #[inline(always)]
    pub unsafe fn pop_custom<F>(&mut self, free: F) -> Box<T>
    where F: FnOnce(*mut ListChonk<* mut T, N>) {
        let key = self.key;
        Box::from_raw(self.list.pop_custom(|i| {
            LocalPointerChonks::push_custom(key, i, free)
        }).unwrap())
    }
}
