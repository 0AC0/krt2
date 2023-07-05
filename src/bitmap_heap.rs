extern crate alloc;

use core::cell::UnsafeCell;
use core::{alloc::Layout};
use core::alloc::GlobalAlloc;

use crate::{serial_println};
//use spin::Mutex;

#[global_allocator]
pub static ALLOC1: BitmapAllocator<0x100, 0x100000> = BitmapAllocator { bitmap: UnsafeCell::new([0u8; 0x100]), managed_space: UnsafeCell::new([0u8; 0x100000]) };//unsafe { (0x80100000 as *const BitmapAllocator<0x1000, 0x100000>) } ;

#[repr(C, align(0x1000))]
#[derive(Debug)]
pub struct BitmapAllocator<const BMPSZ: usize, const MSSZ: usize> {
	pub bitmap: UnsafeCell<[u8; BMPSZ]>,
	pub managed_space: UnsafeCell<[u8; MSSZ]>,
}

unsafe impl<const BMPSZ: usize, const MSSZ: usize> Sync for BitmapAllocator<BMPSZ, MSSZ> {}

unsafe impl<const BMPSZ: usize, const MSSZ: usize> GlobalAlloc for BitmapAllocator<BMPSZ, MSSZ> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		let obj_size = MSSZ / BMPSZ;
		if layout.size() > obj_size {
			serial_println!("HEAP: error alloc too big");
			return 0 as *mut u8
		}
		for i in 0..BMPSZ {
			if (*self.bitmap.get())[i] == 0 {
				(*self.bitmap.get())[i] = 1;
				serial_println!("HEAP: returning: {:#X}", (*self.managed_space.get()).as_mut_ptr().add(i * obj_size + 0x1000 - 0x100) as usize);
				return (*self.managed_space.get()).as_mut_ptr().add(i * 2 * obj_size + 0x1000 - 0x100);
			}
		}
		return 0 as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
		serial_println!("Leaking memory: {:#X}", ptr as u64);
    }
}

/*

pub struct BitmapAllocator {
	start: usize,
	end: usize,
	obj_size: usize,
	bitmap_end: usize,
}

impl BitmapAllocator {
	pub fn new(start: usize, size: usize, obj_size: usize) -> Self {
		unsafe {
		}
		BitmapAllocator {
			start: start,
			end: start + size,
			obj_size: obj_size,
			bitmap_end: size / obj_size
		}
	}

}

unsafe impl GlobalAlloc for BitmapAllocator {
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {

		'start: for bit in 0..self.bitmap_end {
			if *(self.start as *mut u8).add(bit) > 0 {
				continue;
			} else {
				for seek in 0..(self.bitmap_end) {
					if self.start + bit + seek as usize > self.end {
						serial_println!("end of heap");
						return 0 as *mut u8;
					}
					if *(self.start as *mut u8).add(bit + seek as usize) >= 1 {
						continue 'start;
					}
				}
				for seek in 0..(layout.size() / self.obj_size + 1) {
					*(self.start as *mut usize).add(bit + seek as usize) = seek + 1;
				}
				serial_println!("alloc: {:#X}({}B)", (self.start as *mut u8).add(bit * self.obj_size + self.bitmap_end) as u64, layout.size());
				return (self.start as *mut u8).add(bit * self.obj_size + self.bitmap_end);
			}
		}
		return 0 as *mut u8;
	}

	unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
		//todo!();
		serial_println!("dealloc: leaking memory: {:#X}", ptr as u64)
	}
}
*/