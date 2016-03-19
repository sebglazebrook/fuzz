use std::char;
use ncurses::*;

pub struct App {
    done: bool,
    filter_string: String,
}

impl App {

    pub fn new() -> Self {
        App { done: false, filter_string: String::new() }
    }

    pub fn start(&mut self) {
        self.init_curses();
        self.start_scanning();
        self.handle_user_input();
    }

    //---------- private ----------//

    fn init_curses(&self) {
        initscr();
        raw();
        noecho();
        keypad(stdscr, true);
        self.set_cursor_to_filter_input()
    }

    fn start_scanning(&self) {
        // TODO
    }

    fn handle_user_input(&mut self) {
        while !self.done {
            let character = getch();
            if self.is_special_character(character) {
                self.handle_special_character(character);
            } else {
                self.amend_filter_string(character);
            }
        }
        endwin();
    }

    //---------- private -------------//

    fn is_special_character(&self, character: i32) -> bool { 
        keyname(character).len() != 1 
    }

    fn handle_special_character(&mut self, character: i32) {
        match keyname(character).as_ref() {
            "^C" => { self.done = true }
            _ => {
                match character {
                    KEY_BACKSPACE => { 
                        self.filter_string.pop(); 
                        self.update_ui();
                    }
                    _ => { }
                }
            }
        }
    }

    fn amend_filter_string(&mut self, character: i32) {
        self.filter_string = self.filter_string.clone() + &keyname(character);
        // TODO send out change event
        self.update_ui();
    }

    fn update_ui(&self) {
        self.set_cursor_to_filter_input();
        clrtoeol();
        attron(A_BOLD());
        printw(&self.filter_string);
        refresh();
    }

    fn set_cursor_to_filter_input(&self) {
        mv(self.last_line_number(), 0); 
    }

    fn last_line_number(&self) -> i32 {
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(stdscr, &mut max_y, &mut max_x);  
        max_y -1 
    }
}
