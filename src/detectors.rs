use reqwest::Response;

#[derive(Debug, Clone, Copy)]
pub enum Platform {
    Instagram,
    LinkedIn,
    Twitter,
    PwnDB,
}

#[derive(Debug, Clone)]
pub enum ProfileStatus {
    Exists,
    Private,
    NotFound,
    Blocked,
    RateLimited,
    Error(String),
}

pub struct Detectors;

impl Detectors {
    pub fn new() -> Self {
        Self
    }

    pub fn detect_blocking(&self, response: &Response) -> bool {
        let status = response.status().as_u16();

        if status == 403 || status == 429 {
            return true;
        }

        if let Some(server) = response.headers().get("server") {
            if let Ok(server_str) = server.to_str() {
                if server_str.to_lowercase().contains("cloudflare") {
                    return true;
                }
            }
        }

        if response.headers().contains_key("cf-chl-bypass") {
            return true;
        }

        false
    }

    pub fn detect_profile_status(
        &self,
        response: &Response,
        body: &str,
        platform: Platform,
    ) -> ProfileStatus {
        if self.detect_blocking(response) {
            return ProfileStatus::Blocked;
        }

        let status = response.status().as_u16();

        if status == 429 {
            return ProfileStatus::RateLimited;
        }

        if status == 404 {
            return ProfileStatus::NotFound;
        }

        if self.detect_soft_404(body, platform) {
            return ProfileStatus::NotFound;
        }

        if self.detect_private(body, platform) {
            return ProfileStatus::Private;
        }

        if status >= 200 && status < 300 {
            return ProfileStatus::Exists;
        }

        ProfileStatus::Error(format!("HTTP {}", status))
    }

    pub fn detect_soft_404(&self, body: &str, platform: Platform) -> bool {
        let body_lower = body.to_lowercase();

        match platform {
            Platform::Instagram => {
                body_lower.contains("sorry, this page isn't available")
                    || body_lower.contains("page not found")
                    || body_lower.contains("user not found")
            }
            Platform::LinkedIn => {
                body_lower.contains("page not found")
                    || body_lower.contains("profile not found")
                    || body_lower.contains("member not found")
            }
            Platform::Twitter => {
                body_lower.contains("this account doesn't exist")
                    || body_lower.contains("page doesn't exist")
                    || body_lower.contains("user not found")
            }
            Platform::PwnDB => false,
        }
    }

    pub fn detect_private(&self, body: &str, platform: Platform) -> bool {
        let body_lower = body.to_lowercase();

        match platform {
            Platform::Instagram => {
                body_lower.contains("this account is private")
                    || body_lower.contains("\"is_private\":true")
            }
            Platform::LinkedIn => {
                body_lower.contains("sign in to view") || body_lower.contains("private profile")
            }
            Platform::Twitter => {
                body_lower.contains("these tweets are protected")
                    || body_lower.contains("this account is private")
            }
            Platform::PwnDB => false,
        }
    }

    pub fn detect_captcha(&self, body: &str) -> bool {
        let body_lower = body.to_lowercase();

        body_lower.contains("captcha")
            || body_lower.contains("recaptcha")
            || body_lower.contains("hcaptcha")
            || body_lower.contains("challenge")
    }

    pub fn is_redirect(&self, response: &Response) -> bool {
        response.status().is_redirection()
    }
}

impl Default for Detectors {
    fn default() -> Self {
        Self::new()
    }
}
