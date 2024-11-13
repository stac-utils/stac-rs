use std::sync::Mutex;

pub static MUTEX: Mutex<()> = Mutex::new(());
