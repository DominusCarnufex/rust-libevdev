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
    GetId      = 0x80084502,
    GetVersion = 0x80044501,
    GetBits    = 0x80084520,
    GetKeyBits = 0x80604521,
}

#[derive(Clone, Copy, PartialEq)]
enum EventType  {
    Synchro       = 0x00,
    Key           = 0x01,
    Relative      = 0x02,
    Absolute      = 0x03,
    Miscellaneous = 0x04,
    Switch        = 0x05,
    LED           = 0x11,
    Sound         = 0x12,
    Repeat        = 0x14,
    FF            = 0x15, // Pas trouvé à quoi cela correspond.
    Power         = 0x16,
    FFStatus      = 0x17 // Idem.
}

#[derive(Clone, Copy, PartialEq)]
enum EventCode  {
    ButtonLeft    = 0x110,
    ButtonRight   = 0x111,
    ButtonMiddle  = 0x112,
    ButtonSide    = 0x113,
    ButtonExtra   = 0x114,
    ButtonForward = 0x115,
    ButtonBack    = 0x116,
    ButtonTask    = 0x117,
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

    println!("Input device ID: bus 0x{:x} vendor 0x{:x} product 0x{:x} version 0x{:x}",
        ii.bustype, ii.vendor, ii.product, ii.version);

    let mut vers : libc::c_int = 0;

    let _ = ioctl(fd, IOCTL::GetVersion, &mut vers as *mut _ as *mut u8);

    println!("Version = {}.{}.{}", (vers >> 16) % 0x100, (vers >> 8) % 0x100, vers % 0x100);

    let mut bits : libc::c_long = 0;

    let _ = ioctl(fd, IOCTL::GetBits, &mut bits as *mut _ as *mut u8);

    let mut event_types = Vec::new();

    event_types.push(EventType::Synchro); // Il doit nécessairement
                                          // être présent.
    if (bits >> 0x01) % 0b10 == 1   {
        event_types.push(EventType::Key);
    } 
    if (bits >> 0x02) % 0b10 == 1   {
        event_types.push(EventType::Relative);
    } 
    if (bits >> 0x03) % 0b10 == 1   {
        event_types.push(EventType::Absolute);
    } 
    if (bits >> 0x04) % 0b10 == 1   {
        event_types.push(EventType::Miscellaneous);
    } 
    if (bits >> 0x05) % 0b10 == 1   {
        event_types.push(EventType::Switch);
    } 
    if (bits >> 0x11) % 0b10 == 1   {
        event_types.push(EventType::LED);
    } 
    if (bits >> 0x12) % 0b10 == 1   {
        event_types.push(EventType::Sound);
    } 
    if (bits >> 0x14) % 0b10 == 1   {
        event_types.push(EventType::Repeat);
    } 
    if (bits >> 0x15) % 0b10 == 1   {
        event_types.push(EventType::FF);
    } 
    if (bits >> 0x16) % 0b10 == 1   {
        event_types.push(EventType::Power);
    } 
    if (bits >> 0x17) % 0b10 == 1   {
        event_types.push(EventType::FFStatus);
    } 

    println!("libevdev_has_event_type(dev, EV_REL) = {}",
        event_types.contains(&EventType::Relative));
    println!("libevdev_has_event_type(dev, EV_KEY) = {}",
        event_types.contains(&EventType::Key));

    let mut key_bits : [c_ulong; 12] = [0; 12];

    let _ = ioctl(fd, IOCTL::GetKeyBits, &mut key_bits as *mut _ as *mut u8);

    let mut event_codes = Vec::new();

    if (key_bits[4] >> (0x110 - 64 * 4)) % 0b10 == 1    {
        event_codes.push(EventCode::ButtonLeft);
    }
    if (key_bits[4] >> (0x111 - 64 * 4)) % 0b10 == 1    {
        event_codes.push(EventCode::ButtonRight);
    }
    if (key_bits[4] >> (0x112 - 64 * 4)) % 0b10 == 1    {
        event_codes.push(EventCode::ButtonMiddle);
    }
    if (key_bits[4] >> (0x113 - 64 * 4)) % 0b10 == 1    {
        event_codes.push(EventCode::ButtonSide);
    }
    if (key_bits[4] >> (0x114 - 64 * 4)) % 0b10 == 1    {
        event_codes.push(EventCode::ButtonExtra);
    }
    if (key_bits[4] >> (0x115 - 64 * 4)) % 0b10 == 1    {
        event_codes.push(EventCode::ButtonForward);
    }
    if (key_bits[4] >> (0x116 - 64 * 4)) % 0b10 == 1    {
        event_codes.push(EventCode::ButtonBack);
    }
    if (key_bits[4] >> (0x117 - 64 * 4)) % 0b10 == 1    {
        event_codes.push(EventCode::ButtonTask);
    }

    println!("libevdev_has_event_code(dev, EV_KEY, BTN_LEFT) = {}",
        event_codes.contains(&EventCode::ButtonLeft));
}




























