use socialosint::tui::{ProgressTracker, Status, TuiManager};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_tracker_creation() {
        let tracker = ProgressTracker::new();
        assert_eq!(tracker.instagram_status, Status::Pending);
        assert_eq!(tracker.instagram_count, 0);
        assert_eq!(tracker.linkedin_status, Status::Pending);
        assert_eq!(tracker.linkedin_count, 0);
        assert_eq!(tracker.twitter_status, Status::Pending);
        assert_eq!(tracker.twitter_count, 0);
    }

    #[test]
    fn test_progress_tracker_default() {
        let tracker = ProgressTracker::default();
        assert_eq!(tracker.instagram_status, Status::Pending);
    }

    #[test]
    fn test_status_equality() {
        assert_eq!(Status::Pending, Status::Pending);
        assert_eq!(Status::InProgress, Status::InProgress);
        assert_eq!(Status::Success, Status::Success);
        assert_eq!(Status::Failed, Status::Failed);

        assert_ne!(Status::Pending, Status::InProgress);
    }

    #[test]
    fn test_tui_manager_creation() {
        let tui = TuiManager::new();
        let progress = tui.get_progress();
        assert_eq!(progress.instagram_count, 0);
    }

    #[test]
    fn test_tui_manager_default() {
        let tui = TuiManager::default();
        let progress = tui.get_progress();
        assert_eq!(progress.instagram_count, 0);
    }

    #[test]
    fn test_update_instagram_status() {
        let tui = TuiManager::new();
        tui.update_instagram_status(Status::InProgress);

        let progress = tui.get_progress();
        assert_eq!(progress.instagram_status, Status::InProgress);
    }

    #[test]
    fn test_update_instagram_count() {
        let tui = TuiManager::new();
        tui.update_instagram_count(42);

        let progress = tui.get_progress();
        assert_eq!(progress.instagram_count, 42);
    }

    #[test]
    fn test_update_linkedin_status() {
        let tui = TuiManager::new();
        tui.update_linkedin_status(Status::Success);

        let progress = tui.get_progress();
        assert_eq!(progress.linkedin_status, Status::Success);
    }

    #[test]
    fn test_update_linkedin_count() {
        let tui = TuiManager::new();
        tui.update_linkedin_count(15);

        let progress = tui.get_progress();
        assert_eq!(progress.linkedin_count, 15);
    }

    #[test]
    fn test_update_twitter_status() {
        let tui = TuiManager::new();
        tui.update_twitter_status(Status::Failed);

        let progress = tui.get_progress();
        assert_eq!(progress.twitter_status, Status::Failed);
    }

    #[test]
    fn test_update_twitter_count() {
        let tui = TuiManager::new();
        tui.update_twitter_count(99);

        let progress = tui.get_progress();
        assert_eq!(progress.twitter_count, 99);
    }

    #[test]
    fn test_multiple_updates() {
        let tui = TuiManager::new();

        tui.update_instagram_status(Status::InProgress);
        tui.update_instagram_count(10);
        tui.update_linkedin_status(Status::Success);
        tui.update_linkedin_count(20);
        tui.update_twitter_status(Status::Failed);
        tui.update_twitter_count(5);

        let progress = tui.get_progress();
        assert_eq!(progress.instagram_status, Status::InProgress);
        assert_eq!(progress.instagram_count, 10);
        assert_eq!(progress.linkedin_status, Status::Success);
        assert_eq!(progress.linkedin_count, 20);
        assert_eq!(progress.twitter_status, Status::Failed);
        assert_eq!(progress.twitter_count, 5);
    }

    #[test]
    fn test_display_does_not_panic() {
        let tui = TuiManager::new();
        tui.update_instagram_count(5);
        tui.display();
    }

    #[test]
    fn test_display_summary_does_not_panic() {
        let tui = TuiManager::new();
        tui.update_instagram_count(10);
        tui.update_linkedin_count(20);
        tui.update_twitter_count(30);
        tui.display_summary(1.5);
    }

    #[test]
    fn test_concurrent_updates() {
        use std::sync::Arc;
        use std::thread;

        let tui = Arc::new(TuiManager::new());
        let mut handles = vec![];

        for i in 0..10 {
            let tui_clone = Arc::clone(&tui);
            let handle = thread::spawn(move || {
                tui_clone.update_instagram_count(i);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let progress = tui.get_progress();
        assert!(progress.instagram_count < 10);
    }
}
