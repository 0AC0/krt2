use spin::Mutex;
use uart_16550::MmioSerialPort;
use lazy_static::lazy_static;

// TODO: use fdt for this 
const SERIAL_PORT_BASE_ADDRESS: usize = 0x10000000;

lazy_static! {
	static ref SERIAL1: Mutex<MmioSerialPort> = {
		let mut serial_port = unsafe { MmioSerialPort::new(SERIAL_PORT_BASE_ADDRESS) };
		serial_port.init();
		Mutex::new(serial_port)
	};
}

pub fn _print(args: ::core::fmt::Arguments) {
	use core::fmt::Write;
	SERIAL1.lock().write_fmt(args).expect("Printing to serial failed");
}

#[macro_export]
macro_rules! serial_print {
	($($arg:tt)*) => {
		$crate::serial::_print(format_args!($($arg)*))
	};
}

#[macro_export]
macro_rules! serial_println {
	() => ($crate::serial_print!("\n"));
	($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
	($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
		concat!($fmt, "\n"), $($arg)*));
}