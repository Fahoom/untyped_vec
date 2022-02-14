mod utils;

use std::{
    alloc::{handle_alloc_error, Layout},
    ops::Add,
    ptr::NonNull,
};

use crate::utils::array_layout;

/// A type-erased version of the standard [`Vec`]
pub struct UntypedVec {
    ptr: NonNull<u8>,
    capacity: usize,
    len: usize,
    layout: Layout,
    drop: unsafe fn(*mut u8)
}

impl UntypedVec {
    pub fn new<T>() -> Self {
        // We can  hold a usize::MAX amount of zero sized types
        let layout = Layout::new::<T>();
        let capacity = if layout.size() == 0 { usize::MAX } else { 0 };

        Self {
            ptr: NonNull::dangling(),
            capacity,
            len: 0,
            layout,
            drop: utils::drop_ptr::<T>
        }
    }

    pub fn with_capacity<T>(capacity: usize) -> Self {
        let mut vec = UntypedVec::new::<T>();
        vec.reserve_exact(capacity);
        vec
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn reserve_exact(&mut self, amount: usize) {
        let avail = self.capacity() - self.len();
        if avail < amount {
            self.grow(amount)
        }
    }

    pub fn swap_remove<T>(&mut self, index: usize) -> T {
        assert!(index < self.len());
        assert_eq!(Layout::new::<T>(), self.layout);

        unsafe {
            let value = std::ptr::read(self.ptr_to(index).cast::<T>());
            std::ptr::copy(
                self.ptr_to(self.len() - 1),
                self.ptr_to(index),
                self.layout.size(),
            );
            self.len -= 1;
            value
        }
    }

    pub fn push<T>(&mut self, elem: T) {
        assert_eq!(Layout::new::<T>(), self.layout);

        let ptr = utils::to_const_ptr(&elem);
        unsafe { self.push_ptr(ptr) };
    }
    pub fn pop<T>(&mut self) -> Option<T> {
        assert_eq!(Layout::new::<T>(), self.layout);
        if self.len() == 0 {
            None
        } else {
            self.len -= 1;
            unsafe {
                Some(std::ptr::read(
                    self.ptr().as_ptr().cast::<T>().add(self.len()),
                ))
            }
        }
    }

    pub fn get<T>(&self, index: usize) -> &T {
        assert_eq!(Layout::new::<T>(), self.layout);
        assert!(index < self.len());

        unsafe { &*self.ptr_to(index).cast::<T>() }
    }
    pub fn get_mut<T>(&mut self, index: usize) -> &mut T {
        assert_eq!(Layout::new::<T>(), self.layout);
        assert!(index < self.len());

        unsafe { &mut *self.ptr_to(index).cast::<T>() }
    }

    pub fn clear(&mut self) {
        let len = self.len;
        self.len = 0;
        for i in 0..len {
            unsafe {
                let ptr = self.ptr_to(i);
                (self.drop)(ptr);
            }
        }
    }

    fn stores_zst(&self) -> bool {
        self.layout.size() == 0
    }
    fn ptr(&self) -> NonNull<u8> {
        self.ptr
    }

    /// # Safety
    /// Returned pointer may not always contain valid data, and even if it does, it may change after reallocation.
    /// Index should be less than capacity.
    unsafe fn ptr_to(&self, index: usize) -> *mut u8 {
        debug_assert!(index < self.capacity());
        self.ptr().as_ptr().add(index * self.layout.size())
    }

    fn grow(&mut self, amount: usize) {
        // grow() should never be reached if storing a ZST. If it is, the len has managed to exceed usize::MAX
        assert!(!self.stores_zst(), "Exceeded capacity");

        let new_capacity = self.capacity() + amount;
        let new_layout = utils::array_layout(&self.layout, new_capacity)
            .expect("Failed to create valid array layout");

        unsafe {
            let new_ptr = {
                if self.capacity == 0 {
                    std::alloc::alloc(new_layout)
                } else {
                    let old_layout = array_layout(&self.layout, self.capacity())
                        .expect("Failed to create valid array layout");
                    std::alloc::realloc(self.ptr().as_ptr(), old_layout, new_layout.size())
                }
            };

            self.ptr = NonNull::new(new_ptr).unwrap_or_else(|| handle_alloc_error(new_layout));
        }

        self.capacity = new_capacity;
    }

    /// # Safety
    /// src should be a valid pointer for a read of `self.layout.size()`
    unsafe fn push_ptr(&mut self, src: *const u8) {
        self.reserve_exact(1);
        // SAFETY: Safe as we have reserved the next blob of memory
        let ptr = self.ptr_to(self.len());
        std::ptr::copy_nonoverlapping(src, ptr, self.layout.size());
        self.len += 1;
    }
}

mod tests {
    use crate::UntypedVec;

    #[derive(Debug, PartialEq)]
    struct Foo {
        i: usize,
    }
    #[derive(Debug, PartialEq)]
    struct Bar {
        i: u32,
    }
    #[derive(Debug, PartialEq)]
    struct ZST;

    #[test]
    fn push_elements() {
        let mut vec = UntypedVec::new::<Foo>();
        for i in 0..100 {
            vec.push(Foo { i })
        }

        for i in 0..100 {
            assert_eq!(vec.get::<Foo>(i), &Foo { i })
        }

        for i in 0..100 {
            assert_eq!(vec.pop::<Foo>().unwrap(), Foo { i: 99 - i })
        }
    }

    #[test]
    fn push_zsts() {
        let mut vec = UntypedVec::new::<ZST>();
        for i in 0..100 {
            vec.push(ZST)
        }

        for i in 0..100 {
            assert_eq!(vec.get::<ZST>(i), &ZST)
        }

        for i in 0..100 {
            assert_eq!(vec.pop::<ZST>().unwrap(), ZST)
        }
    }

    #[test]
    fn swap_remove() {
        let mut vec = UntypedVec::new::<Foo>();
        for i in 0..100 {
            vec.push(Foo { i })
        }

        for i in 0..100 {
            assert_eq!(vec.get::<Foo>(i), &Foo { i })
        }
        
        assert_eq!(vec.swap_remove::<Foo>(0), Foo {i: 0});
        assert_eq!(vec.get::<Foo>(0), &Foo {i: 99});
    }

}
