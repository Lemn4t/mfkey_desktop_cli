use std::fmt::Display;
use std::io;
use std::io::Write;

pub trait PrintExt {
    fn print(&self);
    fn println(&self);
    fn eprintln(&self);
}

impl<D: Display> PrintExt for D {
    fn print(&self) {
        print!("{self}");
        io::stdout().flush().expect("Could not flush stdout");
    }

    fn println(&self) {
        println!("{self}");
    }

    fn eprintln(&self) {
        eprintln!("{self}");
    }
}
