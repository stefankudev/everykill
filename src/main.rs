fn main() {
    if let Err(e) = everykill::run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
