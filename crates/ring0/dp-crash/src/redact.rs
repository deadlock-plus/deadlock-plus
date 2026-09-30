const PATH_MARKERS: [&str; 3] = ["\\users\\", "/users/", "/home/"];
const SECRET_KEYS: [&str; 5] = ["token", "bearer", "secret", "password", "api_key"];

/// Masks the account name in `C:\Users\<name>\`, `/Users/<name>/` and `/home/<name>/` paths.
pub fn redact_user_paths(text: &str) -> String {
    let lower = text.to_ascii_lowercase();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < text.len() {
        let marker = PATH_MARKERS.iter().find(|m| lower[i..].starts_with(**m) && starts_a_path(text, i, m));
        if let Some(marker) = marker {
            let name_start = i + marker.len();
            let name_len = text[name_start..]
                .find(|c: char| matches!(c, '\\' | '/' | '"' | '\'' | '`') || c.is_whitespace())
                .unwrap_or(text.len() - name_start);
            out.push_str(&text[i..name_start]);
            if name_len > 0 {
                out.push_str("<user>");
            }
            i = name_start + name_len;
        } else {
            let ch = text[i..].chars().next().expect("index is on a char boundary");
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    out
}

/// `/home/` also shows up inside URLs and relative paths; only a path root, not a segment in the middle
/// of a word, names a user directory.
fn starts_a_path(text: &str, at: usize, marker: &str) -> bool {
    if marker != "/home/" {
        return true;
    }
    text[..at].chars().next_back().is_none_or(|c| !(c.is_alphanumeric() || matches!(c, '.' | '-' | '_' | '/')))
}

/// Masks the value after `token`, `bearer`, `secret`, `password` and `api_key`, in `key=value`,
/// `key: value` and `"key": "value"` shapes.
pub fn redact_secrets(text: &str) -> String {
    let lower = text.to_ascii_lowercase();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < text.len() {
        if let Some(key) = SECRET_KEYS.iter().find(|k| lower[i..].starts_with(**k)) {
            let key_end = i + key.len();
            let separator_end = key_end
                + text[key_end..].find(|c: char| !matches!(c, '"' | '\'' | ' ' | '\t' | '=' | ':')).unwrap_or(0);
            let separator = &text[key_end..separator_end];
            let has_separator = separator.contains(['=', ':']) || (*key == "bearer" && !separator.is_empty());
            let value_len = text[separator_end..]
                .find(|c: char| c.is_whitespace() || matches!(c, '"' | '\'' | ',' | '&' | '}' | ')'))
                .unwrap_or(text.len() - separator_end);
            if has_separator && value_len > 0 {
                out.push_str(&text[i..separator_end]);
                out.push_str("<redacted>");
                i = separator_end + value_len;
            } else {
                out.push_str(&text[i..key_end]);
                i = key_end;
            }
        } else {
            let ch = text[i..].chars().next().expect("index is on a char boundary");
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    out
}

pub fn redact(text: &str) -> String {
    redact_secrets(&redact_user_paths(text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_the_windows_user_name_in_either_slash_style() {
        assert_eq!(
            redact_user_paths(r"open C:\Users\Alice\AppData\x failed"),
            r"open C:\Users\<user>\AppData\x failed"
        );
        assert_eq!(redact_user_paths("c:/users/Bob/file"), "c:/users/<user>/file");
        assert_eq!(redact_user_paths(r"C:\Users\Alice"), r"C:\Users\<user>");
    }

    #[test]
    fn redacts_unix_home_directories() {
        assert_eq!(redact_user_paths("open /home/alice/.local/share/x"), "open /home/<user>/.local/share/x");
        assert_eq!(redact_user_paths("/Users/Alice/Library/x"), "/Users/<user>/Library/x");
        assert_eq!(redact_user_paths("path=\"/home/alice\""), "path=\"/home/<user>\"");
    }

    #[test]
    fn a_home_segment_inside_a_url_is_left_alone() {
        assert_eq!(redact_user_paths("https://example.com/home/page"), "https://example.com/home/page");
        assert_eq!(redact_user_paths("src/home/mod.rs"), "src/home/mod.rs");
    }

    #[test]
    fn redaction_leaves_other_text_alone() {
        assert_eq!(redact_user_paths("no paths here"), "no paths here");
        assert_eq!(redact_user_paths(r"D:\Games\Steam"), r"D:\Games\Steam");
    }

    #[test]
    fn masks_secret_values_in_common_shapes() {
        assert_eq!(redact_secrets("ingest token=abc123 sent"), "ingest token=<redacted> sent");
        assert_eq!(redact_secrets("Authorization: Bearer abc.def"), "Authorization: Bearer <redacted>");
        assert_eq!(redact_secrets(r#"{"sessionToken": "abc", "n": 1}"#), r#"{"sessionToken": "<redacted>", "n": 1}"#);
        assert_eq!(redact_secrets("GET /x?api_key=k1&y=2"), "GET /x?api_key=<redacted>&y=2");
    }

    #[test]
    fn plain_mentions_of_a_secret_word_are_kept() {
        assert_eq!(redact_secrets("token refreshed"), "token refreshed");
        assert_eq!(redact_secrets("3 tokens left"), "3 tokens left");
    }
}
