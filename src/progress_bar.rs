use indicatif::ProgressBar;
use std::convert::TryInto;
use std::sync::Mutex;

pub struct UrlProgress(Mutex<ProgressBar>);

impl UrlProgress {
    pub fn for_urls<T>(urls: &Vec<T>) -> Self {
        UrlProgress(Mutex::new(ProgressBar::new(urls.len().try_into().unwrap())))
    }

    pub fn incr(&self) {
        self.0.lock().unwrap().inc(1)
    }

    pub fn finish(&self) {
        self.0.lock().unwrap().finish()
    }
}
