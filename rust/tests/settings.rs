//! Toolbar, friend sound and color settings.

use openase::settings::{
    ColorSettings, FriendSound, ServerRef, Settings, TOOLBAR_DEFAULT, ToolbarItem, ToolbarSettings,
};

#[test]
fn defaults_show_every_button_in_order() {
    let toolbar = ToolbarSettings::default();
    assert_eq!(toolbar.visible_ids(), TOOLBAR_DEFAULT);
    let colors = ColorSettings::default();
    assert_eq!(colors.list_bg, [0xff; 3]);
    assert_eq!(colors.selection, [0x31, 0x6a, 0xc5]);
    assert!(colors.quake_colors);
    let settings = Settings::default();
    assert_eq!(settings.colors, colors);
    assert!(settings.friend_sounds.is_empty());
    assert_eq!(settings.friend_sound("Anyone"), FriendSound::Default);
}

#[test]
fn old_settings_files_load_with_defaults() {
    let settings: Settings =
        serde_json::from_str(r#"{"friends":["Bob"],"wizard_done":true}"#).expect("old file loads");
    assert_eq!(settings.friends, ["Bob"]);
    assert_eq!(settings.toolbar, ToolbarSettings::default());
    assert_eq!(settings.colors, ColorSettings::default());
    assert!(settings.friend_sounds.is_empty());
}

#[test]
fn partial_color_object_keeps_other_defaults() {
    let settings: Settings =
        serde_json::from_str(r#"{"colors":{"selection":[1,2,3]}}"#).expect("loads");
    assert_eq!(settings.colors.selection, [1, 2, 3]);
    assert_eq!(settings.colors.list_bg, ColorSettings::default().list_bg);
    assert!(settings.colors.quake_colors);
}

#[test]
fn round_trip_keeps_every_new_field() {
    let mut settings = Settings::default();
    settings.toolbar.items[0].visible = false;
    settings.toolbar.move_item(1, 1);
    settings.colors.list_bg = [10, 20, 30];
    settings.colors.quake_colors = false;
    settings
        .friend_sounds
        .insert("bob".into(), FriendSound::File("C:\\a\\b.wav".into()));
    settings
        .friend_sounds
        .insert("eve".into(), FriendSound::Silent);
    let text = serde_json::to_string_pretty(&settings).unwrap();
    let back: Settings = serde_json::from_str(&text).unwrap();
    assert_eq!(back.toolbar, settings.toolbar);
    assert_eq!(back.colors, settings.colors);
    assert_eq!(back.friend_sounds, settings.friend_sounds);
    assert_eq!(
        back.friend_sound("BOB"),
        FriendSound::File("C:\\a\\b.wav".into())
    );
    assert_eq!(back.friend_sound("eve"), FriendSound::Silent);
}

#[test]
fn toolbar_normalize_drops_unknown_and_restores_missing() {
    let mut toolbar = ToolbarSettings {
        items: vec![
            ToolbarItem {
                id: "options".into(),
                visible: true,
            },
            ToolbarItem {
                id: "bogus".into(),
                visible: true,
            },
            ToolbarItem {
                id: "options".into(),
                visible: false,
            },
            ToolbarItem {
                id: "refresh".into(),
                visible: false,
            },
        ],
    };
    toolbar.normalize();
    assert_eq!(toolbar.items.len(), TOOLBAR_DEFAULT.len());
    assert_eq!(toolbar.items[0].id, "options");
    assert!(toolbar.items[0].visible);
    assert!(
        toolbar
            .items
            .iter()
            .all(|item| TOOLBAR_DEFAULT.contains(&item.id.as_str()))
    );
    assert!(!toolbar.visible_ids().contains(&"refresh"));
}

#[test]
fn toolbar_move_item_stays_in_bounds() {
    let mut toolbar = ToolbarSettings::default();
    toolbar.move_item(0, -1);
    toolbar.move_item(TOOLBAR_DEFAULT.len() - 1, 1);
    assert_eq!(toolbar, ToolbarSettings::default());
    toolbar.move_item(0, 1);
    assert_eq!(toolbar.items[0].id, "stop");
    assert_eq!(toolbar.items[1].id, "refresh");
}

#[test]
fn import_export_normalizes_the_toolbar() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("profile.json");
    std::fs::write(
        &path,
        r#"{"toolbar":{"items":[{"id":"find","visible":true}]}}"#,
    )
    .unwrap();
    let settings = Settings::import_from(&path).unwrap();
    assert_eq!(settings.toolbar.items.len(), TOOLBAR_DEFAULT.len());
    assert_eq!(settings.toolbar.items[0].id, "find");
}

#[test]
fn replace_server_moves_every_list_and_the_group() {
    let old = ServerRef {
        game: "ut99".into(),
        address: "1.2.3.4:7777".into(),
    };
    let new = ServerRef {
        game: "ut99".into(),
        address: "1.2.3.4:7778".into(),
    };
    let mut settings = Settings::default();
    settings.add_favorite_to(old.clone(), "Favorites3");
    Settings::push_front(&mut settings.recent, old.clone());
    Settings::push_front(&mut settings.recently_added, old.clone());
    // Already present under the new address: no duplicate after the move.
    Settings::push_front(&mut settings.recently_imported, old.clone());
    Settings::push_front(&mut settings.recently_imported, new.clone());

    settings.replace_server(&old, &new);

    assert_eq!(settings.favorites, vec![new.clone()]);
    assert_eq!(settings.favorite_group(&new), "Favorites3");
    assert!(!settings.favorite_groups.contains_key(&old.key()));
    assert_eq!(settings.recent, vec![new.clone()]);
    assert_eq!(settings.recently_added, vec![new.clone()]);
    assert_eq!(settings.recently_imported, vec![new]);
}

#[test]
fn old_settings_have_not_checked_unreal_ports() {
    let settings: Settings = serde_json::from_str("{}").unwrap();
    assert!(!settings.unreal_ports_checked);
}

#[test]
fn ready_games_have_a_program_or_steam_with_a_key() {
    use openase::settings::GameSettings;
    let mut settings = Settings::default();
    assert!(settings.ready_games().is_empty());
    settings.games.insert(
        "q3".into(),
        GameSettings {
            program_location: "quake3.exe".into(),
            ..GameSettings::default()
        },
    );
    let ids = |s: &Settings| {
        s.ready_games()
            .iter()
            .map(|g| g.id.as_str())
            .collect::<Vec<_>>()
    };
    assert_eq!(ids(&settings), ["q3"]);
    // Steam games join through Steam: ready once there is an API key.
    settings.steam_api_key = "key".into();
    let with_key = ids(&settings);
    assert!(with_key.contains(&"q3") && with_key.contains(&"tf2") && with_key.contains(&"css"));
    assert!(!with_key.contains(&"ut99"));
}

#[test]
fn themes_are_presets_of_the_color_settings() {
    use openase::settings::Theme;
    let luma = |[r, g, b]: [u8; 3]| 299 * r as u32 + 587 * g as u32 + 114 * b as u32;
    // Classic is today's look, and the default.
    assert_eq!(Theme::Classic.colors(), ColorSettings::default());
    for theme in Theme::ALL {
        let colors = theme.colors();
        assert_eq!(colors.theme, theme);
        // Text stands out from the list in every theme.
        let (text, bg) = (luma(colors.text), luma(colors.list_bg));
        assert!(text.abs_diff(bg) > 100_000, "{theme:?}");
        assert_eq!(theme.dark(), text > bg, "{theme:?}");
    }
    // Old settings files without a theme load as Classic.
    let old: ColorSettings = serde_json::from_str(r#"{"list_bg":[255,255,255]}"#).unwrap();
    assert_eq!(old.theme, Theme::Classic);
}

#[test]
fn ping_levels_and_theme_colors() {
    use openase::settings::{Theme, ping_level};
    assert_eq!(ping_level(30), 0);
    assert_eq!(ping_level(80), 0);
    assert_eq!(ping_level(81), 1);
    assert_eq!(ping_level(150), 1);
    assert_eq!(ping_level(151), 2);
    for theme in Theme::ALL {
        let [good, mid, bad] = theme.ping_colors();
        assert!(good != mid && mid != bad && good != bad, "{theme:?}");
        assert_ne!(theme.occupancy(), theme.colors().list_bg, "{theme:?}");
    }
}

#[test]
fn upstream_settings_are_imported_once() {
    use openase::settings::import_upstream;
    let root = tempfile::tempdir().unwrap();
    let (old, new) = (
        root.path().join("OpenASE"),
        root.path().join("OpenASE-Plus"),
    );
    // Nothing to import.
    assert!(!import_upstream(&old, &new));
    std::fs::create_dir_all(&old).unwrap();
    std::fs::write(old.join("settings.json"), "{\"friends\":[\"Visor\"]}").unwrap();
    assert!(import_upstream(&old, &new));
    assert_eq!(
        std::fs::read_to_string(new.join("settings.json")).unwrap(),
        "{\"friends\":[\"Visor\"]}"
    );
    // Never over an existing OpenASE-Plus profile.
    std::fs::write(new.join("settings.json"), "{}").unwrap();
    assert!(!import_upstream(&old, &new));
    assert_eq!(
        std::fs::read_to_string(new.join("settings.json")).unwrap(),
        "{}"
    );
}
