extern crate alloc;

use core::mem::size_of;
use crate::serial_println;
use crate::virtio::{VirtioDev, VirtioGpuResourceCreate2d, CtrlHeader, Formats, VirtioGpuResourceAttachBacking, VirtioGpuMemEntry, CtrlType, VirtioGpuSetScanout, VirtioGpuResourceFlush, VirtioGpuRect, VirtioGpuTransferToHost2d};
use crate::virtio::queue::{Queue, Descriptor, AvailableRing, QUEUE_SIZE};
use alloc::boxed::Box;
use riscv::asm::delay;
use spin::Mutex;
use lazy_static::lazy_static;


#[repr(C)]
#[derive(Debug)]
pub struct Pixel {
	pub r: u8,
	pub g: u8,
	pub b: u8,
	pub a: u8
}

#[derive(Debug)]
pub struct GPU {
	dev: VirtioDev,
	fb: usize,
	width: u32,
	height: u32
}

impl GPU {
	pub fn init(&mut self, width: u32, height: u32, fb: *mut u8) {
		self.width = width;
		self.height = height;
		self.fb = fb as usize;

		let resource_create = VirtioGpuResourceCreate2d {
			hdr: CtrlHeader {
				ctrl_type: crate::virtio::CtrlType::CmdResourceCreate2d,
				flags: 0,
				fence_id: 0,
				ctx_id: 0,
				padding: 0
			} ,
			resource_id: 1,
			format: Formats::R8G8B8A8Unorm,
			width: width,
			height: height,
		};

		let desc = Descriptor {
			address: &resource_create as *const VirtioGpuResourceCreate2d as u64 ,
			length: size_of::<VirtioGpuResourceCreate2d>() as u32,
			flags: 1,
			next: 1
		};
		let packet = Box::new(CtrlHeader { ctrl_type: crate::virtio::CtrlType::RespErrUnspec, flags: 0, fence_id: 0, ctx_id: 0, padding: 0 });

		let response = Descriptor {
			address: &*packet as *const CtrlHeader as u64 ,
			length: size_of::<CtrlHeader>() as u32,
			flags: 2,
			next: 0
		};

		self.dev.queue.lock().desc[0] = desc;
		self.dev.queue.lock().desc[1] = response;
		self.dev.queue.lock().aring.ring[0] = 0;
		self.dev.queue.lock().aring.index = 1;

		self.dev.notify(0);

		serial_println!("Waiting for host to allocate resource...");
		//while packet.ctrl_type != crate::virtio::CtrlType::RespOkNoData {
			// TODO: sleep
		//}

		let resource_add_backing = VirtioGpuResourceAttachBacking {
			hdr: CtrlHeader { ctrl_type: crate::virtio::CtrlType::CmdResourceAttachBacking,
				flags: 0,
				fence_id: 0,
				ctx_id: 0,
				padding: 0
			},
			resource_id: 1,
			nr_entries: 1
		};

		let desc1 = Descriptor {
			address: &resource_add_backing as *const VirtioGpuResourceAttachBacking as u64 ,
			length: size_of::<VirtioGpuResourceAttachBacking>() as u32,
			flags: 1,
			next: 3
		};

		let mem_entry = VirtioGpuMemEntry {
			addr: fb as u64,
			length:  self.width * self.height,
			padding: 0,
		};

		let desc2 = Descriptor {
			address: &mem_entry as *const VirtioGpuMemEntry as u64 ,
			length: size_of::<VirtioGpuMemEntry>() as u32,
			flags: 1,
			next: 4
		};

		let packet1 = Box::new(CtrlHeader { ctrl_type: crate::virtio::CtrlType::RespErrUnspec, flags: 0, fence_id: 0, ctx_id: 0, padding: 0 });

		let response1 = Descriptor {
			address: &*packet1 as *const CtrlHeader as u64 ,
			length: size_of::<CtrlHeader>() as u32,
			flags: 2,
			next: 0
		};

		self.dev.queue.lock().desc[2] = desc1;
		self.dev.queue.lock().desc[3] = desc2;
		self.dev.queue.lock().desc[4] = response1;
		self.dev.queue.lock().aring.ring[1] = 2;
		self.dev.queue.lock().aring.index = 2;

		self.dev.notify(0);

		serial_println!("Waiting for host to attach backing...");
		//while packet1.ctrl_type != crate::virtio::CtrlType::RespOkNoData {
			// TODO: sleep
		//}

		let set_scanout = VirtioGpuSetScanout {
			hdr: CtrlHeader { ctrl_type: CtrlType::CmdSetScanout, flags: 0, fence_id: 0, ctx_id: 0, padding: 0 },
			r: crate::virtio::VirtioGpuRect { x: 0, y: 0, w: width, h: height },
			scanout_id: 0,
			resource_id: 1,
		};

		let desc3 = Descriptor {
			address: &set_scanout as *const VirtioGpuSetScanout as u64 ,
			length: size_of::<VirtioGpuSetScanout>() as u32,
			flags: 1,
			next: 6
		};

		let packet2 = Box::new(CtrlHeader { ctrl_type: crate::virtio::CtrlType::RespErrUnspec, flags: 0, fence_id: 0, ctx_id: 0, padding: 0 });

		let response2 = Descriptor {
			address: &*packet2 as *const CtrlHeader as u64 ,
			length: size_of::<CtrlHeader>() as u32,
			flags: 2,
			next: 0
		};

		self.dev.queue.lock().desc[5] = desc3;
		self.dev.queue.lock().desc[6] = response2;
		self.dev.queue.lock().aring.ring[2] = 5;
		self.dev.queue.lock().aring.index = 3;

		self.dev.notify(0);

		self.dev.queue.lock().free_desc = 7;
		self.dev.queue.lock().free_aring = 3;

	}

