use crate::loom::sync::atomic::{AtomicPtr, Ordering};
use std::ptr;

pub struct Node<T> {
    next: Option<ptr::NonNull<Node<T>>>,
    pub value: T,
}

impl<T> Node<T> {
    /// Construct a new wait node.
    pub fn new(value: T) -> Self {
        Self { next: None, value }
    }
}

pub struct LockFreeStack<T> {
    head: AtomicPtr<Node<T>>,
}

impl<T> LockFreeStack<T> {
    pub fn new() -> Self {
        Self {
            head: AtomicPtr::new(ptr::null_mut()),
        }
    }

    /// Steal the head pointer and construct a new queue out of it.
    pub fn steal(&self) -> Self {
        let head = self.head.swap(ptr::null_mut(), Ordering::AcqRel);

        Self {
            head: AtomicPtr::new(head),
        }
    }

    /// Push to the head of the queue.
    ///
    /// Returns `true` if the stack was empty, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ste::lock_free_stack::{LockFreeStack, Node};
    /// use std::ptr;
    ///
    /// let mut a = Node::new(0u32);
    /// let mut b = Node::new(0u32);
    ///
    /// unsafe {
    ///     let stack = LockFreeStack::new();
    ///
    ///     stack.push(ptr::NonNull::from(&mut a));
    ///     stack.push(ptr::NonNull::from(&mut b));
    ///
    ///     let mut n = 1;
    ///
    ///     while let Some(mut node) = stack.pop() {
    ///         node.as_mut().value = n;
    ///         n += 1;
    ///     }
    /// }
    ///
    /// assert_eq!(a.value, 2);
    /// assert_eq!(b.value, 1);
    /// ```
    pub fn push(&self, mut node: ptr::NonNull<Node<T>>) -> bool {
        unsafe {
            let mut head = self.head.load(Ordering::Acquire);

            loop {
                node.as_mut().next = ptr::NonNull::new(head);

                head = match self.head.compare_exchange_weak(
                    head,
                    node.as_ptr(),
                    Ordering::AcqRel,
                    Ordering::Relaxed,
                ) {
                    Ok(old) => return old.is_null(),
                    Err(head) => head,
                };
            }
        }
    }

    /// Pop the head of the wait queue.
    ///
    /// # Safety
    ///
    /// The popper has to assert that it is the only thread trying to pop from
    /// this stack. Otherwise the modification is susceptible to the ABA
    /// problem.
    pub unsafe fn pop(&self) -> Option<ptr::NonNull<Node<T>>> {
        let mut head = self.head.load(Ordering::Acquire);

        loop {
            let next = match head.as_ref()?.next {
                Some(next) => next.as_ptr(),
                None => ptr::null_mut(),
            };

            head = match self.head.compare_exchange_weak(
                head,
                next,
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                Ok(..) => break,
                Err(head) => head,
            };
        }

        Some(ptr::NonNull::new_unchecked(head))
    }
}
