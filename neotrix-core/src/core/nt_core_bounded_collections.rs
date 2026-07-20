use std::collections::{HashSet, VecDeque};
use std::hash::Hash;

pub struct BoundedVec<T> {
    inner: Vec<T>,
    max_len: usize,
}

impl<T> BoundedVec<T> {
    pub fn new(max_len: usize) -> Self {
        Self {
            inner: Vec::with_capacity(max_len.min(1024)),
            max_len,
        }
    }

    pub fn push(&mut self, value: T) {
        if self.inner.len() >= self.max_len {
            self.inner.remove(0);
        }
        self.inner.push(value);
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_full(&self) -> bool {
        self.inner.len() >= self.max_len
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.inner.iter()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn into_inner(self) -> Vec<T> {
        self.inner
    }

    pub fn as_slice(&self) -> &[T] {
        &self.inner
    }

    pub fn drain(&mut self) -> std::vec::Drain<'_, T> {
        self.inner.drain(..)
    }

    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.inner.retain(f);
    }

    pub fn last(&self) -> Option<&T> {
        self.inner.last()
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.inner.get(index)
    }
}

impl<T> IntoIterator for BoundedVec<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a BoundedVec<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

pub struct BoundedQueue<T> {
    inner: VecDeque<T>,
    max_len: usize,
}

impl<T> BoundedQueue<T> {
    pub fn new(max_len: usize) -> Self {
        Self {
            inner: VecDeque::with_capacity(max_len.min(1024)),
            max_len,
        }
    }

    pub fn push_back(&mut self, value: T) {
        if self.inner.len() >= self.max_len {
            self.inner.pop_front();
        }
        self.inner.push_back(value);
    }

    pub fn push_front(&mut self, value: T) {
        if self.inner.len() >= self.max_len {
            self.inner.pop_back();
        }
        self.inner.push_front(value);
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.inner.pop_front()
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.inner.pop_back()
    }

    pub fn front(&self) -> Option<&T> {
        self.inner.front()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.inner.len() >= self.max_len
    }

    pub fn iter(&self) -> std::collections::vec_deque::Iter<'_, T> {
        self.inner.iter()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn drain(&mut self) -> std::collections::vec_deque::Drain<'_, T> {
        self.inner.drain(..)
    }

    pub fn max_len(&self) -> usize {
        self.max_len
    }

    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.inner.retain(f);
    }
}

impl<T> IntoIterator for BoundedQueue<T> {
    type Item = T;
    type IntoIter = std::collections::vec_deque::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

pub struct BoundedSet<T> {
    inner: HashSet<T>,
    max_len: usize,
    insertion_order: VecDeque<T>,
}

impl<T: Clone + Eq + Hash> BoundedSet<T> {
    pub fn new(max_len: usize) -> Self {
        Self {
            inner: HashSet::with_capacity(max_len.min(1024)),
            max_len,
            insertion_order: VecDeque::with_capacity(max_len.min(1024)),
        }
    }

    pub fn insert(&mut self, value: T) -> bool {
        if self.inner.len() >= self.max_len {
            if let Some(oldest) = self.insertion_order.pop_front() {
                self.inner.remove(&oldest);
            }
        }
        let is_new = self.inner.insert(value.clone());
        if is_new {
            self.insertion_order.push_back(value);
        }
        is_new
    }

    pub fn contains(&self, value: &T) -> bool {
        self.inner.contains(value)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_full(&self) -> bool {
        self.inner.len() >= self.max_len
    }

    pub fn clear(&mut self) {
        self.inner.clear();
        self.insertion_order.clear();
    }

    pub fn iter(&self) -> std::collections::hash_set::Iter<'_, T> {
        self.inner.iter()
    }
}

pub struct SlidingWindow<T> {
    inner: VecDeque<T>,
    max_len: usize,
}

impl<T> SlidingWindow<T> {
    pub fn new(max_len: usize) -> Self {
        Self {
            inner: VecDeque::with_capacity(max_len.min(1024)),
            max_len,
        }
    }

