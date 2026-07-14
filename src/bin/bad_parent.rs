use rustix::runtime::{Fork, kernel_fork};

fn main() {
    println!("Spawn a child to sleep 10s and parent exit");
    spawn_orphan_child();
}

fn spawn_orphan_child() {
    unsafe {
        match kernel_fork() {
            Ok(Fork::Child(_pid)) => {
                std::thread::sleep(std::time::Duration::from_secs(10));
                std::process::exit(0);
            }
            Ok(Fork::ParentOf(child_pid)) => {
                println!("spawn child {child_pid}");
            }
            Err(err) => eprintln!("fork failed: {err}"),
        }
    }
}
