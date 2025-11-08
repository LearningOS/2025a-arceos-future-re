#![no_std]

use allocator::{AllocError, BaseAllocator, ByteAllocator, PageAllocator};
use core::alloc::Layout;
use core::cmp::max;
use core::ptr::NonNull;

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const SIZE: usize> {
    start: usize,
    end: usize,
    b_pos: usize,
    p_pos: usize,
    b_count: usize,
}

impl<const SIZE: usize> EarlyAllocator<SIZE> {
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            b_pos: 0,
            p_pos: 0,
            b_count: 0,
        }
    }

    #[inline(always)]
    const fn align_up(x: usize, align: usize) -> usize {
        let mask = align - 1;
        (x + mask) & !mask
    }

    #[inline(always)]
    const fn align_down(x: usize, align: usize) -> usize {
        let mask = align - 1;
        x & !mask
    }
}

impl<const SIZE: usize> BaseAllocator for EarlyAllocator<SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start + size;
        self.b_pos = start;
        self.p_pos = self.end;
        self.b_count = 0;
    }

    fn add_memory(&mut self, start: usize, size: usize) -> allocator::AllocResult {
        // Early bump allocator does not support arbitrary expansion.
        // Allow only the case when it hasn't been initialized yet.
        if self.start == self.end {
            self.init(start, size);
            Ok(())
        } else {
            Err(AllocError::InvalidParam)
        }
    }
}

impl<const SIZE: usize> ByteAllocator for EarlyAllocator<SIZE> {
    fn alloc(
        &mut self,
        layout: Layout,
    ) -> allocator::AllocResult<NonNull<u8>> {
        let align = layout.align().max(1);
        let size = max(layout.size(), 1);

        // Align the bump pointer forward
        let start = Self::align_up(self.b_pos, align);
        let end = start.checked_add(size).ok_or(AllocError::InvalidParam)?;

        if end > self.p_pos {
            return Err(AllocError::NoMemory);
        }

        self.b_pos = end;
        self.b_count = self.b_count.saturating_add(1);
        // SAFETY: start < end <= p_pos <= self.end, so it's a valid non-null ptr
        Ok(unsafe { NonNull::new_unchecked(start as *mut u8) })
    }

    fn dealloc(&mut self, _pos: NonNull<u8>, _layout: Layout) {
        // We don't track individual byte allocations. Just keep a counter.
        if self.b_count > 0 {
            self.b_count -= 1;
            if self.b_count == 0 {
                // When all byte allocations are freed, reset the byte area.
                self.b_pos = self.start;
            }
        }
    }

    fn total_bytes(&self) -> usize {
        // Total manageable bytes in the byte area currently
        self.p_pos.saturating_sub(self.start)
    }

    fn used_bytes(&self) -> usize {
        self.b_pos.saturating_sub(self.start)
    }

    fn available_bytes(&self) -> usize {
        self.p_pos.saturating_sub(self.b_pos)
    }
}

impl<const SIZE: usize> PageAllocator for EarlyAllocator<SIZE> {
    const PAGE_SIZE: usize = SIZE;

    fn alloc_pages(
        &mut self,
        num_pages: usize,
        align_pow2: usize,
    ) -> allocator::AllocResult<usize> {
        if num_pages == 0 {
            return Err(AllocError::InvalidParam);
        }
        if align_pow2 == 0 || !align_pow2.is_power_of_two() {
            return Err(AllocError::InvalidParam);
        }

        let size_bytes = num_pages
            .checked_mul(Self::PAGE_SIZE)
            .ok_or(AllocError::InvalidParam)?;
        // Ensure at least page alignment
        let align = align_pow2.max(Self::PAGE_SIZE);

        // Allocate from the high end, keeping `start` aligned.
        let start = Self::align_down(self.p_pos.saturating_sub(size_bytes), align);
        let end = start + size_bytes;

        if end > self.p_pos || start < self.b_pos {
            return Err(AllocError::NoMemory);
        }

        self.p_pos = start;
        Ok(start)
    }

    fn dealloc_pages(&mut self, pos: usize, num_pages: usize) {
        let _ = (pos, num_pages);
        // For the early allocator, pages are not reclaimed.
    }

    fn total_pages(&self) -> usize {
        // Total pages currently manageable by the page side (used + available)
        self.end.saturating_sub(self.b_pos) / Self::PAGE_SIZE
    }

    fn used_pages(&self) -> usize {
        self.end.saturating_sub(self.p_pos) / Self::PAGE_SIZE
    }

    fn available_pages(&self) -> usize {
        self.p_pos.saturating_sub(self.b_pos) / Self::PAGE_SIZE
    }
}
