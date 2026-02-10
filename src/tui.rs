use std::io::{self, Write};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Status {
    Pending,
    InProgress,
    Success,
    Failed,
}

#[derive(Debug, Clone)]
pub struct ProgressTracker {
    pub instagram_status: Status,
    pub instagram_count: usize,
    pub linkedin_status: Status,
    pub linkedin_count: usize,
    pub twitter_status: Status,
    pub twitter_count: usize,
}

impl ProgressTracker {
    pub fn new() -> Self {
        Self {
            instagram_status: Status::Pending,
            instagram_count: 0,
            linkedin_status: Status::Pending,
            linkedin_count: 0,
            twitter_status: Status::Pending,
            twitter_count: 0,
        }
    }

    fn status_symbol(status: Status) -> &'static str {
        match status {
            Status::Pending => "○",
            Status::InProgress => "◐",
            Status::Success => "●",
            Status::Failed => "✗",
        }
    }

    fn status_color(status: Status) -> &'static str {
        match status {
            Status::Pending => "\x1b[90m",
            Status::InProgress => "\x1b[93m",
            Status::Success => "\x1b[92m",
            Status::Failed => "\x1b[91m",
        }
    }

    pub fn display(&self) {
        print!("\r\x1b[K");

        print!(
            "{}{} Instagram: {} | ",
            Self::status_color(self.instagram_status),
            Self::status_symbol(self.instagram_status),
            self.instagram_count
        );

        print!(
            "{}{} LinkedIn: {} | ",
            Self::status_color(self.linkedin_status),
            Self::status_symbol(self.linkedin_status),
            self.linkedin_count
        );

        print!(
            "{}{} Twitter: {}\x1b[0m",
            Self::status_color(self.twitter_status),
            Self::status_symbol(self.twitter_status),
            self.twitter_count
        );

        io::stdout().flush().unwrap();
    }

    pub fn display_summary(&self, elapsed: f64) {
        println!("\n\n\x1b[1m=== Summary ===\x1b[0m");
        println!("Instagram: {} emails found", self.instagram_count);
        println!("LinkedIn: {} emails found", self.linkedin_count);
        println!("Twitter: {} emails found", self.twitter_count);
        println!(
            "Total: {} emails",
            self.instagram_count + self.linkedin_count + self.twitter_count
        );
        println!("Time elapsed: {:.2}s", elapsed);
    }
}

impl Default for ProgressTracker {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TuiManager {
    progress: Arc<Mutex<ProgressTracker>>,
}

impl TuiManager {
    pub fn new() -> Self {
        Self {
            progress: Arc::new(Mutex::new(ProgressTracker::new())),
        }
    }

    pub fn update_instagram_status(&self, status: Status) {
        let mut progress = self.progress.lock().unwrap();
        progress.instagram_status = status;
        progress.display();
    }

    pub fn update_instagram_count(&self, count: usize) {
        let mut progress = self.progress.lock().unwrap();
        progress.instagram_count = count;
        progress.display();
    }

    pub fn update_linkedin_status(&self, status: Status) {
        let mut progress = self.progress.lock().unwrap();
        progress.linkedin_status = status;
        progress.display();
    }

    pub fn update_linkedin_count(&self, count: usize) {
        let mut progress = self.progress.lock().unwrap();
        progress.linkedin_count = count;
        progress.display();
    }

    pub fn update_twitter_status(&self, status: Status) {
        let mut progress = self.progress.lock().unwrap();
        progress.twitter_status = status;
        progress.display();
    }

    pub fn update_twitter_count(&self, count: usize) {
        let mut progress = self.progress.lock().unwrap();
        progress.twitter_count = count;
        progress.display();
    }

    pub fn display(&self) {
        let progress = self.progress.lock().unwrap();
        progress.display();
    }

    pub fn display_summary(&self, elapsed: f64) {
        let progress = self.progress.lock().unwrap();
        progress.display_summary(elapsed);
    }

    pub fn get_progress(&self) -> ProgressTracker {
        self.progress.lock().unwrap().clone()
    }
}

impl Default for TuiManager {
    fn default() -> Self {
        Self::new()
    }
}

pub fn display_banner() {
    let banner = r#"
╔═══════════════════════════════════════════════════════════╗
║                                                           ║
║              Social OSINT Tool (Rust Edition)             ║
║                                                           ║
║  Instagram • LinkedIn • Twitter • PwnDB • HaveIBeenPwned  ║
║                                                           ║
╚═══════════════════════════════════════════════════════════╝
"#;
    println!("\x1b[96m{}\x1b[0m", banner);
}
