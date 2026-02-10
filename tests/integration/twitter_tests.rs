use socialosint::core::Core;
use socialosint::twitter::{
    extract_email_from_tweet, extract_emails_from_tweets, Tweet, TwitterAPI,
};

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_core() -> Core {
        Core::new(None).unwrap()
    }

    #[test]
    fn test_extract_email_from_tweet_valid() {
        let tweet = "Contact me at test@example.com for opportunities";
        let email = extract_email_from_tweet(tweet);
        assert_eq!(email, Some("test@example.com".to_string()));
    }

    #[test]
    fn test_extract_email_from_tweet_multiple() {
        let tweet = "Emails: first@test.com and second@test.com";
        let email = extract_email_from_tweet(tweet);
        assert_eq!(email, Some("first@test.com".to_string()));
    }

    #[test]
    fn test_extract_email_from_tweet_no_email() {
        let tweet = "Just a regular tweet without email";
        let email = extract_email_from_tweet(tweet);
        assert_eq!(email, None);
    }

    #[test]
    fn test_extract_email_from_tweet_with_underscore() {
        let tweet = "Reach out: test_user@example.com";
        let email = extract_email_from_tweet(tweet);
        assert_eq!(email, Some("test_user@example.com".to_string()));
    }

    #[test]
    fn test_extract_email_from_tweet_with_dash() {
        let tweet = "Contact: test-user@example.com";
        let email = extract_email_from_tweet(tweet);
        assert_eq!(email, Some("test-user@example.com".to_string()));
    }

    #[test]
    fn test_extract_emails_from_tweets() {
        let core = create_test_core();

        let tweets = vec![
            Tweet {
                username: "user1".to_string(),
                user_id: "123".to_string(),
                tweet: "Email me at user1@test.com".to_string(),
            },
            Tweet {
                username: "user2".to_string(),
                user_id: "456".to_string(),
                tweet: "Contact: user2@test.com".to_string(),
            },
            Tweet {
                username: "user3".to_string(),
                user_id: "789".to_string(),
                tweet: "No email in this tweet".to_string(),
            },
        ];

        let results = extract_emails_from_tweets(tweets, &core);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].email, "user1@test.com");
        assert_eq!(results[1].email, "user2@test.com");
    }

    #[test]
    fn test_extract_emails_deduplication() {
        let core = create_test_core();

        let tweets = vec![
            Tweet {
                username: "user1".to_string(),
                user_id: "123".to_string(),
                tweet: "Email: same@test.com".to_string(),
            },
            Tweet {
                username: "user2".to_string(),
                user_id: "456".to_string(),
                tweet: "Contact: same@test.com".to_string(),
            },
        ];

        let results = extract_emails_from_tweets(tweets, &core);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].email, "same@test.com");
    }

    #[test]
    fn test_twitter_api_creation() {
        let core = create_test_core();
        let api = TwitterAPI::new(core);

        assert!(std::mem::size_of_val(&api) > 0);
    }

    #[test]
    fn test_tweet_struct() {
        let tweet = Tweet {
            username: "testuser".to_string(),
            user_id: "12345".to_string(),
            tweet: "Test tweet content".to_string(),
        };

        assert_eq!(tweet.username, "testuser");
        assert_eq!(tweet.user_id, "12345");
        assert_eq!(tweet.tweet, "Test tweet content");
    }

    #[test]
    fn test_extract_email_case_insensitive() {
        let tweet = "Contact: Test@Example.COM";
        let email = extract_email_from_tweet(tweet);
        assert_eq!(email, Some("Test@Example.COM".to_string()));
    }

    #[test]
    fn test_extract_emails_sorting() {
        let core = create_test_core();

        let tweets = vec![
            Tweet {
                username: "user1".to_string(),
                user_id: "1".to_string(),
                tweet: "Email: zebra@test.com".to_string(),
            },
            Tweet {
                username: "user2".to_string(),
                user_id: "2".to_string(),
                tweet: "Email: alpha@test.com".to_string(),
            },
        ];

        let results = extract_emails_from_tweets(tweets, &core);

        assert_eq!(results[0].email, "alpha@test.com");
        assert_eq!(results[1].email, "zebra@test.com");
    }
}
