use super::UrlOutcome;
use std::sync::Mutex;

pub struct UrlOutcomes(Mutex<Vec<UrlOutcome>>);

impl Default for UrlOutcomes {
    fn default() -> Self {
        UrlOutcomes(Mutex::new(vec![]))
    }
}

impl UrlOutcomes {
    pub fn push(&self, outcome: UrlOutcome) {
        let mut inner = self.0.lock().unwrap();
        inner.push(outcome)
    }

    pub fn values(&self) -> Vec<UrlOutcome> {
        self.0.lock().unwrap().to_vec()
    }
}
