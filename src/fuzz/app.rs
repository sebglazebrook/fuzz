use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};
use std::thread;
use std::io;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use directory_filter::{ContinuousFilter, FilteredDirectory, ScannerBuilder, Directory};
use crossbeam;

use fuzz::Curses;


pub struct App<'a> {
    done: AtomicBool,
    filter_string: String,
    trans_filter_change: Arc<Mutex<Sender<String>>>,
    rec_filter_change: Arc<Mutex<Receiver<String>>>,
    trans_new_directory_item: Arc<Mutex<Sender<Directory>>>,
    rec_new_directory_item: Arc<Mutex<Receiver<Directory>>>,
    trans_filter_match: Arc<Mutex<Sender<FilteredDirectory<'a>>>>,
    curses: Arc<Mutex<Curses>>,
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
            curses: Arc::new(Mutex::new(Curses::new())),
        }
    }

    pub fn start(&mut self) {
        self.set_cursor_to_filter_input();
        self.handle_user_input();
        self.start_scanning();
    }

    //---------- private ----------//

    fn start_scanning(&self) {
        let rec_filter_change = self.rec_filter_change.clone();
        let rec_new_directory_item = self.rec_new_directory_item.clone();
        let trans_new_directory_item = self.trans_new_directory_item.clone();

        let mut directory = Directory::new(PathBuf::new());
        crossbeam::scope(|s| {
            let(trans_new_directory_item, rec_new_directory_item) = channel();
            let(tx, rx) = channel();
            let mut scanner_builder = ScannerBuilder::new();
            scanner_builder = scanner_builder.start_from_path("./");
            scanner_builder = scanner_builder.max_threads(1);
            scanner_builder = scanner_builder.update_subscriber(Arc::new(Mutex::new(trans_new_directory_item)));
            let mut scanner = scanner_builder.build();
            directory = scanner.scan();
            s.spawn(|| {
                let(trans_filter_match, rec_filter_match) = channel();
                let mut filter = ContinuousFilter::new(&directory,
                                                       //rec_filter_change,
                                                       Arc::new(Mutex::new(rx)),
                                                       Arc::new(Mutex::new(rec_new_directory_item)),
                                                       Arc::new(Mutex::new(trans_filter_match))
                                                      );
                crossbeam::scope(|scope| {
                    scope.spawn(|| {
                        filter.start();
                    });
                });
                while !self.done.load(Ordering::Relaxed) {
                    let found = rec_filter_match.recv().unwrap();
                    println!("found: {}", found.matches.len());
                }

            });
        });
    }

    fn handle_user_input(&mut self) {
        //TODO move this to use one tread
        while !self.done.load(Ordering::Relaxed) {
            let (character, key) = self.curses.lock().unwrap().get_char_key();
            if self.is_special_key(&key) {
                self.handle_special_character(character, &key);
            } else {
                self.amend_filter_string(&key);
            }
        }
        self.curses.lock().unwrap().close();
    }

    //---------- private -------------//

    fn is_special_key(&self, key: &String) -> bool {
        key.chars().count() != 1
    }

    fn handle_special_character(&mut self, character: i32, key: &String) {
        match key.as_ref() {
            "^C" => { self.done.store(true, Ordering::Relaxed ) }
            _ => {
                match character {
                    //KEY_BACKSPACE => {
                    263 => {
                        self.filter_string.pop(); 
                        self.update_ui();
                    }
                    _ => { }
                }
            }
        }
    }

    fn amend_filter_string(&mut self, key: &String) {
        self.filter_string = self.filter_string.clone() + key;
        // TODO send out change event
        self.update_ui();
    }

    fn update_ui(&self) {
        self.set_cursor_to_filter_input();
        let filter_string = self.filter_string.clone();
        self.curses.lock().unwrap().println(&filter_string);
    }

    fn set_cursor_to_filter_input(&self) {
        let curses = self.curses.lock().unwrap();
        curses.move_cursor(curses.height -1, 0);
    }
}

struct StdinHandler {
    curses: Arc<Mutex<Curses>>,
}

impl StdinHandler {

    pub fn new(curses: Arc<Mutex<Curses>>) -> Self {
        StdinHandler { curses: curses }
    }
}
