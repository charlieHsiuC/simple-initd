use std::ffi::CStr;

use rustix::{
    fs::{self, OFlags, open},
    io::Errno,
    mount::{self, MountFlags, mount},
    process::{self, WaitOptions, setsid},
    runtime::{Fork, execve, kernel_fork},
    stdio::{self},
};

fn main() -> Result<(), i32> {
    if !process::getpid().is_init() {
        eprintln!("Must run as PID 1");
        return Err(Errno::PERM.raw_os_error());
    }

    if let Err(err) = mount_helper("proc", "/proc", "proc") {
        eprintln!("mount failed: {err}");
    }

    if let Err(err) = mount_helper("sysfs", "/sys", "sysfs") {
        eprintln!("mount failed: {err}");
    }

    let mut child_pid = -1;
    let cmd = c"/bin/sh";
    if let Ok(spawn_pid) = spawn_child(cmd, &[], &[]) {
        child_pid = spawn_pid;
    } else {
        eprintln!("start sh fail");
    }

    loop {
        // wait
        match process::wait(WaitOptions::empty()) {
            Ok(Some((pid, status))) => {
                if status.exited() {
                    let exit_code = status.exit_status().unwrap();
                    println!("child {pid} exited with {exit_code}");
                    if child_pid == pid.as_raw_pid() {
                        child_pid = -1;
                        if let Ok(spawn_pid) = spawn_child(cmd, &[], &[]) {
                            child_pid = spawn_pid;
                        } else {
                            eprintln!("start sh fail");
                        }
                    }
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

fn spawn_child(cmd: &CStr, args: &[&[u8]], env: &[&[u8]]) -> Result<i32, Errno> {
    unsafe {
        match kernel_fork() {
            Ok(Fork::Child(_pid)) => {
                match setsid() {
                    Ok(_) => {
                        let fd = open("/dev/ttyAMA0", OFlags::RDWR, fs::Mode::empty());
                        match fd {
                            Ok(fd) => {
                                match stdio::dup2_stdin(&fd) {
                                    Ok(()) => {}
                                    Err(err) => {
                                        std::process::exit(err.raw_os_error());
                                    }
                                }
                                match stdio::dup2_stdout(&fd) {
                                    Ok(()) => {}
                                    Err(err) => {
                                        std::process::exit(err.raw_os_error());
                                    }
                                }
                                match stdio::dup2_stderr(&fd) {
                                    Ok(()) => {}
                                    Err(err) => {
                                        std::process::exit(err.raw_os_error());
                                    }
                                }
                            }
                            Err(err) => {
                                std::process::exit(err.raw_os_error());
                            }
                        }
                    }
                    Err(err) => {
                        std::process::exit(err.raw_os_error());
                    }
                }

                let mut argv = vec![cmd.as_ptr()];
                for v in args {
                    argv.push(v.as_ptr())
                }
                argv.push(std::ptr::null());

                let mut envp = vec![];
                for v in env {
                    envp.push(v.as_ptr());
                }
                envp.push(std::ptr::null());

                let errno = execve(cmd, argv.as_ptr(), envp.as_ptr());
                std::process::exit(errno.raw_os_error());
            }
            Ok(Fork::ParentOf(child_pid)) => Ok(child_pid.as_raw_pid()),
            Err(err) => Err(err),
        }
    }
}
