use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::TryRecvError::*;
use crossbeam;
use clipboard::ClipboardContext;
use directory_filter::{ContinuousFilter, FilteredDirectory, DirectoryScanner, ScannerBuilder, Directory, File, FILTER_EVENT_BROKER};

use fuzz::Curses;

pub struct App {
    done: AtomicBool,
    filter_string: String,
    curses: Curses,
    selected_result: i8,
    displayed_results: Vec<String>
}

impl App {

    pub fn new() -> Self {
        App {
            done: AtomicBool::new(false),
            filter_string: String::new(),
            curses: Curses::new(),
            selected_result: -1,
            displayed_results: vec![],
        }
    }

    pub fn start(&mut self) {
        info!("App started");
        let mut directory = Directory::new(PathBuf::new());
        let(trans_filter_match, rec_filter_match) = channel();
        crossbeam::scope(|scope| {

            let mut scanner = self.build_scanner();
            info!("Starting to scan for files");
            directory = scanner.scan();
            let new_directory_item_event_broker = scanner.event_broker();

            let filter = Arc::new(ContinuousFilter::new(directory,
                                                   Arc::new(Mutex::new(trans_filter_match.clone())),
                                                   new_directory_item_event_broker.clone()
                                                  ));

            let finished_lock = filter.finished_lock.clone();
            let finished_condvar = filter.finished_condvar.clone();
            let local_filter = filter.clone();
            scope.spawn(move|| {
                info!("Starting to filter scanned files");
                local_filter.start();
            });

            self.set_cursor_to_filter_input();

            while !self.done.load(Ordering::Relaxed) {
                if !(filter.is_processing() || !scanner.is_complete()) {
                    match rec_filter_match.try_recv() {
                        Ok(filtered_directory) =>  {
                            info!("Found filter match: {}", filtered_directory.len());
                            self.update_results(filtered_directory); },
                            Err(error) => {
                                match error {
                                    Empty => {}
                                    Disconnected => {}
                                }
                            }
                    }
                    let (character, key) = self.curses.get_char_and_key();
                    self.handle_user_input(character, key);
                } else {
                    match self.curses.try_get_char_and_key() {
                        Some((character, key)) => {
                            info!("Found character {}, key {}", character, key);
                            self.handle_user_input(character, key);
                        },
                        None => {
                            match rec_filter_match.try_recv() {
                                Ok(filtered_directory) =>  {
                                    info!("Found filter match: {}", filtered_directory.len());
                                    self.update_results(filtered_directory); },
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
            }

            self.curses.close();
            let mut finished = finished_lock.lock().unwrap();
            *finished = true;
            finished_condvar.notify_all();

            FILTER_EVENT_BROKER.close();
            new_directory_item_event_broker.close();
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
        for (index, result) in results.clone().into_iter().enumerate() {
            if index == self.max_result_rows() {
                break;
            }
            self.update_result(&result, index);
        }
        self.update_stats(results.total_len(), results.len());
        self.set_cursor_to_filter_input();
    }

    fn update_result(&mut self, result: &File, row_number: usize) {
        self.displayed_results.push(result.as_string());
        self.curses.move_cursor(row_number as i32, 0);
        self.curses.normal();
        self.curses.println(&result.as_string());
    }

    fn clear_results(&mut self) {
        self.displayed_results.clear();
        for row in 0..self.max_result_rows() {
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
                    263 | 127 => { //KEY_BACKSPACE
                        match self.filter_string.pop() {
                            Some(_) => {
                                FILTER_EVENT_BROKER.send(self.filter_string.clone());
                                self.update_ui();
                            },
                            None => {}
                        }
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
        FILTER_EVENT_BROKER.send(self.filter_string.clone());
        self.update_ui();
    }

    fn update_ui(&self) {
        self.set_cursor_to_filter_input_beginning();
        let filter_string = self.filter_string.clone();
        self.curses.bold();
        self.curses.println(&filter_string);
    }

    fn update_stats(&self, total: usize, matching: usize) {
        let row = self.curses.height - 2;
        self.curses.move_cursor(row, 0);
        self.curses.clear_row(row as i32);
        let mut stats = String::new();
        stats = stats + &matching.to_string() + "/" + &total.to_string();
        self.curses.println(&stats);
    }

    fn set_cursor_to_filter_input(&self) {
        let column = self.filter_string.chars().count();
        self.curses.move_cursor(self.curses.height -1, column as i32);
    }

    fn set_cursor_to_filter_input_beginning(&self) {
        self.curses.move_cursor(self.curses.height -1, 0);
    }

    fn max_result_rows(&self) -> usize  {
        (self.curses.height - 2) as usize
    }

    fn move_selected_down(&mut self) {
        if self.selected_result < self.max_result_rows() as i8 {
            self.unselect_current();
            self.selected_result += 1;
            self.select_row();
        }
    }

    fn move_selected_up(&mut self) {
        if self.selected_result > -1 {
            self.unselect_current();
            self.selected_result -= 1;
            self.select_row();
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

    fn select_row(&self) {
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

    fn build_scanner(&self) ->  DirectoryScanner {
        let mut scanner_builder = ScannerBuilder::new();
        scanner_builder = scanner_builder.start_from_path("./");
        scanner_builder.build()
    }
}
