extern crate fuzz;
extern crate env_logger;

use fuzz::App;

fn main() {
    env_logger::init().unwrap();
    let mut app = App::new();
    app.start();
}
