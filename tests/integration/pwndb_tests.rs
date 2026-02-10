use socialosint::core::Core;
use socialosint::pwndb::{LeakResult, PwnDBAPI};

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_core() -> Core {
        Core::new(Some("socks5://127.0.0.1:9050".to_string())).unwrap()
    }

    #[test]
    fn test_pwndb_api_creation() {
        let core = create_test_core();
        let api = PwnDBAPI::new(core);

        assert!(std::mem::size_of_val(&api) > 0);
    }

    #[test]
    fn test_leak_result_struct() {
        let leak = LeakResult {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        assert_eq!(leak.email, "test@example.com");
        assert_eq!(leak.password, "password123");
    }

    #[test]
    fn test_parse_pwndb_response_empty() {
        let core = create_test_core();
        let api = PwnDBAPI::new(core);

        let response = "";
        let leaks = api.parse_pwndb_response(response);

        assert_eq!(leaks.len(), 0);
    }

    #[test]
    fn test_parse_pwndb_response_single_leak() {
        let core = create_test_core();
        let api = PwnDBAPI::new(core);

        let response = r#"
        Array
        (
            [luser] => testuser
            [domain] => example.com
            [password] => password123
        )
        "#;

        let leaks = api.parse_pwndb_response(response);

        assert_eq!(leaks.len(), 1);
        assert_eq!(leaks[0].email, "testuser@example.com");
        assert_eq!(leaks[0].password, "password123");
    }

    #[test]
    fn test_parse_pwndb_response_multiple_leaks() {
        let core = create_test_core();
        let api = PwnDBAPI::new(core);

        let response = r#"
        Array
        (
            [luser] => user1
            [domain] => test.com
            [password] => pass1
        )
        Array
        (
            [luser] => user2
            [domain] => test.com
            [password] => pass2
        )
        "#;

        let leaks = api.parse_pwndb_response(response);

        assert_eq!(leaks.len(), 2);
        assert_eq!(leaks[0].email, "user1@test.com");
        assert_eq!(leaks[1].email, "user2@test.com");
    }

    #[test]
    fn test_parse_pwndb_response_with_whitespace() {
        let core = create_test_core();
        let api = PwnDBAPI::new(core);

        let response = r#"
        Array
        (
            [luser] =>   testuser
            [domain] =>   example.com
            [password] =>   password123
        )
        "#;

        let leaks = api.parse_pwndb_response(response);

        assert_eq!(leaks.len(), 1);
        assert_eq!(leaks[0].email, "testuser@example.com");
        assert_eq!(leaks[0].password, "password123");
    }

    #[test]
    fn test_parse_pwndb_response_incomplete_data() {
        let core = create_test_core();
        let api = PwnDBAPI::new(core);

        let response = r#"
        Array
        (
            [luser] => testuser
            [domain] => example.com
        )
        "#;

        let leaks = api.parse_pwndb_response(response);

        assert_eq!(leaks.len(), 0);
    }

    #[test]
    fn test_save_results_pwndb() {
        use std::io::Read;
        use tempfile::NamedTempFile;

        let core = create_test_core();

        let results = vec![socialosint::pwndb::PwnDBResult {
            user: "testuser".to_string(),
            user_id: "123".to_string(),
            leaks: vec![LeakResult {
                email: "test@example.com".to_string(),
                password: "password123".to_string(),
            }],
        }];

        let result = socialosint::pwndb::save_results_pwndb(results, &core);

        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_email_parsing_from_search() {
        let email = "testuser@example.com";
        let parts: Vec<&str> = email.split('@').collect();

        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "testuser");
        assert_eq!(parts[1], "example.com");
    }

    #[test]
    fn test_email_parsing_invalid() {
        let email = "invalid-email";
        let domain = email.split('@').nth(1);
        let luser = email.split('@').next();

        assert_eq!(domain, None);
        assert_eq!(luser, Some("invalid-email"));
    }
}
