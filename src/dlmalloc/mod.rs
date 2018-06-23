use std::alloc::{Alloc, Layout, AllocErr};
use std::cmp;
use std::ptr;

pub use self::global::GlobalDlmalloc;

mod global;
mod dlmalloc;
mod sys;

pub struct Dlmalloc(dlmalloc::Dlmalloc);

impl Dlmalloc {
    #[inline]
    pub unsafe fn malloc(&mut self, size: usize, align: usize) -> *mut u8 {
        if align <= self.0.malloc_alignment() {
            self.0.malloc(size)
        } else {
            self.0.memalign(align, size)
        }
    }

    #[inline]
    pub unsafe fn calloc(&mut self, size: usize, align: usize) -> *mut u8 {
        let ptr = self.malloc(size, align);
        if !ptr.is_null() && self.0.calloc_must_clear(ptr) {
            ptr::write_bytes(ptr, 0, size);
        }
        ptr
    }

    #[inline]
    pub unsafe fn free(&mut self, ptr: *mut u8, size: usize, align: usize) {
        drop((size, align));
        self.0.free(ptr)
    }

    #[inline]
    pub unsafe fn realloc(&mut self,
                          ptr: *mut u8,
                          old_size: usize,
                          old_align: usize,
                          new_size: usize) -> *mut u8 {
        if old_align <= self.0.malloc_alignment() {
            self.0.realloc(ptr, new_size)
        } else {
            let res = self.malloc(new_size, old_align);
            if !res.is_null() {
                let size = cmp::min(old_size, new_size);
                ptr::copy_nonoverlapping(ptr, res, size);
                self.free(ptr, old_size, old_align);
            }
            res
        }
    }
}

unsafe impl Alloc for Dlmalloc {
    #[inline]
    unsafe fn alloc(
        &mut self,
        layout: Layout
    ) -> Result<ptr::NonNull<u8>, AllocErr> {
        let ptr = <Dlmalloc>::malloc(self, layout.size(), layout.align());
        ptr::NonNull::new(ptr as *mut u8).ok_or(AllocErr)
    }

    #[inline]
    unsafe fn dealloc(&mut self, ptr: ptr::NonNull<u8>, layout: Layout) {
        <Dlmalloc>::free(self, ptr.as_ptr() as *mut u8, layout.size(), layout.align())
    }

    #[inline]
    unsafe fn realloc(
        &mut self,
        ptr: ptr::NonNull<u8>,
        layout: Layout,
        new_size: usize
    ) -> Result<ptr::NonNull<u8>, AllocErr> {
        let ptr = <Dlmalloc>::realloc(
            self,
            ptr.as_ptr() as *mut u8,
            layout.size(),
            layout.align(),
            new_size,
        );
        ptr::NonNull::new(ptr as *mut u8).ok_or(AllocErr)
    }

    #[inline]
    unsafe fn alloc_zeroed(
        &mut self,
        layout: Layout
    ) -> Result<ptr::NonNull<u8>, AllocErr> {
        let ptr = <Dlmalloc>::calloc(self, layout.size(), layout.align());
        ptr::NonNull::new(ptr as *mut u8).ok_or(AllocErr)
    }
}
