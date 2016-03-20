use ncurses::*;
use std::sync::Mutex;

pub struct Curses {
    pub width: i32,
    pub height: i32,
    //screen: Mutex<*mut i8>,
    //window: i8,
}

impl Curses {

    pub fn new() -> Self {
        let window = Curses::init();
        //set_term(window);
        //box_(window);
        //use_window(window);
        let mut width = 0;
        let mut height = 0;
        getmaxyx(stdscr, &mut height, &mut width);  
        Curses { width: width, height: height }
    }

    pub fn move_cursor(&self, row: i32, column: i32) {
        mv(row, column);
    }

    pub fn print(&self, message: &str) {
        printw(message);
    }

    pub fn println(&self, message: &str) {
        clrtoeol();
        self.print(message);
    }

    pub fn clear_row(&self, row: i32) {
        self.move_cursor(row, 0);
        clrtoeol();
    }

    pub fn get_char_key(&self) -> (i32, String) {
        let char = getch();
        (char, keyname(char))
    }

    pub fn get_char_and_key(&self) -> Option<(i32, String)> {
        nodelay(stdscr, true);
        let char = getch();
        if char == -1 {
            None
        } else {
            Some((char, keyname(char)))
        }
    }

    pub fn close(&self) {
        endwin();
    }
    //--------- private -----------//

    fn init() -> *mut i8 {
        let window = initscr();
        raw();
        noecho();
        keypad(stdscr, true);
        attron(A_BOLD());
        window
    }
}
