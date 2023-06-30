extern crate alloc;

use core::mem::size_of;
use crate::serial_println;
use crate::virtio::{VirtioDev, VirtioGpuResourceCreate2d, CtrlHeader, Formats};
use crate::virtio::queue::{Queue, Descriptor, AvailableRing};
use alloc::boxed::Box;
use spin::Mutex;
use lazy_static::lazy_static;


#[repr(packed)]
#[derive(Debug)]
pub struct Pixel {
	r: u8,
	g: u8,
	b: u8,
	a: u8
}

#[derive(Debug)]
pub struct GPU {
	dev: VirtioDev,
	fb: usize,
	width: usize,
	height: usize
}

impl GPU {
	pub fn init(&mut self) {
		let req = VirtioGpuResourceCreate2d {
			hdr: CtrlHeader {
				ctrl_type: crate::virtio::CtrlType::CmdResourceCreate2d,
				flags: 0,
				fence_id: 0,
				ctx_id: 0,
				padding: 0
			} ,
			resource_id: 1,
			format: Formats::R8G8B8A8Unorm,
			width: 640,
			height: 480,
		};
		self.width = 640;
		self.height = 480;

		let desc = Descriptor {
			address: &req as *const VirtioGpuResourceCreate2d as u64 ,
			length: size_of::<VirtioGpuResourceCreate2d>() as u32,
			flags: 1,
			next: 1
		};
		let response = Descriptor {
			address: &*Box::new(0u64) as *const u64 as u64 ,
			length: 0x1000 as u32,
			flags: 2,
			next: 0
		};
		self.dev.queue.lock().desc[0] = desc;
		self.dev.queue.lock().desc[1] = response;

		//serial_println!("{:?}", self.dev.queue.lock());
		self.dev.notify(0);
		serial_println!("{:?}", self.dev.queue.lock());
	}
}

lazy_static! {
	pub static ref GPU1: Mutex::<GPU> = {
		let mmio = 0x10008000;

		let gpu = Mutex::new(GPU {
			// 0x10 is the gpu device id
			dev: VirtioDev::new(mmio, 0x10).unwrap(),
			fb: 0x0,
			width: 0,
			height: 0
		});
		gpu
	};
}

