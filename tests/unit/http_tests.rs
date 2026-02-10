use socialosint::http::HttpClient;
use reqwest::header::HeaderMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_http_client_creation() {
        let client = HttpClient::new(None);
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_http_client_with_proxy() {
        let client = HttpClient::new(Some("socks5://127.0.0.1:9050".to_string()));
        assert!(client.is_ok());
    }

    #[test]
    fn test_random_user_agent() {
        let ua1 = HttpClient::random_user_agent();
        let ua2 = HttpClient::random_user_agent();

        assert!(!ua1.is_empty());
        assert!(!ua2.is_empty());
        assert!(ua1.contains("Mozilla"));
    }

    #[tokio::test]
    async fn test_http_get_with_headers() {
        use mockito::Server;

        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/test")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"status": "ok"}"#)
            .create_async()
            .await;

        let client = HttpClient::new(None).unwrap();
        let url = format!("{}/test", server.url());

        let result = client.get(&url, HeaderMap::new()).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.status(), 200);

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_http_post_with_body() {
        use mockito::Server;

        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/api")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(r#"{"created": true}"#)
            .create_async()
            .await;

        let client = HttpClient::new(None).unwrap();
        let url = format!("{}/api", server.url());

        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());

        let result = client
            .post(&url, headers, r#"{"test": "data"}"#.to_string())
            .await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.status(), 201);

        mock.assert_async().await;
    }

    #[test]
    fn test_raw_client_access() {
        let client = HttpClient::new(None).unwrap();
        let raw = client.raw_client();

        assert!(raw.get("https://example.com").build().is_ok());
    }
}
