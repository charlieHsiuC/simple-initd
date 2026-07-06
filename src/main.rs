use std::ffi::CStr;

use rustix::{
    fs,
    io::Errno,
    mount::{self, MountFlags, mount},
    process,
};

fn main() -> Result<(), i32> {
    if !process::getpid().is_init() {
        eprintln!("Must run as PID 1");
        return Err(Errno::PERM.raw_os_error());
    }

    if let Err(err) = mount_helper("proc", "/proc", "proc") {
        eprintln!("mount failed: {err}");
        return Err(err.raw_os_error());
    }

    if let Err(err) = mount_helper("sys", "/sys", "sys") {
        eprintln!("mount failed: {err}");
        return Err(err.raw_os_error());
    }

    Ok(())
}

fn mount_helper(source: &str, target: &str, fstype: &str) -> rustix::io::Result<()> {
    match mount::mount(source, target, fstype, MountFlags::empty(), None::<&CStr>) {
        Ok(()) => {
            println!("Mounted {target} successfully.");
            Ok(())
        }
        Err(Errno::PERM) => {
            println!("EPERM, permission failed");
            Err(Errno::PERM)
        }
        Err(Errno::BUSY) => {
            println!("EBUSY, {target} had already mounted");
            Ok(())
        }
        Err(Errno::NOENT) => {
            println!("ENOENT, need {target} directory to mount");
            println!("Trying rustix::fs::mkdir, then mount again");
            let mode = fs::Mode::RUSR
                | fs::Mode::RGRP
                | fs::Mode::ROTH
                | fs::Mode::XUSR
                | fs::Mode::XGRP
                | fs::Mode::XOTH;
            if let Err(mkdir_err) = fs::mkdir(target, mode) {
                eprintln!("mkdir failed: {mkdir_err}");
                Err(mkdir_err)
            } else {
                mount(source, target, fstype, MountFlags::empty(), None::<&CStr>)
            }
        }
        Err(err) => {
            println!("mount returned errno {:?} ({err})", err.raw_os_error());
            Err(err)
        }
    }
}
