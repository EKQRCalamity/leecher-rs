use termsize;
use std::io::{Write, stdout};

const BACKSPACE: char = 8u8 as char;

pub struct Progress {
    pub current_value: f64,
    pub max_value: f64,
}

impl Progress {
    pub fn new(current_value: f64, max_value: f64) -> Progress {
        return Progress { current_value: current_value, max_value: max_value };
    }
    
    pub fn get_progress_percent(&self) -> f64 {
        return (((self.current_value as f64) / (self.max_value as f64)) * (100 as f64)) as f64;
    }

    pub fn set_progress(&mut self, progress: &f64) {
        self.current_value = *progress;
    }

    pub fn is_complete(&self) -> bool {
        return self.current_value >= self.max_value;
    }
}

pub struct ProgressBar {
    pub prefix: String,
    pub suffix: String,
    progress: String,
    progress_left: String,
    pub progress_obj: Progress,
}

impl ProgressBar {
    pub fn new(prefix: String, suffix: String, progress: String, progress_left: String, current_value: f64, max_value: f64) -> ProgressBar {
        return ProgressBar {prefix: prefix, suffix: suffix, progress: progress, progress_left: progress_left, progress_obj: Progress::new(current_value, max_value) };
    }

    pub fn percent(&self) -> f64 {
        return self.progress_obj.get_progress_percent();
    }

    pub fn update_progress(&mut self, current_progress: f64) {
        self.progress_obj.set_progress(&current_progress);
    }
    #[allow(dead_code)]
    pub fn get_progress_obj(&mut self) -> &mut Progress {
        return &mut self.progress_obj;
    }

    pub fn show(&mut self) -> std::io::Result<bool>  {
        let mut stdout = stdout();
        let cols: u64 = termsize::get().unwrap().cols as u64;
        let percentage: f64 = 0.3 as f64;
        let cols_to_draw = ((cols as f64) - ((cols as f64 * percentage) as f64)).ceil();

        let cols_progress = (cols_to_draw as usize - self.prefix.len()) - self.suffix.len();
        let mut progress_str = String::new();
        let cols_false = cols_progress as f64 - (cols_progress as f64 * (self.percent() / 100.0)).floor();
        for _i in 0..(cols_progress as f64 * (self.percent() / 100.0)).floor() as i64 {
            progress_str += self.progress.as_str();
        }
        if !self.hasfinished(){
            for _i in 0..cols_false.floor() as i64 {
                progress_str += self.progress_left.as_str();
            }
        }
        let outstr = format!("{}\r{}{}{} {:3.2}%", BACKSPACE, self.prefix, progress_str, self.suffix, self.percent());
        let remaining_cols = cols - outstr.len() as u64;
        let mut overwrite = String::new();
        for _i in 0..(remaining_cols / 2) as u64 {
            overwrite += " ";
        }
        print!("{}{}", outstr, overwrite);
        stdout.flush().unwrap();
        if self.hasfinished() {
            return Ok(true);
        } else {
            return Ok(false);
        }
    }

    pub fn hasfinished(&self) -> bool {
        return self.progress_obj.is_complete();
    }
}
