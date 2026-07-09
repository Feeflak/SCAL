use std::io::Write;

pub use scal_ipc_macros::main;

pub fn run_main(user_main: impl FnOnce() -> scal_core::Project) {
    let project = user_main();
    let encoded = bincode::serialize(&project).expect("failed to serialize project");

    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    let len = encoded.len() as u64;
    handle
        .write_all(&len.to_le_bytes())
        .expect("failed to write length");
    handle
        .write_all(&encoded)
        .expect("failed to write project data");
    handle.flush().expect("failed to flush stdout");
}
