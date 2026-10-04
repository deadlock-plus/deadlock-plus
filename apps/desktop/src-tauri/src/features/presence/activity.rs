use dp_discord_ipc::Activity;
use dp_presence::Presence;

pub fn to_activity(p: &Presence) -> Activity {
    Activity {
        details: p.details.clone(),
        state: p.state.clone(),
        start: p.start_timestamp,
        large_image: p.large_image.clone(),
        large_text: p.large_text.clone(),
        small_image: p.small_image.clone(),
        small_text: p.small_text.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carries_every_field_and_renames_the_timestamp() {
        let p = Presence {
            details: Some("d".into()),
            state: Some("s".into()),
            start_timestamp: Some(42),
            large_image: Some("li".into()),
            large_text: Some("lt".into()),
            small_image: Some("si".into()),
            small_text: Some("st".into()),
        };
        assert_eq!(
            to_activity(&p),
            Activity {
                details: Some("d".into()),
                state: Some("s".into()),
                start: Some(42),
                large_image: Some("li".into()),
                large_text: Some("lt".into()),
                small_image: Some("si".into()),
                small_text: Some("st".into()),
            }
        );
    }

    #[test]
    fn empty_presence_gives_empty_activity() {
        assert_eq!(to_activity(&Presence::default()), Activity::default());
    }
}
