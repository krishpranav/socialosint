struct Colors;

impl Colors {
    const RESET: &'static str = "\x1b[0m";
    const RED: &'static str = "\x1b[91m";
    const GREEN: &'static str = "\x1b[92m";
    const YELLOW: &'static str = "\x1b[93m";
    const BLUE: &'static str = "\x1b[94m";
    const VIOLET: &'static str = "\x1b[95m";
    const CYAN: &'static str = "\x1b[96m";
    const WHITE: &'static str = "\x1b[97m";
    const BOLD: &'static str = "\x1b[1m";
    const UNDERLINE: &'static str = "\x1b[4m";
}

pub struct Logger;

impl Logger {
    pub fn new() -> Self {
        Self
    }

    pub fn info(&self, msg: &str) {
        println!(
            "{}{}[-]{} {}",
            Colors::BOLD,
            Colors::BLUE,
            Colors::RESET,
            msg
        );
        tracing::info!("{}", msg);
    }

    pub fn good(&self, msg: &str) {
        println!(
            "{}{}[+]{} {}",
            Colors::BOLD,
            Colors::GREEN,
            Colors::RESET,
            msg
        );
        tracing::info!("{}", msg);
    }

    pub fn bad(&self, msg: &str) {
        println!(
            "{}{}[!]{} {}",
            Colors::BOLD,
            Colors::RED,
            Colors::RESET,
            msg
        );
        tracing::warn!("{}", msg);
    }

    pub fn debug(&self, msg: &str) {
        println!(
            "{}{}[D]{} {}",
            Colors::BOLD,
            Colors::CYAN,
            Colors::RESET,
            msg
        );
        tracing::debug!("{}", msg);
    }

    pub fn field(&self, key: &str, value: &str) {
        println!("  {}{}{}: {}", Colors::YELLOW, key, Colors::RESET, value);
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

pub fn info(msg: &str) {
    Logger::new().info(msg);
}

pub fn good(msg: &str) {
    Logger::new().good(msg);
}

pub fn bad(msg: &str) {
    Logger::new().bad(msg);
}
