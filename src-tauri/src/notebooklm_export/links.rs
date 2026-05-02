pub(crate) fn detect_urls(text: &str) -> Vec<String> {
    let mut urls = Vec::new();

    for token in text.split_whitespace() {
        let trimmed = token
            .trim_start_matches(['(', '[', '{', '<'])
            .trim_end_matches(['.', ',', ';', ':', '!', '?', ')', ']', '}', '>']);

        if (trimmed.starts_with("http://") || trimmed.starts_with("https://"))
            && !urls.iter().any(|url| url == trimmed)
        {
            urls.push(trimmed.to_string());
        }
    }

    urls
}

#[cfg(test)]
mod tests {
    use super::detect_urls;

    #[test]
    fn detects_and_trims_http_urls() {
        assert_eq!(
            detect_urls("See (https://example.com/a), and https://example.com/a."),
            vec!["https://example.com/a"]
        );
        assert_eq!(
            detect_urls("See <https://example.com/a>"),
            vec!["https://example.com/a"]
        );
    }
}
