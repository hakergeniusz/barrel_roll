// The mouse's look, while the elytra carries the player: the sideways turn
// feeds the roll instead of the yaw (the rudder steers), the vertical turn
// keeps being the pitch. The zoom scale still applies, and the vanilla body
// runs untouched when the player is not gliding, the mod is off, or the
// Mouse X setting says the sideways turn yaws vanilla.
// Copyright (c) 2026 hakergeniusz. MIT.
fn barrel_look(
    game: &mut crate::game::Game<'_>,
    x: f64,
    y: f64,
) -> core::ops::ControlFlow<()> {
    // Whether the page's Enabled toggle is on: a missing setting means on;
    // only `false` turns it off.
    fn enabled(game: &crate::game::Game<'_>) -> bool {
        game
            .stored_setting(barrel_roll::ID, "enabled")
            .is_none_or(|value| value.trim() != "false")
    }
    if !enabled(game) {
        return core::ops::ControlFlow::Continue(());
    }
    let gliding = game.playing()
        && game
            .player
            .as_ref()
            .is_some_and(|p| p.fall_flying);
    if !gliding {
        return core::ops::ControlFlow::Continue(());
    }
    let s = barrel_roll::Settings::read(|key| game.stored_setting(barrel_roll::ID, key));
    let zoom = game.zoom_turn_scale();
    if !s.mouse_roll {
        // The sideways turn yaws vanilla (the body below runs): the banking
        // still sees it through the tick's yaw rate.
        return core::ops::ControlFlow::Continue(());
    }
    barrel_roll::state::add_mouse_roll(barrel_roll::mouse_roll(x * zoom, &s), true);
    game.turn_player(0.0, y * zoom);
    core::ops::ControlFlow::Break(())
}
