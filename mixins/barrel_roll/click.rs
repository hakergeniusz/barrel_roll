// A click on the mod's settings page: the row's change goes straight into
// the setting file the gates re-read, so it lands within the tick that
// follows the click.
// Copyright (c) 2026 hakergeniusz. MIT.
fn barrel_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    use crate::game::barrel_roll as barrel;
    if mod_id != barrel::ID {
        return;
    }
    let clicked = {
        let game = &*game;
        let s = barrel::Settings::read(|key| game.native_setting(barrel::ID, key));
        barrel::page_click(button, &s)
    };
    if let Some((key, value)) = clicked {
        game.native_set_setting(barrel::ID, key, &value);
    }
}
