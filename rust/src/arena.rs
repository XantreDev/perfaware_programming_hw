use std::{cell::Cell, mem::MaybeUninit};

pub struct TypedChunk<T: Sized> {
    _offset: usize,
    _inner: Box<[MaybeUninit<T>]>,
}
impl<T: Sized> TypedChunk<T> {
    fn new() -> Self {
        const CHUNK_SIZE: usize = 1 << 22;
        let size = size_of::<T>();
        let capacity = CHUNK_SIZE / size;
        assert!(capacity >= 1);

        TypedChunk {
            _offset: 0,
            _inner: Box::new_uninit_slice(capacity),
        }
    }

    fn has_capacity(&self) -> bool {
        self._offset < self._inner.len()
    }

    fn alloc(&mut self, value: T) -> Option<&mut T> {
        if !self.has_capacity() {
            return None;
        }

        self._inner[self._offset] = MaybeUninit::new(value);

        let res = unsafe { (self._inner[self._offset]).assume_init_mut() };
        self._offset += 1;

        Some(res)
    }
}

pub struct TypedArena<T: Sized> {
    _inner: Cell<Vec<TypedChunk<T>>>,
}

impl<T: Sized> TypedArena<T> {
    pub fn new() -> Self {
        TypedArena {
            _inner: Cell::new(Vec::new()),
        }
    }

    pub fn alloc(&self, value: T) -> Option<&mut T> {
        // in order to avoid borrowing typed area and mess with lifetimes
        // i've made
        let inner = unsafe { &mut (*self._inner.as_ptr()) };
        let last = if inner.len() > 0 && inner.last().unwrap().has_capacity() {
            inner.last_mut().unwrap()
        } else {
            let mut_inner = inner;
            mut_inner.push(TypedChunk::new());
            mut_inner.last_mut().unwrap()
        };

        last.alloc(value)
    }
    pub fn alloc_unwrap(&self, value: T) -> &mut T {
        let res = self.alloc(value);
        match res {
            Some(v) => v,
            None => panic!("failed to allocate element, because of overflow"),
        }
    }
}
