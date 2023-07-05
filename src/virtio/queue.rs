
pub const QUEUE_SIZE: u16 = 10;


#[derive(Debug, Copy, Clone)]
#[repr(packed)]
pub struct Descriptor {
	pub address: u64,
	pub length: u32,
	pub flags: u16,
	pub next: u16
}

#[derive(Debug, Clone, Copy)]
#[repr(packed)]
pub struct AvailableRing {
	pub flags: u16,
	pub index: u16, 
	pub ring: [u16; QUEUE_SIZE as usize]
}

#[derive(Debug, Clone, Copy)]
#[repr(packed)]
pub struct UsedRing {
	pub flags: u16,
	pub index: u16,
	pub ring: [Elem; QUEUE_SIZE as usize],
	pub avail_event: u16
}

#[derive(Debug, Copy, Clone)]
#[repr(packed)]
pub struct Elem {
	pub id: u32,
	pub len: u32
}

#[derive(Debug)]
#[repr(packed)]
pub struct Queue {
	pub desc: [Descriptor; QUEUE_SIZE as usize],
	pub aring: AvailableRing,
	pub uring: UsedRing,
	pub free_desc: u16,
	pub free_aring: u16,
}

impl Queue {
}

