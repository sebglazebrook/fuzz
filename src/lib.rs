extern crate ncurses;
extern crate directory_filter;
extern crate crossbeam;
extern crate clipboard;
#[macro_use] extern crate log;

mod fuzz;

pub use fuzz::App;
