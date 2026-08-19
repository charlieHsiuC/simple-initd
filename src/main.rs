use std::env;
use std::ffi::{CStr, CString};

use libc::c_int;
use libc::{self};

mod command;

use command::{init, reboot};

fn main() -> Result<(), c_int> {
    let args: Vec<String> = env::args().collect();

    if let Ok(arg0) = CString::new(args[0].as_str()) {
        let exec_cstr = unsafe { libc::gnu_basename(arg0.as_ptr()) };
        let exec = unsafe { CStr::from_ptr(exec_cstr) }
            .to_str()
            .expect("Data was not valid UTF-8");
        match exec {
            "init" => init(),
            "reboot" => reboot(),
            _ => Err(1),
        }
    } else {
        Err(1)
    }
}
