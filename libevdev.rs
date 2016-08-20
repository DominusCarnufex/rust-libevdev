#![feature(libc)]

extern crate libc;
use libc::{c_char, c_ulong};

#[repr(C)]
struct InputId  {
	    bustype : u16,
	    vendor  : u16,
	    product : u16,
	    version : u16
}

fn main()   {
    let mut st = "/dev/input/event6".to_string();
    st.push('\0');
    let pt = st.as_ptr() as *const c_char;
    let fd = unsafe { libc::open(pt, libc::O_RDONLY | libc::O_NONBLOCK) };

    let mut ii = InputId    {
	        bustype : 0,
	        vendor  : 0,
	        product : 0,
	        version : 0
    };

    let io = unsafe {
        libc::ioctl(fd, 0x80084502 as c_ulong, &mut ii as *mut _)
    };

    println!("Input device ID: bus 0x{:x} vendor 0x{:x} product 0x{:x}",
        ii.bustype, ii.vendor, ii.product);
}
