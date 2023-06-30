#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(atomic_from_mut)]

use crate::bitmap_heap::BitmapAllocator;
use crate::graphics::GPU1;
use riscv::asm::wfi;
use riscv_rt::entry;
//use fdt::Fdt;

mod serial;
mod virtio;
mod bitmap_heap;
mod graphics;

/*
fn find_virtio_gpu(fdt: &fdt::Fdt) -> Option<usize> {
	for node in fdt.find_all_nodes("/soc/virtio_mmio") {
		let reg = node.reg();
		match reg {
			Some(x) => for i in x {
				unsafe {
					if *(i.starting_address as *const u32).offset(2) == 0x10 {
						return Some(i.starting_address as usize);
					}
				}
			},
			None => return None,
		}
	}
	None
}
*/

#[entry]
fn main(_hartid: usize, fdt_addr: usize) -> ! {
	serial_println!("Hart {} with fdt: {:#X}", _hartid, fdt_addr);

	GPU1.lock().init();

	loop {}
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
	serial_print!("Aborting: ");
	if let Some(p) = info.location() {
		serial_println!(
					"line {}, file {}: {}",
					p.line(),
					p.file(),
					info.message().unwrap()
		);
	} else {
		serial_println!("no information available.");
	}
	loop { unsafe { wfi() } }
}
