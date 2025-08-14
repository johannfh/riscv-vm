use core::sync::atomic::{self, AtomicUsize};
use std::alloc::Allocator;

pub struct MonitoredAllocator {
    inner: std::alloc::System,
    limit: usize,
    allocated: AtomicUsize,
}

unsafe impl Allocator for MonitoredAllocator {
    fn allocate(
        &self,
        layout: std::alloc::Layout,
    ) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        debug!("Attempting to allocate {} bytes", layout.size());

        // Check if the allocation exceeds the limit
        let current_allocated = self.allocated.load(atomic::Ordering::Relaxed);
        if current_allocated + layout.size() > self.limit {
            error!(
                "Allocation of {} bytes exceeds limit, {}+{} B = {} B > {} B (max)",
                layout.size(),
                current_allocated,
                layout.size(),
                current_allocated + layout.size(),
                self.limit
            );
            return Err(std::alloc::AllocError);
        }

        // Proceed with the allocation, incrementing the allocated size tracker
        self.allocated
            .fetch_add(layout.size(), atomic::Ordering::Relaxed);

        self.inner.allocate(layout)
    }

    unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, layout: std::alloc::Layout) {
        debug!("Deallocating {} bytes", layout.size());
        // Decrement the allocated size tracker
        self.allocated
            .fetch_sub(layout.size(), atomic::Ordering::Relaxed);
        unsafe { self.inner.deallocate(ptr, layout) }
    }
}

impl MonitoredAllocator {
    pub const fn new(limit: usize) -> Self {
        Self {
            inner: std::alloc::System,
            limit,
            allocated: AtomicUsize::new(0),
        }
    }

    /// Returns the currently allocated memory size.
    pub fn allocated(&self) -> usize {
        self.allocated.load(atomic::Ordering::Relaxed)
    }

    /// Returns the current limit of the allocator.
    ///
    /// # Returns
    ///
    /// The maximum amount of memory that can be allocated by this allocator.
    /// If the limit is reached, further allocations will return null pointers.
    ///
    /// # Example
    ///
    /// ```
    /// use constrained_allocator::ConstrainedAllocator;
    /// let allocator = ConstrainedAllocator::new(1024 * 1024); // 1 MB limit
    /// assert_eq!(allocator.limit(), 1024 * 1024);
    /// ```
    #[inline]
    pub const fn limit(&self) -> usize {
        self.limit
    }

    /// Sets a new limit for the allocator.
    ///
    /// # Panics
    ///
    /// * If the new limit is less than the currently allocated memory.
    pub fn set_limit(&mut self, new_limit: usize) {
        if new_limit < self.allocated() {
            panic!("New limit cannot be less than currently allocated memory.");
        }
        self.limit = new_limit;
    }
}

/// Default limit for the [`MONITORED_ALLOCATOR`].
pub const DEFAULT_LIMIT: usize = 1024 * 1024;

/// A global instance of the constrained allocator with a default limit of [`DEFAULT_LIMIT`]
/// This allocator is used inside of the Virtual Machine to limit and monitor memory usage.
pub static MONITORED_ALLOCATOR: MonitoredAllocator = MonitoredAllocator::new(DEFAULT_LIMIT);
