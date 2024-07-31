// Include the clap crate for command line argument parsing

fn main() {
    if let Err(e) = rytest::get_args().and_then(rytest::run) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
