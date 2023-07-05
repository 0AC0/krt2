
extern crate alloc;
use alloc::boxed::Box;
use alloc::sync::Arc;
use spin::Mutex;
use crate::serial_println;
use crate::virtio::queue::QUEUE_SIZE;
use crate::virtio::queue::Queue;
use crate::virtio::queue::Descriptor;
use crate::virtio::queue::AvailableRing;
use crate::virtio::queue::UsedRing;
use crate::virtio::queue::Elem;
use core::mem::size_of;
use core::panic;
pub mod queue;

const PAGE_SIZE: usize = 0x1000;

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum Formats {
   B8G8R8A8Unorm = 1,
   B8G8R8X8Unorm = 2,
   A8R8G8B8Unorm = 3,
   X8R8G8B8Unorm = 4,
   R8G8B8A8Unorm = 67,
   X8B8G8R8Unorm = 68,
   A8B8G8R8Unorm = 121,
   R8G8B8X8Unorm = 134,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
pub enum CtrlType {
   /* 2d commands */
   CmdGetDisplayInfo = 0x0100,
   CmdResourceCreate2d,
   CmdResourceUref,
   CmdSetScanout,
   CmdResourceFlush,
   CmdTransferToHost2d,
   CmdResourceAttachBacking,
   CmdResourceDetachBacking,
   CmdGetCapsetInfo,
   CmdGetCapset,
   CmdGetEdid,
   /* cursor commands */
   CmdUpdateCursor = 0x0300,
   CmdMoveCursor,
   /* success responses */
   RespOkNoData = 0x1100,
   RespOkDisplayInfo,
   RespOkCapsetInfo,
   RespOkCapset,
   RespOkEdid,
   /* error responses */
   RespErrUnspec = 0x1200,
   RespErrOutOfMemory,
   RespErrInvalidScanoutId,
   RespErrInvalidResourceId,
   RespErrInvalidContextId,
   RespErrInvalidParameter,
}

#[repr(packed)]
#[derive(Debug, Clone, Copy)]
pub struct CtrlHeader {
   pub ctrl_type: CtrlType,
   pub flags: u32,
   pub fence_id: u64,
   pub ctx_id: u32,
   pub padding: u32
}

#[repr(packed)]
#[derive(Debug)]
pub struct VirtioGpuResourceCreate2d {
	pub hdr: CtrlHeader,
	pub resource_id: u32,
	pub format: Formats,
	pub width: u32,
	pub height: u32,
}


#[repr(packed)]
#[derive(Debug)]
pub struct VirtioGpuResourceAttachBacking {
	pub hdr: CtrlHeader,
	pub resource_id: u32,
	pub nr_entries: u32
}

#[repr(packed)]
#[derive(Debug)]
pub struct VirtioGpuMemEntry {
	pub addr: u64,
	pub length: u32,
	pub padding: u32,
}

#[repr(packed)]
#[derive(Debug)]
pub struct VirtioGpuSetScanout {
	pub hdr: CtrlHeader,
	pub r: VirtioGpuRect,
	pub scanout_id: u32,
	pub resource_id: u32,
}

#[repr(packed)]
#[derive(Debug)]
pub struct VirtioGpuResourceFlush {
	pub hdr: CtrlHeader,
	pub r: VirtioGpuRect,
	pub resource_id: u32,
	pub padding: u32,
}

#[repr(packed)]
#[derive(Debug)]
pub struct VirtioGpuTransferToHost2d {
	pub hdr: CtrlHeader,
	pub r: VirtioGpuRect,
	pub offset: u64,
	pub resource_id: u32,
	pub padding: u32,
}

#[repr(packed)]
#[derive(Debug, Clone, Copy)]
pub struct VirtioGpuRect {
	pub x: u32,
	pub y: u32,
	pub w: u32,
	pub h: u32,
}

#[repr(u32)]
enum Status {
	Acknowledge			 =     1,
	Driver				 =     2,
	Failed				 =   128,
	FeaturesOk			 =     8,
	DriverOk			 =     4,
	DeviceNeedsReset	 =    64,
}

#[repr(usize)]
enum MmioOffsets {
	MagicValue			 = 0x000,
	Version				 = 0x004,
	DeviceId			 = 0x008,
	VendorId			 = 0x00c,
	HostFeatures		 = 0x010,
	HostFeaturesSel		 = 0x014,
	GuestFeatures		 = 0x020,
	GuestFeaturesSel	 = 0x024,
	GuestPageSize		 = 0x028,
	QueueSel			 = 0x030,
	QueueNumMax			 = 0x034,
	QueueNum			 = 0x038,
	QueueAlign			 = 0x03c,
	QueuePfn			 = 0x040,
	QueueNotify			 = 0x050,
	InterruptStatus		 = 0x060,
	InterruptAck		 = 0x064,
	Status				 = 0x070,
	Config				 = 0x100
}

impl MmioOffsets {
	fn scale<T>(self) -> usize {
		self as usize / size_of::<T>()
	}
}

#[derive(Debug)]
pub struct VirtioDev {
	pub mmio: usize,
	pub queue: Arc<Mutex<Box<Queue>>>
}

impl VirtioDev {
	pub fn new(ptr: usize, devid: u32) -> Result<Self, &'static str> {

		unsafe {
			if (ptr as *mut u32).add(MmioOffsets::MagicValue.scale::<u32>()).read_volatile() != 0x74726976 {
				return Err("Bad MagicValue");
			}
			// using legacy interface, version 1
			if (ptr as *mut u32).add(MmioOffsets::Version.scale::<u32>()).read_volatile() != 1 {
				return Err("Bad virtio version");
			}
			if (ptr as *mut u32).add(MmioOffsets::DeviceId.scale::<u32>()).read_volatile() != devid {
				return Err("Not the right device");
			}

			// reset device by writing 0 to status field
			(ptr as *mut u32).add(MmioOffsets::Status.scale::<u32>()).write_volatile(0);

			let mut stat: u32 = Status::Acknowledge as u32;
			(ptr as *mut u32).add(MmioOffsets::Status.scale::<u32>()).write_volatile(stat);
			stat |= Status::Driver as u32;
			(ptr as *mut u32).add(MmioOffsets::Status.scale::<u32>()).write_volatile(stat);

			let host_features = (ptr as *mut u32).add(MmioOffsets::HostFeatures.scale::<u32>()).read_volatile();
			let guest_features = host_features;
			(ptr as *mut u32).add(MmioOffsets::GuestFeatures.scale::<u32>()).write_volatile(guest_features);

			stat |= Status::FeaturesOk as u32;
			(ptr as *mut u32).add(MmioOffsets::Status.scale::<u32>()).write_volatile(stat);
			let status_ok = (ptr as *mut u32).add(MmioOffsets::Status.scale::<u32>()).read_volatile();
			if status_ok & Status::FeaturesOk as u32 == 0 {
				todo!("fail gracefully using Status::Failed");
				//return Err("Could not negotiate features");
			}

			(ptr as *mut u32).add(MmioOffsets::QueueSel.scale::<u32>()).write_volatile(0);

			let q_max = (ptr as *mut u32).add(MmioOffsets::QueueNumMax.scale::<u32>()).read_volatile();
			(ptr as *mut u32).add(MmioOffsets::QueueNum.scale::<u32>()).write_volatile(QUEUE_SIZE as u32);

			(ptr as *mut u32).add(MmioOffsets::GuestPageSize.scale::<u32>()).write_volatile(PAGE_SIZE as u32);

			let q_ptr = Arc::new(Mutex::new(Box::new(Queue {
				desc: [Descriptor { address: 0, length: 0, flags: 0, next: 0  }; QUEUE_SIZE as usize] ,
				aring: AvailableRing { flags: 0, index: 0, ring: [0; QUEUE_SIZE as usize] } ,
				uring: UsedRing { flags: 0, index: 0, ring: [Elem { id: 0, len: 0 }; QUEUE_SIZE as usize], avail_event: 0 },
				free_desc: 0,
				free_aring: 0,
			})));
			let q_pfn = q_ptr.lock().as_ref() as *const Queue as u32 / PAGE_SIZE as u32;
			(ptr as *mut u32).add(MmioOffsets::QueuePfn.scale::<u32>()).write_volatile(q_pfn);

			stat |= Status::DriverOk as u32;
			(ptr as *mut u32).add(MmioOffsets::Status.scale::<u32>()).write_volatile(stat);
		
			return Ok( VirtioDev { mmio: ptr, queue: q_ptr } )
		}
	}

	pub fn notify(&self, i: u32) {
		unsafe {
			(self.mmio as *mut u32).add(MmioOffsets::QueueNotify.scale::<u32>()).write_volatile(i);
		}
	}
}
