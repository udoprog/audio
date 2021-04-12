//! An intrusive linked list.

use std::ptr;

/// A node in the intrusive [LinkedList].
pub struct Node<T> {
    next: Option<ptr::NonNull<Node<T>>>,
    prev: Option<ptr::NonNull<Node<T>>>,
    pub value: T,
}

impl<T> Node<T> {
    /// Construct a new wait node.
    pub fn new(value: T) -> Self {
        Self {
            next: None,
            prev: None,
            value,
        }
    }
}

/// An intrusive linked list.
///
/// This is an exceedingly unsafe collection that allows you to construct and
/// reason about lists out of data stored somewhere else.
pub struct LinkedList<T> {
    first: Option<ptr::NonNull<Node<T>>>,
    last: Option<ptr::NonNull<Node<T>>>,
}

impl<T> LinkedList<T> {
    /// Construct a new empty linked list.
    pub fn new() -> Self {
        Self {
            first: None,
            last: None,
        }
    }

    /// Test if the linked list is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ste::linked_list::LinkedList;
    /// let mut list = LinkedList::<u32>::new();
    /// assert!(list.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.first.is_none()
    }

    /// Steal the entire contents of the linked list, removing it from the list
    /// that we stole it from.
    pub fn steal(&mut self) -> Self {
        Self {
            first: self.first.take(),
            last: self.last.take(),
        }
    }

    /// Push to the front of the linked list.
    ///
    /// Returns a boolean that if `true` indicates that this was the first
    /// element in the list.
    ///
    /// # Safety
    ///
    /// The soundness of manipulating the data in the list depends entirely on
    /// what was pushed. If you intend to mutate the data, you must push a
    /// pointer that is based out of something that was exclusively borrowed
    /// (example below).
    ///
    /// The caller also must ensure that the data pushed doesn't outlive its
    /// use.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::ptr;
    /// use ste::linked_list::{Node, LinkedList};
    ///
    /// let mut list = LinkedList::new();
    ///
    /// let mut a = Node::new(0);
    /// let mut b = Node::new(0);
    ///
    /// unsafe {
    ///     list.push_front(ptr::NonNull::from(&mut a));
    ///     list.push_front(ptr::NonNull::from(&mut b));
    ///
    ///     let mut n = 1;
    ///
    ///     while let Some(mut last) = list.pop_back() {
    ///         last.as_mut().value += n;
    ///         n <<= 1;
    ///     }
    /// }
    ///
    /// assert_eq!(a.value, 1);
    /// assert_eq!(b.value, 2);
    /// ```
    pub unsafe fn push_front(&mut self, mut node: ptr::NonNull<Node<T>>) -> bool {
        if let Some(mut first) = self.first.take() {
            node.as_mut().next = Some(first);
            first.as_mut().prev = Some(node);
            self.first = Some(node);
            false
        } else {
            self.first = Some(node);
            self.last = Some(node);
            true
        }
    }

    /// Push to the front of the linked list.
    ///
    /// Returns a boolean that if `true` indicates that this was the first
    /// element in the list.
    ///
    /// # Safety
    ///
    /// The soundness of manipulating the data in the list depends entirely on
    /// what was pushed. If you intend to mutate the data, you must push a
    /// pointer that is based out of something that was exclusively borrowed
    /// (example below).
    ///
    /// The caller also must ensure that the data pushed doesn't outlive its
    /// use.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::ptr;
    /// use ste::linked_list::{Node, LinkedList};
    ///
    /// let mut list = LinkedList::new();
    ///
    /// let mut a = Node::new(0);
    /// let mut b = Node::new(0);
    ///
    /// unsafe {
    ///     list.push_back(ptr::NonNull::from(&mut a));
    ///     list.push_back(ptr::NonNull::from(&mut b));
    ///
    ///     let mut n = 1;
    ///
    ///     while let Some(mut last) = list.pop_back() {
    ///         last.as_mut().value += n;
    ///         n <<= 1;
    ///     }
    /// }
    ///
    /// assert_eq!(a.value, 2);
    /// assert_eq!(b.value, 1);
    /// ```
    pub unsafe fn push_back(&mut self, mut node: ptr::NonNull<Node<T>>) -> bool {
        if let Some(mut last) = self.last.take() {
            node.as_mut().prev = Some(last);
            last.as_mut().next = Some(node);
            self.last = Some(node);
            false
        } else {
            self.first = Some(node);
            self.last = Some(node);
            true
        }
    }

    /// Pop the front element from the list.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::ptr;
    /// use ste::linked_list::{Node, LinkedList};
    ///
    /// let mut list = LinkedList::new();
    ///
    /// let mut a = Node::new(0);
    /// let mut b = Node::new(0);
    ///
    /// unsafe {
    ///     list.push_back(ptr::NonNull::from(&mut a));
    ///     list.push_back(ptr::NonNull::from(&mut b));
    ///
    ///     let mut n = 1;
    ///
    ///     while let Some(mut last) = list.pop_front() {
    ///         last.as_mut().value += n;
    ///         n <<= 1;
    ///     }
    /// }
    ///
    /// assert_eq!(a.value, 1);
    /// assert_eq!(b.value, 2);
    /// ```
    pub unsafe fn pop_front(&mut self) -> Option<ptr::NonNull<Node<T>>> {
        let mut first = self.first?;

        if let Some(mut next) = first.as_mut().next.take() {
            next.as_mut().prev = None;
            self.first = Some(next);
        } else {
            self.first = None;
            self.last = None;
        }

        debug_assert!(first.as_ref().prev.is_none());
        debug_assert!(first.as_ref().next.is_none());
        Some(first)
    }

    /// Pop the back element from the list.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::ptr;
    /// use ste::linked_list::{Node, LinkedList};
    ///
    /// let mut list = LinkedList::new();
    ///
    /// let mut a = Node::new(0);
    /// let mut b = Node::new(0);
    ///
    /// unsafe {
    ///     list.push_back(ptr::NonNull::from(&mut a));
    ///     list.push_back(ptr::NonNull::from(&mut b));
    ///
    ///     let mut n = 1;
    ///
    ///     while let Some(mut last) = list.pop_back() {
    ///         last.as_mut().value += n;
    ///         n <<= 1;
    ///     }
    /// }
    ///
    /// assert_eq!(a.value, 2);
    /// assert_eq!(b.value, 1);
    /// ```
    pub unsafe fn pop_back(&mut self) -> Option<ptr::NonNull<Node<T>>> {
        let mut last = self.last?;

        if let Some(mut prev) = last.as_mut().prev.take() {
            prev.as_mut().next = None;
            self.last = Some(prev);
        } else {
            self.first = None;
            self.last = None;
        }

        debug_assert!(last.as_ref().prev.is_none());
        debug_assert!(last.as_ref().next.is_none());
        Some(last)
    }
}
