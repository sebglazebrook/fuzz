use ncurses::*;


/* Individual color handles. */
static COLOR_SELECTED_BACKGROUND: i16 = 237;

/* Color pairs; foreground && background. */
static COLOR_PAIR_DEFAULT: i16 = 1;
static COLOR_PAIR_SELECTED: i16 = 2;

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
        Curses { width: width, height: height }
    }

    pub fn move_cursor(&self, row: i32, column: i32) {
        mv(row, column);
    }

    pub fn bold(&self) {
        attron(A_BOLD());
    }

    pub fn normal(&self) {
        attroff(A_BOLD());
        self.normal_background();
    }

    pub fn normal_background(&self) {
        attron(COLOR_PAIR(COLOR_PAIR_DEFAULT));
    }

    pub fn selected_background(&self) {
        attron(COLOR_PAIR(COLOR_PAIR_SELECTED));
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

    pub fn get_char_and_key(&self) -> (i32, String) {
        nodelay(stdscr, false);
        let char = getch();
        (char, keyname(char))
    }

    pub fn try_get_char_and_key(&self) -> Option<(i32, String)> {
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

    fn init() {
        initscr();
        raw();
        noecho();
        keypad(stdscr, true);

        start_color();
        init_pair(COLOR_PAIR_DEFAULT, COLOR_WHITE, COLOR_BLACK);
        init_pair(COLOR_PAIR_SELECTED, COLOR_WHITE, COLOR_SELECTED_BACKGROUND);
    }
}
