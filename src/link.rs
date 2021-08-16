use core::mem::swap;
use core::ptr::NonNull;

pub struct Link<T>(pub(crate) Option<NonNull<T>>);

impl<T> Clone for Link<T> {
    #[inline(always)]
    fn clone(&self) -> Self { Link(self.0) }
}

impl<T> Copy for Link<T> {}

impl<T> From<Option<NonNull<T>>> for Link<T> {
    #[inline(always)]
    fn from(option: Option<NonNull<T>>) -> Self { Link(option) }
}

impl<T> From<Box<T>> for Link<T> {
    #[inline(always)]
    fn from(b: Box<T>) -> Self {
        let b = Box::leak(b) as *mut T;
        Link(Some(unsafe { NonNull::new_unchecked(b) }))
    }
}

impl<T> From<NonNull<T>> for Link<T> {
    #[inline(always)]
    fn from(ptr: NonNull<T>) -> Self { Link(Some(ptr)) }
}

impl<T> From<Link<T>> for Option<NonNull<T>> {
    #[inline(always)]
    fn from(link: Link<T>) -> Self { link.0 }
}

impl<T> Default for Link<T> {
    fn default() -> Self { Link(None) }
}

impl<T> Link<T> {
    #[inline(always)]
    pub fn take(&mut self) -> Link<T> { Link(self.0.take()) }
    #[inline(always)]
    pub fn replace(&mut self, mut other: Link<T>) -> Link<T> { self.swap(&mut other); other }
    #[inline(always)]
    pub fn swap(&mut self, other: &mut Link<T>) { swap(self,other) }
    #[inline(always)]
    pub fn as_ref(&self) -> Option<&T> { self.0.map(|x| unsafe { x.as_ptr().as_ref() }.unwrap()) }
    #[inline(always)]
    pub fn as_mut(&mut self) -> Option<&mut T> {
        self.0.as_mut().map(|x| unsafe { x.as_ptr().as_mut() }.unwrap())
    }
    #[inline(always)]
    pub fn or_else<L: Into<Link<T>>>(&mut self, f: impl FnOnce() -> L) {
        if self.0.is_none() {
            let mut val = f().into();
            self.swap(&mut val)
        }
    }
    #[inline(always)]
    pub unsafe fn boxed(self) -> Option<Box<T>> {
        self.0.map(|ptr| Box::from_raw(ptr.as_ptr()))
    }
}
