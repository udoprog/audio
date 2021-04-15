/// A runtime-checked container for the given type `T`.
struct Cell<T> {
    inner: std::cell::RefCell<T>,
}

impl<T> Cell<T> {
    /// Construct a new cell.
    fn new(value: T) -> Self {
        Self {
            inner: std::cell::RefCell::new(value),
        }
    }

    /// Acquire the cell for reading.
    fn read(&self) -> Result<std::cell::Ref<'_, T>> {
        match self.inner.try_borrow() {
            Ok(borrow) => Ok(borrow),
            Err(..) => Err(Error::BusyShared),
        }
    }

    /// Acquire the cell for writing.
    fn write(&self) -> Result<std::cell::RefMut<'_, T>> {
        match self.inner.try_borrow_mut() {
            Ok(borrow) => Ok(borrow),
            Err(..) => Err(Error::BusyShared),
        }
    }

    /// Get the mutable inner value without checking.
    ///
    /// This is permitted, because mutable access implies exclusive access.
    fn get_mut(&mut self) -> &mut T {
        self.inner.get_mut()
    }
}
