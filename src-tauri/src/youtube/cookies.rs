use crate::error::{AppError, AppResult};
use crate::secret_store::{youtube_default_cookies_secret, SecretStoreState};

pub(crate) async fn read_youtube_cookies(secrets: &SecretStoreState) -> AppResult<Option<String>> {
    secrets.get_secret(youtube_default_cookies_secret()).await
}

pub(crate) async fn save_youtube_cookies(
    secrets: &SecretStoreState,
    cookies: String,
) -> AppResult<()> {
    validate_netscape_cookie_file(&cookies)?;
    secrets
        .set_secret(youtube_default_cookies_secret(), cookies)
        .await
}

pub(crate) async fn clear_youtube_cookies(secrets: &SecretStoreState) -> AppResult<()> {
    secrets
        .delete_secret(youtube_default_cookies_secret())
        .await
}

pub(crate) fn validate_netscape_cookie_file(cookies: &str) -> AppResult<()> {
    let mut cookie_rows = 0usize;

    if cookies.trim().is_empty() {
        return Err(AppError::validation(
            "YouTube cookies cannot be empty; use clear_youtube_auth to remove stored cookies",
        ));
    }

    for (index, line) in cookies.lines().enumerate() {
        let line_number = index + 1;
        let trimmed = line.trim_matches(|ch| ch == ' ' || ch == '\r');
        if trimmed.trim().is_empty() {
            continue;
        }
        if trimmed.starts_with('#') && !trimmed.starts_with("#HttpOnly_") {
            continue;
        }

        let cookie_line = trimmed.strip_prefix("#HttpOnly_").unwrap_or(trimmed);
        let fields: Vec<&str> = cookie_line.split('\t').collect();
        if fields.len() != 7 {
            return Err(AppError::validation(format!(
                "Invalid YouTube cookie file at line {line_number}: expected 7 tab-separated fields",
            )));
        }

        let domain = fields[0].trim();
        let include_subdomains = fields[1].trim();
        let path = fields[2].trim();
        let secure = fields[3].trim();
        let expires = fields[4].trim();
        let name = fields[5].trim();

        if domain.is_empty() {
            return Err(AppError::validation(format!(
                "Invalid YouTube cookie file at line {line_number}: domain is empty",
            )));
        }
        if include_subdomains != "TRUE" && include_subdomains != "FALSE" {
            return Err(AppError::validation(format!(
                "Invalid YouTube cookie file at line {line_number}: include-subdomains must be TRUE or FALSE",
            )));
        }
        if !path.starts_with('/') {
            return Err(AppError::validation(format!(
                "Invalid YouTube cookie file at line {line_number}: path must start with /",
            )));
        }
        if secure != "TRUE" && secure != "FALSE" {
            return Err(AppError::validation(format!(
                "Invalid YouTube cookie file at line {line_number}: secure must be TRUE or FALSE",
            )));
        }
        if expires.parse::<i64>().is_err() {
            return Err(AppError::validation(format!(
                "Invalid YouTube cookie file at line {line_number}: expires must be an integer timestamp",
            )));
        }
        if name.is_empty() {
            return Err(AppError::validation(format!(
                "Invalid YouTube cookie file at line {line_number}: cookie name is empty",
            )));
        }

        cookie_rows += 1;
    }

    if cookie_rows == 0 {
        return Err(AppError::validation(
            "YouTube cookies must contain at least one Netscape cookie row",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::secret_store::{tests::InMemorySecretStore, SecretStoreState};

    use super::{
        clear_youtube_cookies, read_youtube_cookies, save_youtube_cookies,
        validate_netscape_cookie_file,
    };

    #[test]
    fn validates_netscape_cookie_rows_without_exposing_values() {
        let cookies =
            "# Netscape HTTP Cookie File\n.youtube.com\tTRUE\t/\tTRUE\t1893456000\tSID\tsecret-value\n";
        assert!(validate_netscape_cookie_file(cookies).is_ok());

        let err =
            validate_netscape_cookie_file(".youtube.com TRUE / TRUE 1893456000 SID secret-value")
                .expect_err("space separated cookies should fail");

        assert!(!err.message.contains("secret-value"));
    }

    #[test]
    fn accepts_http_only_cookie_rows() {
        let cookies = "#HttpOnly_.youtube.com\tTRUE\t/\tTRUE\t1893456000\tSID\tsecret-value\n";

        assert!(validate_netscape_cookie_file(cookies).is_ok());
    }

    #[test]
    fn accepts_empty_cookie_values() {
        let cookies = ".youtube.com\tTRUE\t/\tTRUE\t1893456000\tSID\t\n";

        assert!(validate_netscape_cookie_file(cookies).is_ok());
    }

    #[test]
    fn rejects_empty_cookie_text() {
        let err =
            validate_netscape_cookie_file("  \n\t").expect_err("empty cookie text should fail");

        assert!(err.message.contains("cannot be empty"));
    }

    #[test]
    fn rejects_files_without_cookie_rows() {
        let err = validate_netscape_cookie_file("# Netscape HTTP Cookie File\n# only comments\n")
            .expect_err("comments-only file should fail");

        assert!(err.message.contains("at least one"));
    }

    #[tokio::test]
    async fn stores_reads_and_clears_youtube_cookies_through_secret_store() {
        let store = Arc::new(InMemorySecretStore::new());
        let secrets = SecretStoreState::new(store);
        let cookies = ".youtube.com\tTRUE\t/\tTRUE\t1893456000\tSID\tsecret-value\n".to_string();

        save_youtube_cookies(&secrets, cookies.clone())
            .await
            .expect("save cookies");

        assert_eq!(
            read_youtube_cookies(&secrets).await.expect("read cookies"),
            Some(cookies)
        );

        clear_youtube_cookies(&secrets)
            .await
            .expect("clear cookies");

        assert_eq!(read_youtube_cookies(&secrets).await.unwrap(), None);
    }

    #[tokio::test]
    async fn rejects_invalid_cookie_text_before_saving_secret() {
        let store = Arc::new(InMemorySecretStore::new());
        let secrets = SecretStoreState::new(store);

        let err = save_youtube_cookies(
            &secrets,
            ".youtube.com TRUE / TRUE 1893456000 SID secret-value".to_string(),
        )
        .await
        .expect_err("invalid cookies should fail");

        assert!(!err.message.contains("secret-value"));
        assert_eq!(read_youtube_cookies(&secrets).await.unwrap(), None);
    }
}
