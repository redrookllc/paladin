fn main() {
    if let Err(e) = paladin_gui::run_app() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
