#[derive(Debug, Clone)]
pub struct RingBuffer<T> {
    data: Vec<T>,
    capacity: usize,
    head: usize,
    count: usize,
}

impl<T> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            capacity,
            head: 0,
            count: 0,
        }
    }

    pub fn push(&mut self, item: T) {
        if self.count < self.capacity {
            self.data.push(item);
            self.count += 1;
        } else {
            self.data[self.head] = item;
            self.head = (self.head + 1) % self.capacity;
        }
    }

    pub fn len(&self) -> usize {
        self.count.min(self.capacity)
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        let n = self.len();
        (0..n).map(move |i| {
            let idx = (self.head + i) % self.capacity;
            &self.data[idx]
        })
    }

    pub fn as_slice(&self) -> &[T] {
        &self.data[..self.len()]
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.head = 0;
        self.count = 0;
    }
}

impl<T: Clone> RingBuffer<T> {
    pub fn to_vec(&self) -> Vec<T> {
        self.iter().cloned().collect()
    }
}
