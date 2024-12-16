//! This module supports memory size measurement for `Vec` and `hashbrown::HashMap`.
//! It provides a custom memory allocator to track allocated and deallocated memory sizes
//! and also offers a separate `Vec` type that allows specifying the memory allocator during creation.

use allocator_api2::alloc::{AllocError, Allocator};
pub use allocator_api2::vec::Vec;
use std::{
    alloc::{GlobalAlloc, System, Layout},
    sync::atomic::{AtomicUsize, Ordering},
    ptr::NonNull,
    slice,
};

static ALLOC: AtomicUsize = AtomicUsize::new(0);
static DEALLOC: AtomicUsize = AtomicUsize::new(0);

/// A custom allocator that tracks memory allocations and deallocations.
#[derive(Debug, Copy, Clone, Default)]
pub struct TrackingAllocator;

impl TrackingAllocator {
    /// Resets the allocation and deallocation counters to zero.
    pub fn reset() {
        ALLOC.store(0, Ordering::SeqCst);
        DEALLOC.store(0, Ordering::SeqCst);
    }

    /// Records an allocation of a given size.
    pub fn record_alloc(layout: Layout) {
        ALLOC.fetch_add(layout.size(), Ordering::SeqCst);
    }

    /// Records a deallocation of a given size.
    pub fn record_dealloc(layout: Layout) {
        DEALLOC.fetch_add(layout.size(), Ordering::SeqCst);
    }

    /// Retrieves the current memory statistics.
    pub fn stats() -> Stats {
        let alloc = ALLOC.load(Ordering::SeqCst);
        let dealloc = DEALLOC.load(Ordering::SeqCst);
        let diff = (alloc as isize) - (dealloc as isize);

        Stats { alloc, dealloc, diff }
    }
}

/// Memory usage statistics for the allocator.
pub struct Stats {
    pub alloc: usize,
    pub dealloc: usize,
    pub diff: isize,
}

unsafe impl Allocator for TrackingAllocator {
    /// Allocates memory using the system allocator and tracks the allocation.
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        unsafe {
            let ptr = System.alloc(layout);
            if ptr.is_null() {
                Err(AllocError)
            } else {
                let slice_ptr: *mut [u8] = slice::from_raw_parts_mut(ptr, layout.size());
                let non_null_slice: NonNull<[u8]> = NonNull::new_unchecked(slice_ptr);
                Self::record_alloc(layout);

                Ok(non_null_slice)
            }
        }
    }

    /// Deallocates memory using the system allocator and tracks the deallocation.
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        Self::record_dealloc(layout);
        let raw_ptr: *mut u8 = ptr.as_ptr();
        System.dealloc(raw_ptr, layout);
    }
}
