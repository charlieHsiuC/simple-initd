use std::ffi::CStr;

use rustix::{
    fs,
    io::Errno,
    mount::{self, MountFlags, mount},
    process::{self, WaitOptions},
    runtime::{Fork, execve, kernel_fork},
};

fn main() -> Result<(), i32> {
    if !process::getpid().is_init() {
        eprintln!("Must run as PID 1");
        return Err(Errno::PERM.raw_os_error());
    }

    if let Err(err) = mount_helper("proc", "/proc", "proc") {
        eprintln!("mount failed: {err}");
    }

    if let Err(err) = mount_helper("sys", "/sys", "sys") {
        eprintln!("mount failed: {err}");
    }

    spawn_shell();

    loop {
        // wait
        match process::wait(WaitOptions::empty()) {
            Ok(Some((pid, status))) => {
                if status.exited() {
                    let exit_code = status.exit_status().unwrap();
                    println!("child {pid} exited with {exit_code}");
                }
            }
            Ok(None) => {}
            Err(err) => match err {
                Errno::CHILD => {}
                Errno::AGAIN => {}
                Errno::INTR => {}
                Errno::INVAL => {}
                Errno::SRCH => {}
                _ => {
                    eprintln!("unexpected error with {err}");
                }
            },
        }
    }
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

fn spawn_shell() {
    unsafe {
        match kernel_fork() {
            Ok(Fork::Child(_pid)) => {
                let path = c"/bin/sh";
                let argv = [path.as_ptr(), std::ptr::null()];
                let envp = [std::ptr::null()];
                let errno = execve(path, argv.as_ptr(), envp.as_ptr());
                eprintln!("execve failed: {errno}");
                std::process::exit(1);
            }
            Ok(Fork::ParentOf(child_pid)) => {
                println!("spawn child {child_pid}");
            }
            Err(err) => eprintln!("fork failed: {err}"),
        }
    }
}
