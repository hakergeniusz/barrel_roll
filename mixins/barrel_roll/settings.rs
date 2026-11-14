// The settings page: the seven rows, in the state the files hold (read
// fresh from disk, so the page always shows what the gates act on).
// Copyright (c) 2026 hakergeniusz. MIT.
fn barrel_page(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    use crate::game::barrel_roll as barrel;
    if mod_id != barrel::ID {
        return;
    }
    let game = &*game;
    let s = barrel::Settings::read(|key| game.native_setting(barrel::ID, key));
    buttons.extend(barrel::page_labels(&s));
}
