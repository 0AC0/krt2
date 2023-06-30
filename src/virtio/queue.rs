
pub const QUEUE_SIZE: u32 = 10;
pub const NUM_RINGS: usize = 1;


#[derive(Debug, Copy, Clone)]
pub struct Descriptor {
	pub address: u64,
	pub length: u32,
	pub flags: u16,
	pub next: u16
}

#[derive(Debug)]
pub struct AvailableRing {
	pub flags: u16,
	pub index: u16, 
	pub ring: [u16; NUM_RINGS]
}

#[derive(Debug)]
pub struct UsedRing {
	pub flags: u16,
	pub index: u16,
	pub ring: [Elem; NUM_RINGS],
	pub avail_event: u16
}

#[derive(Debug)]
pub struct Elem {
	pub id: u32,
	pub len: u32
}

#[derive(Debug)]
pub struct Queue {
	pub desc: [Descriptor; QUEUE_SIZE as usize],
	pub aring: AvailableRing,
	pub uring: UsedRing
}

impl Queue {
}

