/// A typed command with an assigned sequence identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Command<T> {
    pub id: u32,
    pub payload: T,
}

impl<T> Command<T> {
    pub fn new(id: u32, payload: T) -> Self {
        Self { id, payload }
    }
}

/// A bounded, single-producer/single-consumer style command ring buffer.
///
/// Producers push commands; consumers drain them. If the buffer is full,
/// `push` returns the command unchanged so the producer can decide how to proceed.
#[derive(Debug, Clone, Default)]
pub struct CommandBuffer<T> {
    slots: Vec<Option<Command<T>>>,
    capacity: usize,
    head: usize,
    tail: usize,
    len: usize,
    next_id: u32,
}

impl<T> CommandBuffer<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        let mut slots = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            slots.push(None);
        }
        Self {
            slots,
            capacity,
            head: 0,
            tail: 0,
            len: 0,
            next_id: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn is_full(&self) -> bool {
        self.len == self.capacity
    }

    pub fn clear(&mut self) {
        for slot in &mut self.slots {
            *slot = None;
        }
        self.head = 0;
        self.tail = 0;
        self.len = 0;
    }

    pub fn push(&mut self, payload: T) -> Result<u32, (T, &'static str)> {
        if self.is_full() {
            return Err((payload, "buffer full"));
        }

        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);

        self.slots[self.tail] = Some(Command { id, payload });
        self.tail = (self.tail + 1) % self.capacity;
        self.len += 1;

        Ok(id)
    }

    pub fn drain(&mut self) -> impl Iterator<Item = Command<T>> + '_ {
        struct DrainIter<'a, T> {
            buf: &'a mut CommandBuffer<T>,
        }

        impl<'a, T> Iterator for DrainIter<'a, T> {
            type Item = Command<T>;

            fn next(&mut self) -> Option<Self::Item> {
                if self.buf.is_empty() {
                    return None;
                }

                let cmd = self.buf.slots[self.buf.head]
                    .take()
                    .expect("drain called with empty slot");
                self.buf.head = (self.buf.head + 1) % self.buf.capacity;
                self.buf.len -= 1;
                Some(cmd)
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                (self.buf.len, Some(self.buf.len))
            }
        }

        impl<'a, T> ExactSizeIterator for DrainIter<'a, T> {}

        DrainIter { buf: self }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Foo(u32);

    #[test]
    fn test_push_drain_fifo() {
        let mut buf = CommandBuffer::with_capacity(4);

        assert_eq!(buf.push(Foo(1)).unwrap(), 0);
        assert_eq!(buf.push(Foo(2)).unwrap(), 1);
        assert_eq!(buf.len(), 2);

        let drained: Vec<_> = buf.drain().collect();
        assert_eq!(drained.len(), 2);
        assert_eq!(drained[0].payload, Foo(1));
        assert_eq!(drained[1].payload, Foo(2));

        assert!(buf.is_empty());
    }

    #[test]
    fn test_overwrite_returns_err() {
        let mut buf = CommandBuffer::with_capacity(2);
        let _ = buf.push(Foo(1));
        let _ = buf.push(Foo(2));

        let result = buf.push(Foo(3));
        assert!(result.is_err());
        let (returned, _) = result.unwrap_err();
        assert_eq!(returned, Foo(3));
    }

    #[test]
    fn test_clear_resets_state() {
        let mut buf = CommandBuffer::with_capacity(4);
        let _ = buf.push("a");
        let _ = buf.push("b");
        buf.clear();

        assert!(buf.is_empty());
        assert_eq!(buf.push("c").unwrap(), 2);
    }

    #[test]
    fn test_exact_size_drain() {
        let mut buf = CommandBuffer::with_capacity(8);
        for i in 0..5 {
            let _ = buf.push(Foo(i));
        }
        let collected: Vec<_> = buf.drain().collect();
        assert_eq!(collected.len(), 5);
        assert_eq!(
            collected.iter().map(|c| c.payload).collect::<Vec<_>>(),
            vec![Foo(0), Foo(1), Foo(2), Foo(3), Foo(4)]
        );
    }
}
