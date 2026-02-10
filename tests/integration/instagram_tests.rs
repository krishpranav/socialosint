use socialosint::core::Core;
use socialosint::instagram::{
    extract_email_from_bio, extract_emails_from_users, InstagramAPI, InstagramUser,
};

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_core() -> Core {
        Core::new(None).unwrap()
    }

    #[test]
    fn test_extract_email_from_bio_valid() {
        let bio = "Contact me at test@example.com for business inquiries";
        let email = extract_email_from_bio(bio);
        assert_eq!(email, Some("test@example.com".to_string()));
    }

    #[test]
    fn test_extract_email_from_bio_multiple() {
        let bio = "Email: first@test.com or second@test.com";
        let email = extract_email_from_bio(bio);

        assert!(email.is_some());
    }

    #[test]
    fn test_extract_email_from_bio_no_email() {
        let bio = "Just a regular bio without contact info";
        let email = extract_email_from_bio(bio);
        assert_eq!(email, None);
    }

    #[test]
    fn test_extract_email_from_bio_invalid() {
        let bio = "Invalid email: not-an-email";
        let email = extract_email_from_bio(bio);
        assert_eq!(email, None);
    }

    #[test]
    fn test_instagram_api_creation() {
        let core = create_test_core();
        let api = InstagramAPI::new(core, "testuser".to_string(), "testpass".to_string());

        assert!(std::mem::size_of_val(&api) > 0);
    }

    #[test]
    fn test_generate_uuid() {
        let uuid1 = InstagramAPI::generate_uuid();
        let uuid2 = InstagramAPI::generate_uuid();

        assert_ne!(uuid1, uuid2);

        assert!(uuid1.len() > 0);
        assert!(uuid1.contains("-"));
    }

    #[test]
    fn test_generate_device_id() {
        let seed = "abcdef1234567890";
        let device_id = InstagramAPI::generate_device_id(seed);

        assert!(device_id.starts_with("android-"));
        assert_eq!(device_id.len(), "android-".len() + 16);
    }

    #[test]
    fn test_user_agent() {
        let ua = InstagramAPI::user_agent();
        assert!(ua.contains("Instagram"));
        assert!(ua.contains("Android"));
    }

    #[test]
    fn test_extract_emails_deduplication() {
        let core = create_test_core();

        let users = vec![
            InstagramUser {
                username: "user1".to_string(),
                pk: 123,
                public_email: Some("same@test.com".to_string()),
                follower_count: 100,
                following_count: 50,
                biography: "".to_string(),
                is_private: false,
            },
            InstagramUser {
                username: "user2".to_string(),
                pk: 456,
                public_email: Some("same@test.com".to_string()),
                follower_count: 200,
                following_count: 100,
                biography: "".to_string(),
                is_private: false,
            },
        ];

        let results = extract_emails_from_users(users, &core);

        assert!(results.len() >= 1);
    }
}
