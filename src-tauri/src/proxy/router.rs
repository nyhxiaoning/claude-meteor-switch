use crate::config::provider::Provider;
use regex::Regex;

/// Match model name to a provider using keyword matching.
/// Priority:
/// 1. If only one provider is enabled, use it for every model.
/// 2. With multiple providers, exact match > keyword boundary match.
/// 3. If no model was selected or no keyword matched, prefer sonnet.
/// 4. If no sonnet provider exists, use the first enabled provider.
pub fn match_provider<'a>(model: &str, providers: &'a [Provider]) -> Option<&'a Provider> {
    if providers.len() <= 1 {
        return providers.first();
    }

    // Exact match on keyword
    if let Some(p) = providers.iter().find(|p| p.keyword == model) {
        return Some(p);
    }

    // Keyword boundary match: (^|-)keyword(-|$)
    for provider in providers {
        if let Ok(re) = Regex::new(&format!(r"(^|-){}(-|$)", regex::escape(&provider.keyword))) {
            if re.is_match(model) {
                return Some(provider);
            }
        }
    }

    // Default model choice: Claude defaults to sonnet-like behavior unless the user switches.
    if let Some(p) = providers.iter().find(|p| p.keyword == "sonnet") {
        return Some(p);
    }

    providers.first()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::provider::{AuthHeader, Protocol};

    fn make_provider(keyword: &str) -> Provider {
        Provider {
            id: format!("test-{}", keyword),
            name: format!("Test {}", keyword),
            base_url: "https://api.test.com".to_string(),
            api_key_enc: "".to_string(),
            protocol: Protocol::Anthropic,
            model_mapping: None,
            auth_header: AuthHeader::ApiKey,
            keyword: keyword.to_string(),
            enabled: true,
            sort_order: 0,
        }
    }

    #[test]
    fn test_keyword_match() {
        let providers = vec![
            make_provider("opus"),
            make_provider("sonnet"),
            make_provider("haiku"),
        ];

        assert_eq!(
            match_provider("claude-opus-4-6", &providers)
                .unwrap()
                .keyword,
            "opus"
        );
        assert_eq!(
            match_provider("claude-sonnet-4-6", &providers)
                .unwrap()
                .keyword,
            "sonnet"
        );
        assert_eq!(
            match_provider("claude-haiku-4-5", &providers)
                .unwrap()
                .keyword,
            "haiku"
        );
    }

    #[test]
    fn test_exact_match() {
        let providers = vec![make_provider("opus")];
        assert_eq!(match_provider("opus", &providers).unwrap().keyword, "opus");
    }

    #[test]
    fn test_fallback() {
        let providers = vec![make_provider("opus")];
        assert_eq!(
            match_provider("unknown-model", &providers).unwrap().keyword,
            "opus"
        );
    }

    #[test]
    fn test_single_provider_ignores_model_family() {
        let providers = vec![make_provider("haiku")];
        assert_eq!(
            match_provider("claude-opus-4-6", &providers)
                .unwrap()
                .keyword,
            "haiku"
        );
        assert_eq!(
            match_provider("claude-sonnet-4-6", &providers)
                .unwrap()
                .keyword,
            "haiku"
        );
        assert_eq!(
            match_provider("unknown-model", &providers).unwrap().keyword,
            "haiku"
        );
    }

    #[test]
    fn test_fallback_prefers_sonnet() {
        let providers = vec![
            make_provider("opus"),
            make_provider("sonnet"),
            make_provider("haiku"),
        ];
        // Should pick sonnet even though it is not first.
        assert_eq!(
            match_provider("unknown-model", &providers).unwrap().keyword,
            "sonnet"
        );
    }

    #[test]
    fn test_fallback_no_sonnet() {
        let providers = vec![make_provider("opus"), make_provider("haiku")];
        // No sonnet, should pick first.
        assert_eq!(
            match_provider("unknown-model", &providers).unwrap().keyword,
            "opus"
        );
    }
}
