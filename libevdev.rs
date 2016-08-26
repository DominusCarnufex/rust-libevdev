#![feature(libc)]

extern crate libc;
use libc::{c_char, c_int, c_ulong};

struct CString(String);

impl CString    {
    fn new(s : &str) -> Self    {
        let mut string = s.to_string();
        string.push('\0');
        CString(string)
    }

    fn as_ptr(&self) -> *const c_char   {
        let &CString(ref st) = self;
        st.as_ptr() as *const c_char
    }

    fn as_ref(&self) -> &str    {
        let &CString(ref st) = self;
        st
    }
}

#[repr(C)]
struct InputId  {
        bustype : u16,
        vendor  : u16,
        product : u16,
        version : u16
}

impl InputId    {
    fn new() -> Self    {
        InputId {
            bustype : 0,
            vendor  : 0,
            product : 0,
            version : 0
        }
    }

    fn as_mut_ptr(&mut self) -> *mut u8 {
        self as *mut Self as *mut u8
    }
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

impl EventType  {
    fn new(int : u8) -> Self    {
        match int   {
            0x00 => EventType::Synchro,
            0x01 => EventType::Key,
            0x02 => EventType::Relative,
            0x03 => EventType::Absolute,
            0x04 => EventType::Miscellaneous,
            0x05 => EventType::Switch,
            0x11 => EventType::LED,
            0x12 => EventType::Sound,
            0x14 => EventType::Repeat,
            0x15 => EventType::FF,
            0x16 => EventType::Power,
            0x17 => EventType::FFStatus,
            _    => panic!("EventType inconnu : 0x{:x}", int)
        }
    }
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

impl EventCode  {
    fn new(event_type : EventType, int : usize) -> Self {
        match event_type    {
            EventType::Key => match int {
                0x110 => EventCode::ButtonLeft,
                0x111 => EventCode::ButtonRight,
                0x112 => EventCode::ButtonMiddle,
                0x113 => EventCode::ButtonSide,
                0x114 => EventCode::ButtonExtra,
                0x115 => EventCode::ButtonForward,
                0x116 => EventCode::ButtonBack,
                0x117 => EventCode::ButtonTask,
                _     => unimplemented!()
            },
            _ => unimplemented!()
        }
    }
}

fn errno() -> c_int {
    unsafe { *libc::__errno_location() }
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
    let name = CString::new("/dev/input/event6");
    let fd = unsafe {
        libc::open(name.as_ptr(), libc::O_RDONLY | libc::O_NONBLOCK)
    };

    if fd < 0   {
        panic!("Impossible d’ouvrir le fichier {}.", name.as_ref());
    }

    let mut ii = InputId::new();

    let _ = ioctl(fd, IOCTL::GetId, ii.as_mut_ptr());

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
    for i in 0x01..0x20 {
        if (bits >> i) % 0b10 == 1  {
            event_types.push(EventType::new(i));
        }
    }

    println!("libevdev_has_event_type(dev, EV_REL) = {}",
        event_types.contains(&EventType::Relative));
    println!("libevdev_has_event_type(dev, EV_KEY) = {}",
        event_types.contains(&EventType::Key));

    let mut key_bits : [c_ulong; 12] = [0; 12];

    let _ = ioctl(fd, IOCTL::GetKeyBits, &mut key_bits as *mut _ as *mut u8);

    let mut event_codes = Vec::new();

    for i in 0x00..0x300    {
        let a = i / 64;
        if (key_bits[a] >> (i - 64 * a)) % 0b10 == 1    {
            event_codes.push(EventCode::new(EventType::Key, i));
        }
    }

    println!("libevdev_has_event_code(dev, EV_KEY, BTN_LEFT) = {}",
        event_codes.contains(&EventCode::ButtonLeft));
}
