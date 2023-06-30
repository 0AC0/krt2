extern crate alloc;

use core::{ptr::write_bytes, alloc::Layout};
use core::alloc::GlobalAlloc;

use crate::{serial_println, serial_print};
//use spin::Mutex;

#[global_allocator]
pub static ALLOC1: BitmapAllocator = BitmapAllocator {
	start: 0x80100000,
	end: 0x80300000,
	obj_size: 0x1000,
	bitmap_end: 0x1000
	//BitmapAllocator::new(0x80100000, 0x1000000, 0x1000)
};

pub struct BitmapAllocator {
	start: usize,
	end: usize,
	obj_size: usize,
	bitmap_end: usize,
}

/*
impl BitmapAllocator {
	pub fn new(start: usize, size: usize, obj_size: usize) -> Self {
		unsafe {
			write_bytes::<u8>(start as *mut u8, 0, size);
		}
		BitmapAllocator {
			start: start,
			end: start + size,
			obj_size: obj_size,
			bitmap_end: size / obj_size
		}
	}

}
*/

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
				for seek in 0..(layout.size() / self.obj_size) {
					*(self.start as *mut usize).add(bit + seek as usize) = seek + 1;
				}
				return (self.start as *mut u8).add(bit * self.obj_size + self.bitmap_end);
			}
		}
		return 0 as *mut u8;
	}

	unsafe fn dealloc(&self, _: *mut u8, layout: Layout) {
		todo!();
	}
}