//! Production entry. All real work happens in `glassmarlin_lib::run()`.

fn main() {
    if let Err(e) = glassmarlin_lib::run() {
        eprintln!("glassmarlin: fatal: {e:?}");
        std::process::exit(1);
    }
}
