use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};
use std::thread;
use std::io;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use directory_filter::{ContinuousFilter, FilteredDirectory, ScannerBuilder, Directory};
use crossbeam;
use std::sync::mpsc::TryRecvError::*;
use clipboard::ClipboardContext;

use fuzz::Curses;


pub struct App<'a> {
    done: AtomicBool,
    filter_string: String,
    trans_filter_change: Arc<Mutex<Sender<String>>>,
    rec_filter_change: Arc<Mutex<Receiver<String>>>,
    trans_new_directory_item: Arc<Mutex<Sender<Directory>>>,
    rec_new_directory_item: Arc<Mutex<Receiver<Directory>>>,
    trans_filter_match: Arc<Mutex<Sender<FilteredDirectory<'a>>>>,
    curses: Curses,
    selected_result: i8,
    displayed_results: Vec<String>
}

impl<'a> App<'a> {

    pub fn new() -> Self {
        let(trans_filter_change, rec_filter_change) = channel();
        let(trans_new_directory_item, rec_new_directory_item) = channel();
        let(trans_filter_match, rec_filter_match) = channel();
        App {
            done: AtomicBool::new(false),
            filter_string: String::new(),
            trans_filter_change: Arc::new(Mutex::new(trans_filter_change)),
            rec_filter_change: Arc::new(Mutex::new(rec_filter_change)),
            trans_new_directory_item: Arc::new(Mutex::new(trans_new_directory_item)),
            rec_new_directory_item: Arc::new(Mutex::new(rec_new_directory_item)),
            trans_filter_match: Arc::new(Mutex::new(trans_filter_match)),
            curses: Curses::new(),
            selected_result: -1,
            displayed_results: vec![],
        }
    }

    pub fn start(&mut self) {
        let rec_filter_change = self.rec_filter_change.clone();
        let rec_new_directory_item = self.rec_new_directory_item.clone();
        let trans_new_directory_item = self.trans_new_directory_item.clone();
        let mut directory = Directory::new(PathBuf::new());
        let(trans_new_directory_item, rec_new_directory_item) = channel();
        let(trans_filter_match, rec_filter_match) = channel();
        crossbeam::scope(|scope| {

            let mut scanner_builder = ScannerBuilder::new();
            scanner_builder = scanner_builder.start_from_path("./");
            scanner_builder = scanner_builder.max_threads(1);
            scanner_builder = scanner_builder.update_subscriber(Arc::new(Mutex::new(trans_new_directory_item)));
            let mut scanner = scanner_builder.build();
            directory = scanner.scan();

            let mut filter = ContinuousFilter::new(&directory,
                                                   rec_filter_change,
                                                   Arc::new(Mutex::new(rec_new_directory_item)),
                                                   Arc::new(Mutex::new(trans_filter_match.clone()))
                                                  );

            scope.spawn(move|| {
                    filter.start();
            });

            self.set_cursor_to_filter_input();

            while !self.done.load(Ordering::Relaxed) {
                match self.curses.get_char_and_key() {
                    Some((character, key)) => {
                        self.handle_user_input(character, key);
                    },
                    None => {
                        match rec_filter_match.try_recv() {
                            Ok(filtered_directory) =>  {
                                self.update_results(filtered_directory);
                            },
                            Err(error) => {
                                match error {
                                    Empty => {}
                                    Disconnected => {}
                                }
                            }
                        }
                    }
                }
            }

            self.curses.close();
        });
    }

    //---------- private ----------//

    fn handle_user_input(&mut self, character: i32, key: String) {
        if self.is_special_key(&key) {
            self.handle_special_character(character, &key);
        } else {
            self.amend_filter_string(&key);
        }
    }

    fn update_results(&mut self, results: FilteredDirectory) {
        self.clear_results();
        for (index, result) in results.matches.iter().enumerate() {
            if index == self.max_result_rows() {
                break;
            }
            self.update_result(result, index);
        }
        self.set_cursor_to_filter_input();
    }

    fn update_result(&mut self, result: &String, row_number: usize) {
        self.displayed_results.push(result.clone());
        self.curses.move_cursor(row_number as i32, 0);
        self.curses.normal();
        self.curses.println(result);
    }

    fn clear_results(&mut self) {
        self.displayed_results.clear();
        for row in (0..self.max_result_rows()) {
            self.curses.clear_row(row as i32);
        }
    }

    fn is_special_key(&self, key: &String) -> bool {
        key.chars().count() != 1
    }

    fn handle_special_character(&mut self, character: i32, key: &String) {
        match key.as_ref() {
            "^C" => { self.done.store(true, Ordering::Relaxed) },
            "^Y" => {
                self.copy_selected_to_clipboard();
                self.done.store(true, Ordering::Relaxed)
            },
            "^J" => { self.move_selected_down(); },
            "^K" => { self.move_selected_up(); }
            _ => {
                match character {
                    263 => { //KEY_BACKSPACE
                        self.filter_string.pop(); 
                        self.trans_filter_change.lock().unwrap().send(self.filter_string.clone());
                        self.update_ui();
                    },
                    27 => { // ESCAPE
                        self.done.store(true, Ordering::Relaxed);
                    },
                    10 => { // ENTER
                        self.copy_selected_to_clipboard();
                        self.done.store(true, Ordering::Relaxed);
                    },
                    258 => { // KEY_DOWN
                        self.move_selected_down();
                    },
                    259 => { // KEY_UP
                        self.move_selected_up();
                    },
                    _ => { }
                }
            }
        }
    }

    fn amend_filter_string(&mut self, key: &String) {
        self.filter_string = self.filter_string.clone() + key;
        self.trans_filter_change.lock().unwrap().send(self.filter_string.clone());
        self.update_ui();
    }

    fn update_ui(&self) {
        self.set_cursor_to_filter_input_beginning();
        let filter_string = self.filter_string.clone();
        self.curses.bold();
        self.curses.println(&filter_string);
    }

    fn set_cursor_to_filter_input(&self) {
        let column = self.filter_string.chars().count();
        self.curses.move_cursor(self.curses.height -1, column as i32);
    }

    fn set_cursor_to_filter_input_beginning(&self) {
        self.curses.move_cursor(self.curses.height -1, 0);
    }

    fn max_result_rows(&self) -> usize  {
        (self.curses.height - 1) as usize
    }

    fn move_selected_down(&mut self) {
        if self.selected_result < self.max_result_rows() as i8 {
            self.unselect_current();
            self.selected_result += 1;
            self.select_row(self.selected_result);
        }
    }

    fn move_selected_up(&mut self) {
        if self.selected_result > -1 {
            self.unselect_current();
            self.selected_result -= 1;
            self.select_row(self.selected_result);
        }
    }

    fn unselect_current(&self) {
        if self.selected_result >= 0 {
            match self.displayed_results.get(self.selected_result as usize) {
                Some(result) => {
                    self.curses.move_cursor(self.selected_result as i32, 0);
                    self.curses.normal_background();
                    self.curses.println(&result);
                },
                None => {}
            }
        }
    }

    fn select_row(&self, row: i8) {
        match self.displayed_results.get(self.selected_result as usize) {
            Some(result) => {
                self.curses.move_cursor(self.selected_result as i32, 0);
                self.curses.selected_background();
                self.curses.println(&result);
            },
            None => {}
        }
    }


    fn copy_selected_to_clipboard(&self) {
        match self.displayed_results.get(self.selected_result as usize) {
            Some(result) => {
                let mut ctx = ClipboardContext::new().unwrap();
                let _ = ctx.set_contents(result.clone());
            },
            None => {}
        }
    }
}
