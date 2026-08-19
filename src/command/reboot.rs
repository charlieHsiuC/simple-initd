use libc::c_int;

pub fn reboot() -> Result<(), c_int> {
    Ok(())
}
