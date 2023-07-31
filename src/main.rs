#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(atomic_from_mut)]

use crate::{bitmap_heap::BitmapAllocator, graphics::Pixel};
use crate::graphics::GPU1;
use riscv::asm::{wfi, delay};
use riscv_rt::entry;
use embedded_graphics::{pixelcolor::Rgb888, prelude::*};
use tinybmp::Bmp;
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

	GPU1.lock().init(1920, 1080, 0x80200000 as *mut u8);

	// Include the BMP file data.
	let bmp_data = include_bytes!("../peppers.bmp");

	// Parse the BMP file.
	// Note that it is necessary to explicitly specify the color type which the colors in the BMP
	// file will be converted into.
	let bmp = Bmp::<Rgb888>::from_slice(bmp_data).unwrap();

	for embedded_graphics::Pixel::<Rgb888>(position, color) in bmp.pixels() {
		//println!("R: {}, G: {}, B: {} @ ({})", color.r(), color.g(), color.b(), position);
		GPU1.lock().putpx(position.x as u32, position.y as u32, Pixel{r: color.r(),g: color.g(),b: color.b(),a: 0xFF});

	}

	loop {
		unsafe { delay(1000000000) }; GPU1.lock().update();
	}
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
