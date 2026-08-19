use std::{ffi::CStr, ptr::null};

use libc::c_int;
use libc::{self};

use errno::errno;

pub fn init() -> Result<(), c_int> {
    let pid = unsafe { libc::getpid() };
    if pid != 1 {
        eprintln!("Must run as PID 1");
        return Err(libc::EPERM);
    }

    if let Err(err) = mount_helper(c"proc", c"/proc", c"proc", 0, null()) {
        eprintln!("mount failed: {err}");
    }

    if let Err(err) = mount_helper(c"sysfs", c"/sys", c"sysfs", 0, null()) {
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
        let mut status: libc::c_int = 0;
        let result = unsafe { libc::wait(&mut status as *mut libc::c_int) };
        match result {
            pid if pid > 0 && libc::WIFEXITED(status) => {
                let exit_code = libc::WEXITSTATUS(status);
                println!("child {pid} exited with {exit_code}");
                if child_pid == pid {
                    child_pid = -1;
                    if let Ok(spawn_pid) = spawn_child(cmd, &[], &[]) {
                        child_pid = spawn_pid;
                    } else {
                        eprintln!("start sh fail");
                    }
                }
            }
            _ => {}
        }
    }
}

fn mount_helper(
    src: &CStr,
    target: &CStr,
    fstype: &CStr,
    flags: u64,
    data: *const std::ffi::c_void,
) -> Result<(), c_int> {
    let ret = unsafe { libc::mount(src.as_ptr(), target.as_ptr(), fstype.as_ptr(), flags, data) };
    if ret < 0 {
        let err = c_int::from(errno());
        match err {
            libc::EPERM => {
                println!("EPERM, permission failed");
                return Err(err);
            }
            libc::EBUSY => {
                if let Ok(target_str) = target.to_str() {
                    println!("EBUSY, {target_str} had already mounted");
                }
                return Ok(());
            }
            libc::ENOENT => {
                let mkdir_ret = unsafe { libc::mkdir(src.as_ptr(), 0o555) };
                if mkdir_ret < 0 {
                    let mkdir_err = c_int::from(errno());
                    return Err(mkdir_err);
                } else {
                    let retry_ret = unsafe {
                        libc::mount(src.as_ptr(), target.as_ptr(), fstype.as_ptr(), flags, data)
                    };
                    let retry_err = c_int::from(errno());
                    if retry_ret < 0 {
                        return Err(retry_err);
                    } else {
                        return Ok(());
                    }
                }
            }
            _ => {
                return Err(err);
            }
        }
    }
    Ok(())
}

fn spawn_child(cmd: &CStr, args: &[&[u8]], env: &[&[u8]]) -> Result<i32, c_int> {
    match unsafe { libc::fork() } {
        0 => {
            match unsafe { libc::setsid() } {
                sid if sid >= 0 => {
                    let fd = unsafe { libc::open(c"/dev/ttyAMA0".as_ptr(), libc::O_RDWR) };
                    match fd {
                        fd if fd >= 0 => {
                            match unsafe { libc::dup2(fd, libc::STDIN_FILENO) } {
                                ret if ret >= 0 => {}
                                _ => {
                                    std::process::exit(c_int::from(errno()));
                                }
                            }
                            match unsafe { libc::dup2(fd, libc::STDOUT_FILENO) } {
                                ret if ret >= 0 => {}
                                _ => {
                                    std::process::exit(c_int::from(errno()));
                                }
                            }
                            match unsafe { libc::dup2(fd, libc::STDERR_FILENO) } {
                                ret if ret >= 0 => {}
                                _ => {
                                    std::process::exit(c_int::from(errno()));
                                }
                            }
                        }
                        _ => {
                            std::process::exit(c_int::from(errno()));
                        }
                    }
                    unsafe { libc::close(fd) };
                }
                _ => {
                    std::process::exit(c_int::from(errno()));
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

            unsafe { libc::execve(cmd.as_ptr(), argv.as_ptr(), envp.as_ptr()) };
            std::process::exit(c_int::from(errno()));
        }
        child_pid if child_pid > 0 => Ok(child_pid),
        _ => Err(c_int::from(errno())),
    }
}
