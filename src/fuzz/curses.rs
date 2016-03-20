use ncurses::*;

pub struct Curses {
    pub width: i32,
    pub height: i32,
}

impl Curses {

    pub fn new() -> Self {
        Curses::init();
        let mut width = 0;
        let mut height = 0;
        getmaxyx(stdscr, &mut height, &mut width);  
        Curses { width: width, height: height  }
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

    pub fn get_char_key(&self) -> (i32, String) {
        let char = getch();
        (char, keyname(char))
    }


    pub fn close(&self) {
        endwin();
    }
    //--------- private -----------//

    fn init() {
        initscr();
        raw();
        noecho();
        keypad(stdscr, true);
        attron(A_BOLD());
    }
}
