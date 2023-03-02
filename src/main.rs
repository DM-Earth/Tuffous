pub mod base;
mod cli;

pub fn get_version() -> String {
    String::from("0.1")
}

fn main() {
    cli::execute();
}
