//! Lock-free chunk mailboxes for cross-thread ownership transfer.

use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::Arc;

/// Minimal lock-free mailbox for transferring ownership of chunks across
/// threads.
///
/// Producers push one node per chunk with a single compare-and-swap; the
/// owner takes the whole pending list with one atomic swap. There is no
/// capacity bound, no occupancy accounting, and no blocking protocol,
/// which makes it the smallest possible remote-free handoff.
#[derive(Debug)]
pub struct ChunkMailbox<T> {
    head: Arc<AtomicPtr<MailboxNode<T>>>,
}

/// Cloneable producer handle for a [`ChunkMailbox`].
#[derive(Debug)]
pub struct ChunkMailboxSender<T> {
    head: Arc<AtomicPtr<MailboxNode<T>>>,
}

#[derive(Debug)]
struct MailboxNode<T> {
    item: T,
    next: *mut MailboxNode<T>,
}

impl<T> ChunkMailbox<T> {
    /// Creates an empty mailbox.
    #[must_use]
    pub fn new() -> Self {
        Self {
            head: Arc::new(AtomicPtr::new(ptr::null_mut())),
        }
    }

    /// Returns a producer handle.
    #[must_use]
    pub fn sender(&self) -> ChunkMailboxSender<T> {
        ChunkMailboxSender {
            head: Arc::clone(&self.head),
        }
    }

    /// Takes every pending chunk, delivering them oldest first per
    /// producer, and returns how many chunks were delivered.
    pub fn take_all(&self, mut deliver: impl FnMut(T)) -> usize {
        let mut node_ptr = self.head.swap(ptr::null_mut(), Ordering::Acquire);
        if node_ptr.is_null() {
            return 0;
        }

        // Reverse the LIFO list so delivery is FIFO in push order.
        let mut reversed: *mut MailboxNode<T> = ptr::null_mut();
        while !node_ptr.is_null() {
            // SAFETY: node_ptr came from Box::into_raw in push and was
            // detached from the shared head by the swap above, so this
            // thread has exclusive ownership of the whole list.
            let node = unsafe { &mut *node_ptr };
            let next = node.next;
            node.next = reversed;
            reversed = node_ptr;
            node_ptr = next;
        }

        let mut delivered = 0_usize;
        while !reversed.is_null() {
            // SAFETY: exclusive ownership as above; every node is
            // reconstituted into a Box exactly once and dropped here.
            let node = unsafe { Box::from_raw(reversed) };
            reversed = node.next;
            deliver(node.item);
            delivered += 1;
        }
        delivered
    }
}

impl<T> Default for ChunkMailbox<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for ChunkMailbox<T> {
    fn drop(&mut self) {
        self.take_all(drop);
    }
}

impl<T> Clone for ChunkMailboxSender<T> {
    fn clone(&self) -> Self {
        Self {
            head: Arc::clone(&self.head),
        }
    }
}

impl<T> ChunkMailboxSender<T> {
    /// Pushes one chunk; lock free, one CAS on the fast path.
    pub fn push(&self, item: T) {
        let node = Box::into_raw(Box::new(MailboxNode {
            item,
            next: ptr::null_mut(),
        }));
        let mut current = self.head.load(Ordering::Relaxed);
        loop {
            // SAFETY: node is exclusively owned until the CAS below
            // publishes it, so writing next is race free.
            unsafe { (*node).next = current };
            match self.head.compare_exchange_weak(
                current,
                node,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => return,
                Err(observed) => current = observed,
            }
        }
    }
}

// SAFETY: the mailbox only transfers ownership of T values between
// threads; T: Send is required and no shared references to T escape.
unsafe impl<T: Send> Send for ChunkMailbox<T> {}
unsafe impl<T: Send> Sync for ChunkMailbox<T> {}
unsafe impl<T: Send> Send for ChunkMailboxSender<T> {}
unsafe impl<T: Send> Sync for ChunkMailboxSender<T> {}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::thread;

    use super::ChunkMailbox;

    #[test]
    fn round_trips_chunks_in_fifo_order() {
        let mailbox = ChunkMailbox::new();
        let sender = mailbox.sender();
        sender.push(vec![1_u32]);
        sender.push(vec![2, 3]);
        sender.push(vec![4]);

        let mut seen = Vec::new();
        let delivered = mailbox.take_all(|chunk| seen.push(chunk));
        assert_eq!(delivered, 3);
        assert_eq!(seen, vec![vec![1], vec![2, 3], vec![4]]);
        assert_eq!(mailbox.take_all(|_| {}), 0);
    }

    #[test]
    fn delivers_every_chunk_exactly_once_under_contention() {
        const PRODUCERS: usize = 4;
        const CHUNKS_PER_PRODUCER: usize = 10_000;

        let mailbox = ChunkMailbox::new();
        let mut workers = Vec::new();
        for producer in 0..PRODUCERS {
            let sender = mailbox.sender();
            workers.push(thread::spawn(move || {
                for sequence in 0..CHUNKS_PER_PRODUCER {
                    sender.push((producer, sequence));
                }
            }));
        }

        let mut received: Vec<Vec<usize>> = vec![Vec::new(); PRODUCERS];
        let mut total = 0_usize;
        while total < PRODUCERS * CHUNKS_PER_PRODUCER {
            total += mailbox.take_all(|(producer, sequence)| {
                received[producer].push(sequence);
            });
        }
        for worker in workers {
            worker.join().expect("producer joins");
        }
        assert_eq!(mailbox.take_all(|_| {}), 0);

        for sequences in &received {
            assert_eq!(sequences.len(), CHUNKS_PER_PRODUCER);
            // Per-producer FIFO must hold even across interleaved sweeps.
            assert!(sequences.windows(2).all(|pair| pair[0] < pair[1]));
        }
    }

    #[test]
    fn drop_reclaims_undelivered_chunks() {
        struct CountsDrops(Arc<AtomicUsize>);
        impl Drop for CountsDrops {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::Relaxed);
            }
        }

        let drops = Arc::new(AtomicUsize::new(0));
        {
            let mailbox = ChunkMailbox::new();
            let sender = mailbox.sender();
            for _ in 0..8 {
                sender.push(CountsDrops(Arc::clone(&drops)));
            }
        }
        assert_eq!(drops.load(Ordering::Relaxed), 8);
    }
}
