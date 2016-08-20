#![feature(libc)]

extern crate libc;
use libc::{c_char, c_int, c_ulong};

#[repr(C)]
struct InputId  {
        bustype : u16,
        vendor  : u16,
        product : u16,
        version : u16
}

#[derive(Clone, Copy)]
enum IOCTL  {
    GetId = 0x80084502,
}

fn errno() -> c_int {
    unsafe { *libc::__errno_location() }
}

fn new_input_id() -> InputId    {
    InputId {
        bustype : 0,
        vendor  : 0,
        product : 0,
        version : 0
    }
}

fn to_c_string(st : &mut String) -> *const c_char   {
    st.push('\0');
    st.as_ptr() as *const c_char
}

fn ioctl(fd : c_int, request : IOCTL, arg : *mut u8) -> c_int {
    let mut ret : c_int;

    loop    {
        ret = unsafe { libc::ioctl(fd, request as c_ulong, arg) };
        if ret == -1 && (errno() == libc::EINTR || errno() == libc::EAGAIN)
             { continue; }
        else { break;    } // Ersatz moche de do-while.
    }

    if ret < 0   {
        panic!("L’IOCTL a échoué.");
    }

    ret
}

fn main()   {
    let mut st = "/dev/input/event6".to_string();
    let pt = to_c_string(&mut st);
    let fd = unsafe { libc::open(pt, libc::O_RDONLY | libc::O_NONBLOCK) };

    if fd < 0   {
        panic!("Impossible d’ouvrir le fichier {}.", st);
    }

    let mut ii = new_input_id();

    let _ = ioctl(fd, IOCTL::GetId, &mut ii as *mut _ as *mut u8);

    println!("Input device ID: bus 0x{:x} vendor 0x{:x} product 0x{:x}",
        ii.bustype, ii.vendor, ii.product);
}
