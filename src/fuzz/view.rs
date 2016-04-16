use directory_filter::{FilteredDirectory, File};

use fuzz::{Curses};

pub struct View {
    selected_result: i8,
    displayed_results: Vec<String>,
    curses: Curses,
    filter_string: String,
}

impl View {

    pub fn new(curses: Curses) -> Self {
        View {
            selected_result: -1,
            displayed_results: vec![],
            curses: curses,
            filter_string: String::new(),
        }
    }

    pub fn update_results(&mut self, results: FilteredDirectory) {
        info!("Found filter match: {}", results.len());
        self.clear_results();
        for (index, result) in results.clone().into_iter().enumerate() {
            if index == self.max_result_rows() {
                break;
            }
            let row_to_update = self.max_result_rows() - index - 1;
            self.update_result(&result, row_to_update);
        }
        self.select_row();
        self.update_stats(results.total_len(), results.len());
        self.set_cursor_to_filter_input();
    }

    pub fn select_first_result(&mut self) {
        self.selected_result = self.max_result_rows() as i8;
    }

    pub fn move_selected_down(&mut self) {
        if self.selected_result < self.max_result_rows() as i8 {
            self.unselect_current();
            self.selected_result += 1;
            self.select_row();
        }
    }

    pub fn move_selected_up(&mut self) {
        if self.selected_result > -1 {
            self.unselect_current();
            self.selected_result -= 1;
            self.select_row();
        }
    }

    pub fn get_selected_result(&self) -> Option<String> {
        let selected_result = self.max_result_rows() - self.selected_result as usize;
        match self.displayed_results.get(selected_result) {
            Some(result) => { Some(result.clone()) }
            None => { None }
        }
    }

    fn unselect_current(&self) {
        if self.selected_result >= 0 {
            let selected_result = self.max_result_rows() - self.selected_result as usize;
            match self.displayed_results.get(selected_result) {
                Some(result) => {
                    let row = self.selected_result as i32 - 1;
                    self.curses.move_cursor(row, 0);
                    self.curses.normal_background();
                    self.curses.println(&result);
                },
                None => {}
            }
        }
    }

    fn clear_results(&mut self) {
        self.displayed_results.clear();
        for row in 0..self.max_result_rows() {
            self.curses.clear_row(row as i32);
        }
    }

    fn update_result(&mut self, result: &File, row_number: usize) {
        self.displayed_results.push(result.as_string());
        self.curses.move_cursor(row_number as i32, 0);
        self.curses.normal();
        self.curses.println(&result.as_string());
    }

    fn max_result_rows(&self) -> usize  {
        (self.curses.height - 2) as usize
    }

    fn select_row(&mut self) {
        let selected_result = self.max_result_rows() - self.selected_result as usize;
        match self.displayed_results.get(selected_result) {
            Some(result) => {
                let row = self.selected_result as i32 - 1;
                self.curses.move_cursor(row, 0);
                self.curses.selected_background();
                self.curses.println(&result);
            },
            None => {
                self.selected_result = self.max_result_rows() as i8;
                match self.displayed_results.get(self.selected_result as usize) {
                    Some(result) => {
                        let row = self.selected_result as i32 - 1;
                        self.curses.move_cursor(row, 0);
                        self.curses.selected_background();
                        self.curses.println(&result);
                },
                None => {}
                    }
            }
        }
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
}
