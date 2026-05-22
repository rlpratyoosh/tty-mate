pub struct Log { }

impl Log {
    pub fn info(message: &str) {
        println!("[INFO] {}", message);
    }

    pub fn error(message: &str) {
        eprintln!("[ERROR] {}", message);
    }
}
