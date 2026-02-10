use socialosint::logger::Logger;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_info() {
        let logger = Logger::new();
        logger.info("Test info message");
    }

    #[test]
    fn test_logger_good() {
        let logger = Logger::new();
        logger.good("Test success message");
    }

    #[test]
    fn test_logger_bad() {
        let logger = Logger::new();
        logger.bad("Test error message");
    }

    #[test]
    fn test_logger_debug() {
        let logger = Logger::new();
        logger.debug("Test debug message");
    }

    #[test]
    fn test_legacy_functions() {
        socialosint::logger::info("Legacy info");
        socialosint::logger::good("Legacy good");
        socialosint::logger::bad("Legacy bad");
    }

    #[test]
    fn test_logger_with_special_characters() {
        let logger = Logger::new();
        logger.info("Test with émojis 🚀 and spëcial çhars");
        logger.good("Test with newlines\nand\ttabs");
        logger.bad("Test with quotes \"double\" and 'single'");
    }

    #[test]
    fn test_logger_with_empty_string() {
        let logger = Logger::new();
        logger.info("");
        logger.good("");
        logger.bad("");
        logger.debug("");
    }

    #[test]
    fn test_logger_with_long_message() {
        let logger = Logger::new();
        let long_msg = "A".repeat(10000);
        logger.info(&long_msg);
    }
}
