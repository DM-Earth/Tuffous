pub mod base;
mod cli;
pub(crate) mod gui;

pub fn get_version() -> String {
    String::from("0.1")
}

fn main() {
    cli::execute();
}
