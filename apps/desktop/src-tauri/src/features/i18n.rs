use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

const EN: &str = include_str!("../../../../../locales/en.json");
const DEFAULT_LOCALE: &str = "en";

fn catalogs() -> &'static HashMap<&'static str, Value> {
    static CATALOGS: OnceLock<HashMap<&'static str, Value>> = OnceLock::new();
    CATALOGS.get_or_init(|| {
        HashMap::from([(DEFAULT_LOCALE, serde_json::from_str(EN).expect("locales/en.json is valid JSON"))])
    })
}

fn lookup<'a>(catalog: &'a Value, key: &str) -> Option<&'a str> {
    key.split('.').try_fold(catalog, |node, part| node.get(part))?.as_str()
}

fn interpolate(template: &str, params: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        match after.find('}') {
            Some(end) => {
                let name = &after[..end];
                match params.iter().find(|(n, _)| *n == name) {
                    Some((_, value)) => out.push_str(value),
                    None => out.push_str(&rest[start..start + end + 2]),
                }
                rest = &after[end + 1..];
            }
            None => {
                out.push_str(&rest[start..]);
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out
}

fn chain<'a>(locale: &str, all: &'a HashMap<&'static str, Value>) -> Vec<&'a Value> {
    let base = locale.split('-').next().unwrap_or(locale);
    let mut out: Vec<&Value> = Vec::new();
    for code in [locale, base, DEFAULT_LOCALE] {
        if let Some(catalog) = all.get(code) {
            if !out.iter().any(|c| std::ptr::eq(*c, catalog)) {
                out.push(catalog);
            }
        }
    }
    out
}

pub fn translate_in(locale: &str, key: &str, params: &[(&str, &str)]) -> String {
    for catalog in chain(locale, catalogs()) {
        if let Some(template) = lookup(catalog, key) {
            return interpolate(template, params);
        }
    }
    log::warn!("missing i18n key: {key}");
    key.to_string()
}

pub struct I18nState {
    locale: Mutex<String>,
}

impl Default for I18nState {
    fn default() -> Self {
        Self { locale: Mutex::new(DEFAULT_LOCALE.to_string()) }
    }
}

impl I18nState {
    pub fn locale(&self) -> String {
        self.locale.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// Returns whether the locale changed. The frontend re-sends its language on every reload, so a repeat is a no-op.
    pub fn set(&self, locale: &str) -> bool {
        let mut current = self.locale.lock().unwrap_or_else(|e| e.into_inner());
        if *current == locale {
            return false;
        }
        *current = locale.to_string();
        true
    }

    pub fn tr(&self, key: &str, params: &[(&str, &str)]) -> String {
        translate_in(&self.locale(), key, params)
    }
}

pub mod commands {
    use super::*;
    use tauri::{AppHandle, State};

    #[tauri::command]
    pub fn set_language(locale: String, app: AppHandle, state: State<'_, I18nState>) {
        if state.set(&locale) {
            log::info!("language set to {locale}");
            crate::features::tray::apply_language(&app);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translates_a_dotted_key() {
        assert_eq!(translate_in("en", "tray.quit", &[]), "Quit");
    }

    #[test]
    fn fills_placeholders_and_keeps_unknown_ones() {
        assert_eq!(interpolate("Hi {name}, {other}.", &[("name", "A")]), "Hi A, {other}.");
        assert_eq!(interpolate("open { brace", &[]), "open { brace");
    }

    #[test]
    fn unknown_locale_falls_back_to_en() {
        assert_eq!(translate_in("de-DE", "tray.quit", &[]), "Quit");
    }

    #[test]
    fn missing_key_returns_the_key() {
        assert_eq!(translate_in("en", "nope.nothing", &[]), "nope.nothing");
    }

    #[test]
    fn setting_the_same_locale_twice_changes_once() {
        let state = I18nState::default();
        assert!(!state.set("en"));
        assert!(state.set("fr"));
        assert!(!state.set("fr"));
        assert_eq!(state.locale(), "fr");
    }

    #[test]
    fn state_translates_in_its_locale() {
        let state = I18nState::default();
        assert_eq!(state.tr("tray.show", &[]), "Show Deadlock+");
    }
}