    pub fn push(&mut self, value: T) {
        if self.inner.len() >= self.max_len {
            self.inner.pop_front();
        }
        self.inner.push_back(value);
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_full(&self) -> bool {
        self.inner.len() >= self.max_len
    }

    pub fn iter(&self) -> std::collections::vec_deque::Iter<'_, T> {
        self.inner.iter()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn drain(&mut self) -> std::collections::vec_deque::Drain<'_, T> {
        self.inner.drain(..)
    }

    pub fn back(&self) -> Option<&T> {
        self.inner.back()
    }

    pub fn front(&self) -> Option<&T> {
        self.inner.front()
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.inner.get(index)
    }

    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.inner.retain(f);
    }

    pub fn to_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        self.inner.iter().cloned().collect()
    }

    pub fn max_len(&self) -> usize {
        self.max_len
    }
}

impl<T> IntoIterator for SlidingWindow<T> {
    type Item = T;
    type IntoIter = std::collections::vec_deque::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a SlidingWindow<T> {
    type Item = &'a T;
    type IntoIter = std::collections::vec_deque::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounded_vec_evicts_oldest() {
        let mut v = BoundedVec::new(3);
        v.push(1);
        v.push(2);
        v.push(3);
        v.push(4);
        assert_eq!(v.len(), 3);
        assert_eq!(v.as_slice(), &[2, 3, 4]);
    }

    #[test]
    fn test_bounded_vec_under_capacity() {
        let mut v = BoundedVec::new(10);
        v.push(1);
        v.push(2);
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn test_bounded_queue_evicts_front() {
        let mut q = BoundedQueue::new(3);
        q.push_back(1);
        q.push_back(2);
        q.push_back(3);
        q.push_back(4);
        assert_eq!(q.len(), 3);
        assert_eq!(*q.front().unwrap(), 2);
    }

    #[test]
    fn test_bounded_queue_push_front() {
        let mut q = BoundedQueue::new(3);
        q.push_front(1);
        q.push_front(2);
        q.push_front(3);
        q.push_front(4);
        assert_eq!(q.len(), 3);
        assert_eq!(*q.front().unwrap(), 4);
    }

    #[test]
    fn test_bounded_set_evicts_oldest() {
        let mut s = BoundedSet::new(3);
        s.insert("a");
        s.insert("b");
        s.insert("c");
        s.insert("d");
        assert_eq!(s.len(), 3);
        assert!(!s.contains(&"a"));
        assert!(s.contains(&"d"));
    }

    #[test]
    fn test_sliding_window_keeps_last_n() {
        let mut w = SlidingWindow::new(3);
        w.push(1);
        w.push(2);
        w.push(3);
        w.push(4);
        w.push(5);
        assert_eq!(w.len(), 3);
        let v: Vec<_> = w.iter().collect();
        assert_eq!(v, vec![&3, &4, &5]);
    }

    #[test]
    fn test_sliding_window_under_capacity() {
        let mut w = SlidingWindow::new(10);
        w.push(1);
        w.push(2);
        assert_eq!(w.len(), 2);
    }

    #[test]
    fn test_bounded_vec_clear() {
        let mut v = BoundedVec::new(5);
        v.push(1);
        v.push(2);
        v.clear();
        assert_eq!(v.len(), 0);
    }

    #[test]
    fn test_bounded_queue_pop() {
        let mut q = BoundedQueue::new(5);
        q.push_back(1);
        q.push_back(2);
        assert_eq!(q.pop_front(), Some(1));
        assert_eq!(q.len(), 1);
    }

    #[test]
    fn test_bounded_set_duplicate() {
        let mut s = BoundedSet::new(5);
        assert!(s.insert("a"));
        assert!(!s.insert("a"));
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn test_bounded_vec_drain() {
        let mut v = BoundedVec::new(5);
        v.push(1);
        v.push(2);
        v.push(3);
        let drained: Vec<_> = v.drain().collect();
        assert_eq!(drained, vec![1, 2, 3]);
        assert_eq!(v.len(), 0);
    }

    #[test]
    fn test_sliding_window_to_vec() {
        let mut w = SlidingWindow::new(3);
        w.push(10);
        w.push(20);
        assert_eq!(w.to_vec(), vec![10, 20]);
    }

    #[test]
    fn test_bounded_queue_is_full() {
        let mut q = BoundedQueue::new(2);
        assert!(!q.is_full());
        q.push_back(1);
        q.push_back(2);
        assert!(q.is_full());
    }
}
