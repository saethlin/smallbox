use std::ops;
use std::mem;
use std::ptr;
use std::ptr::Unique;
use std::marker;
use std::fmt;
use std::hash;
use std::hash::Hash;
use std::cmp::Ordering;
use super::space::U4;

/// On-stack allocation for dynamically-sized type.
///
/// # Examples
///
/// ```
/// use smallbox::StackBox;
///
/// let val: StackBox<PartialEq<usize>> = StackBox::new(5usize).unwrap();
///
/// assert!(*val == 5)
/// ```
pub struct StackBox<T: ?Sized, Space = U4> {
    ptr: Unique<T>,
    space: Space,
}

impl<T: ?Sized, Space> StackBox<T, Space> {
    /// Try to alloc on stack, and return Err<T>
    /// when val is too large (about 4 words)
    ///
    /// # Examples
    ///
    /// ```
    /// use std::any::Any;
    /// use smallbox::StackBox;
    ///
    /// assert!(StackBox::<Any>::new(5usize).is_ok());
    /// assert!(StackBox::<Any>::new([5usize; 8]).is_err());
    /// ```
    pub fn new<U>(val: U) -> Result<StackBox<T, Space>, U>
        where U: marker::Unsize<T>
    {
        if mem::size_of::<U>() > mem::size_of::<Space>() {
            Err(val)
        } else {
            unsafe { Ok(Self::box_up(val)) }
        }
    }

    pub fn resize<ToSpace>(self) -> Result<StackBox<T, ToSpace>, Self> {
        if mem::size_of::<Space>() > mem::size_of::<ToSpace>() {
            Err(self)
        } else {
            unsafe {
                let ptr = self.ptr;
                let mut space = mem::uninitialized::<ToSpace>();
                ptr::copy_nonoverlapping(&self.space, &mut space as *mut _ as *mut Space, 1);
                mem::forget(self);
                Ok(StackBox { ptr, space })
            }
        }
    }

    unsafe fn box_up<U>(mut val: U) -> StackBox<T, Space>
        where U: marker::Unsize<T>
    {
        let ptr: Unique<T> = Unique::new(&mut val);

        let mut space = mem::uninitialized::<Space>();
        ptr::copy_nonoverlapping(&val, &mut space as *mut _ as *mut U, 1);
        mem::forget(val);

        StackBox { ptr, space }
    }

    unsafe fn as_ptr(&self) -> *const T {
        let mut ptr: *const T = self.ptr.as_ptr();
        *(&mut ptr as *mut _ as *mut usize) = &self.space as *const _ as usize;
        ptr
    }
}

impl<T: ?Sized, Space> ops::Deref for StackBox<T, Space> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.as_ptr() }
    }
}

impl<T: ?Sized, Space> ops::DerefMut for StackBox<T, Space> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *(self.as_ptr() as *mut T) }
    }
}

impl<T: ?Sized, Space> ops::Drop for StackBox<T, Space> {
    fn drop(&mut self) {
        unsafe { ptr::drop_in_place(&mut **self) }
    }
}

impl<T: fmt::Display + ?Sized, Space> fmt::Display for StackBox<T, Space> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: fmt::Debug + ?Sized, Space> fmt::Debug for StackBox<T, Space> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: ?Sized, Space> fmt::Pointer for StackBox<T, Space> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // It's not possible to extract the inner Uniq directly from the Box,
        // instead we cast it to a *const which aliases the Unique
        let ptr: *const T = &**self;
        fmt::Pointer::fmt(&ptr, f)
    }
}

impl<T: ?Sized + PartialEq, Space> PartialEq for StackBox<T, Space> {
    #[inline]
    fn eq(&self, other: &StackBox<T, Space>) -> bool {
        PartialEq::eq(&**self, &**other)
    }
    #[inline]
    fn ne(&self, other: &StackBox<T, Space>) -> bool {
        PartialEq::ne(&**self, &**other)
    }
}

impl<T: ?Sized + PartialOrd, Space> PartialOrd for StackBox<T, Space> {
    #[inline]
    fn partial_cmp(&self, other: &StackBox<T, Space>) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
    #[inline]
    fn lt(&self, other: &StackBox<T, Space>) -> bool {
        PartialOrd::lt(&**self, &**other)
    }
    #[inline]
    fn le(&self, other: &StackBox<T, Space>) -> bool {
        PartialOrd::le(&**self, &**other)
    }
    #[inline]
    fn ge(&self, other: &StackBox<T, Space>) -> bool {
        PartialOrd::ge(&**self, &**other)
    }
    #[inline]
    fn gt(&self, other: &StackBox<T, Space>) -> bool {
        PartialOrd::gt(&**self, &**other)
    }
}

impl<T: ?Sized + Ord, Space> Ord for StackBox<T, Space> {
    #[inline]
    fn cmp(&self, other: &StackBox<T, Space>) -> Ordering {
        Ord::cmp(&**self, &**other)
    }
}

impl<T: ?Sized + Eq, Space> Eq for StackBox<T, Space> {}

impl<T: ?Sized + Hash, Space> Hash for StackBox<T, Space> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}