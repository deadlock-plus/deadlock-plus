#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostHogConfig {
    pub key: String,
    pub host: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Config {
    pub posthog: Option<PostHogConfig>,
    pub sentry_dsn: Option<String>,
}

fn present(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|v| !v.is_empty())
}

pub fn resolve(key: Option<&str>, host: Option<&str>, dsn: Option<&str>, debug: bool) -> Config {
    if debug {
        return Config::default();
    }
    let posthog = present(key)
        .zip(present(host))
        .map(|(key, host)| PostHogConfig { key: key.to_string(), host: host.trim_end_matches('/').to_string() });
    Config { posthog, sentry_dsn: present(dsn).map(str::to_string) }
}

pub fn build() -> Config {
    resolve(
        option_env!("DP_POSTHOG_KEY"),
        option_env!("DP_POSTHOG_HOST"),
        option_env!("DP_SENTRY_DSN"),
        cfg!(debug_assertions),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: Option<&str> = Some("phc_key");
    const HOST: Option<&str> = Some("https://eu.i.posthog.com");
    const DSN: Option<&str> = Some("https://abc@o1.ingest.de.sentry.io/2");

    #[test]
    fn a_release_build_with_every_value_is_configured() {
        let config = resolve(KEY, HOST, DSN, false);
        assert_eq!(
            config.posthog,
            Some(PostHogConfig { key: "phc_key".into(), host: "https://eu.i.posthog.com".into() })
        );
        assert_eq!(config.sentry_dsn.as_deref(), DSN);
    }

    #[test]
    fn a_debug_build_sends_nothing() {
        assert_eq!(resolve(KEY, HOST, DSN, true), Config::default());
    }

    #[test]
    fn missing_or_blank_values_disable_only_their_service() {
        let config = resolve(None, HOST, Some("  "), false);
        assert_eq!(config, Config::default());
        let config = resolve(KEY, None, DSN, false);
        assert_eq!(config.posthog, None);
        assert!(config.sentry_dsn.is_some());
    }

    #[test]
    fn a_trailing_slash_on_the_host_is_dropped() {
        let config = resolve(KEY, Some("https://eu.i.posthog.com/"), None, false);
        assert_eq!(config.posthog.unwrap().host, "https://eu.i.posthog.com");
    }
}
