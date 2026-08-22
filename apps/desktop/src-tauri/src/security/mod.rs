//! Path validation, filename sanitization and security-sensitive policies.

use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use url::Url;

/// Calculates a SHA-256 digest without loading the complete file into memory.
pub fn sha256_file(path: &Path) -> Result<String, String> {
    let file = File::open(path).map_err(|error| format!("Could not open file: {error}"))?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|error| format!("Could not read file: {error}"))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Redacts credential-bearing URL components before a value is written to logs.
pub fn redact_sensitive_text(value: &str) -> String {
    let trimmed = value.trim();
    if let Ok(mut url) = Url::parse(trimmed) {
        if !matches!(url.scheme(), "http" | "https") {
            return redact_header_text(trimmed);
        }
        let _ = url.set_username("");
        let _ = url.set_password(None);
        if url.query().is_some() {
            let redacted_query = url
                .query_pairs()
                .map(|(key, query_value)| {
                    let lower = key.to_ascii_lowercase();
                    let safe_value = if [
                        "token",
                        "access_token",
                        "refresh_token",
                        "api_key",
                        "apikey",
                        "key",
                        "password",
                        "passwd",
                        "secret",
                        "signature",
                        "sig",
                    ]
                    .iter()
                    .any(|secret| lower == *secret || lower.contains(secret))
                    {
                        "[REDACTED]".to_string()
                    } else {
                        query_value.into_owned()
                    };
                    format!("{}={}", key, safe_value)
                })
                .collect::<Vec<_>>()
                .join("&");
            url.set_query(Some(&redacted_query));
        }
        return url.to_string();
    }

    redact_header_text(trimmed)
}

fn redact_header_text(trimmed: &str) -> String {
    let mut output = trimmed.to_string();
    for marker in [
        "Authorization:",
        "authorization:",
        "Cookie:",
        "cookie:",
        "X-Api-Key:",
        "x-api-key:",
    ] {
        if let Some(start) = output.find(marker) {
            let value_start = start + marker.len();
            let end = output[value_start..]
                .find([';', '\n', '\r'])
                .map(|offset| value_start + offset)
                .unwrap_or(output.len());
            output.replace_range(value_start..end, " [REDACTED]");
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{redact_sensitive_text, sha256_file};
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn redacts_url_credentials_and_secret_query_values() {
        let value = redact_sensitive_text(
            "https://alice:password@example.com/file.zip?token=abc123&download=1",
        );
        assert_eq!(
            value,
            "https://example.com/file.zip?token=[REDACTED]&download=1"
        );
        assert!(!value.contains("password"));
        assert!(!value.contains("abc123"));
    }

    #[test]
    fn redacts_auth_and_cookie_headers_in_plain_text() {
        let value = redact_sensitive_text("Authorization: Bearer abc; Cookie: sid=secret\nurl");
        assert!(!value.contains("Bearer abc"));
        assert!(!value.contains("sid=secret"));
        assert!(value.contains("[REDACTED]"));
    }

    #[test]
    fn hashes_file_contents_without_loading_the_whole_file() {
        let path = PathBuf::from(std::env::temp_dir())
            .join(format!("zynero-hash-test-{}.txt", std::process::id()));
        fs::write(&path, b"ZYNERO hash fixture").expect("fixture should be written");
        let digest = sha256_file(&path).expect("fixture should hash");
        assert_eq!(
            digest,
            "4ca9cbe977fa0865dbd19e3b5e15bfe60c9cdfb61aa3a86a916ef5046f6e6e64"
        );
        fs::remove_file(path).expect("fixture should be removed");
    }

    #[test]
    fn preserves_non_sensitive_url_query_values() {
        assert_eq!(
            redact_sensitive_text("https://example.com/file.zip?part=1"),
            "https://example.com/file.zip?part=1"
        );
    }
}