	pub fn transfer_rect(&mut self, x: u32, y: u32, w: u32, h: u32, offset: u64) {
		let head = self.dev.queue.lock().free_desc;
		let mut idx = self.dev.queue.lock().free_desc;

		let cmd = VirtioGpuTransferToHost2d {
			hdr: CtrlHeader { ctrl_type: CtrlType::CmdTransferToHost2d, flags: 0, fence_id: 0, ctx_id: 0, padding: 0 },
			r: VirtioGpuRect { x: x, y: y, w: w, h: h },
			offset: offset,
			resource_id: 1,
			padding: 0 };

		let mut plusone = idx + 1; if plusone == QUEUE_SIZE { plusone = 0 }
		self.dev.queue.lock().desc[idx as usize] = Descriptor { address: &cmd as *const VirtioGpuTransferToHost2d as u64,
			length: size_of::<VirtioGpuTransferToHost2d>() as u32,
			flags: 1,
			next: plusone
		};
		idx += 1; if idx == QUEUE_SIZE { idx = 0 }

		let resp = CtrlHeader { ctrl_type: CtrlType::RespErrUnspec, flags: 0, fence_id: 0, ctx_id: 0, padding: 0 };

		self.dev.queue.lock().desc[idx as usize] = Descriptor { address: &resp as *const CtrlHeader as u64,
			length: size_of::<CtrlHeader>() as u32,
			flags: 2,
			next: 0 
		};
		idx += 1; if idx == QUEUE_SIZE { idx = 0 }

		let aring_idx = self.dev.queue.lock().free_aring as usize;
		self.dev.queue.lock().aring.ring[aring_idx] = head;
		self.dev.queue.lock().free_aring += 1; if self.dev.queue.lock().free_aring == QUEUE_SIZE { self.dev.queue.lock().free_aring = 0 }
		self.dev.queue.lock().aring.index += 1; // TODO:? wrapping add
		self.dev.queue.lock().free_desc = idx;

		self.dev.notify(0);
	}

	pub fn flush_rect(&mut self, x: u32, y: u32, w: u32, h: u32) {
		let head = self.dev.queue.lock().free_desc;
		let mut idx = self.dev.queue.lock().free_desc;

		let cmd = VirtioGpuResourceFlush {
			hdr: CtrlHeader { ctrl_type: CtrlType::CmdResourceFlush, flags: 0, fence_id: 0, ctx_id: 0, padding: 0 },
			r: VirtioGpuRect { x: x, y: y, w: w, h: h },
			resource_id: 1,
			padding: 0 };

		let mut plusone = idx + 1; if plusone == QUEUE_SIZE { plusone = 0 }
		self.dev.queue.lock().desc[idx as usize] = Descriptor { address: &cmd as *const VirtioGpuResourceFlush as u64,
			length: size_of::<VirtioGpuResourceFlush>() as u32,
			flags: 1,
			next: plusone
		};
		idx += 1; if idx == QUEUE_SIZE { idx = 0 }

		let resp = CtrlHeader { ctrl_type: CtrlType::RespErrUnspec, flags: 0, fence_id: 0, ctx_id: 0, padding: 0 };

		self.dev.queue.lock().desc[idx as usize] = Descriptor { address: &resp as *const CtrlHeader as u64,
			length: size_of::<CtrlHeader>() as u32,
			flags: 2,
			next: 0 
		};
		idx += 1; if idx == QUEUE_SIZE { idx = 0 }

		let aring_idx = self.dev.queue.lock().free_aring as usize;
		self.dev.queue.lock().aring.ring[aring_idx] = head;
		self.dev.queue.lock().free_aring += 1; if self.dev.queue.lock().free_aring == QUEUE_SIZE { self.dev.queue.lock().free_aring = 0 }
		self.dev.queue.lock().aring.index += 1; // TODO:? wrapping add
		self.dev.queue.lock().free_desc = idx;

		self.dev.notify(0);
	}

	pub fn update(&mut self) {
		unsafe { delay(1000000000) }
		self.transfer_rect(0, 0, self.width, self.height, 0);
		unsafe { delay(1000000000) }
		self.flush_rect(0, 0, self.width, self.height);
	}

	pub fn putpx(&mut self, x: u32, y: u32, px: Pixel) {
		unsafe {
			(*(self.fb as *mut Pixel).add((x + self.width * y) as usize)) = px;
		}
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

