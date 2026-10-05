use dp_discord_ipc::{Activity, Button, Party};
use dp_presence::Presence;

pub fn to_activity(p: &Presence, session: &str) -> Activity {
    Activity {
        details: p.details.clone(),
        state: p.state.clone(),
        start: p.start_timestamp,
        large_image: p.large_image.clone(),
        large_text: p.large_text.clone(),
        small_image: p.small_image.clone(),
        small_text: p.small_text.clone(),
        buttons: p.buttons.iter().map(|b| Button { label: b.label.clone(), url: b.url.clone() }).collect(),
        party: p.party.map(|(size, max)| Party { id: session.to_owned(), size, max }),
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
            buttons: vec![dp_presence::Button { label: "b".into(), url: "u".into() }],
            party: Some((3, 6)),
        };
        assert_eq!(
            to_activity(&p, "session"),
            Activity {
                details: Some("d".into()),
                state: Some("s".into()),
                start: Some(42),
                large_image: Some("li".into()),
                large_text: Some("lt".into()),
                small_image: Some("si".into()),
                small_text: Some("st".into()),
                buttons: vec![dp_discord_ipc::Button { label: "b".into(), url: "u".into() }],
                party: Some(dp_discord_ipc::Party { id: "session".into(), size: 3, max: 6 }),
            }
        );
    }

    #[test]
    fn empty_presence_gives_empty_activity() {
        assert_eq!(to_activity(&Presence::default(), "session"), Activity::default());
    }
}
