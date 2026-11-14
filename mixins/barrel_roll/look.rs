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
) -> std::ops::ControlFlow<()> {
    use crate::game::barrel_state as state;
    if !game.native_on(crate::game::barrel_roll::ID) {
        return std::ops::ControlFlow::Continue(());
    }
    let gliding = game.screen == crate::game::Screen::Playing
        && game
            .player
            .as_ref()
            .is_some_and(|p| p.fall_flying);
    if !gliding {
        return std::ops::ControlFlow::Continue(());
    }
    let s = crate::game::barrel_roll::Settings::read(|key| {
        game.native_setting(crate::game::barrel_roll::ID, key)
    });
    let zoom = game.zoom_turn_scale();
    if !s.mouse_roll {
        // The sideways turn yaws vanilla (the body below runs): the banking
        // still sees it through the tick's yaw rate.
        return std::ops::ControlFlow::Continue(());
    }
    state::add_mouse_roll(crate::game::barrel_roll::mouse_roll(x * zoom, &s), true);
    game.turn_player(0.0, y * zoom);
    std::ops::ControlFlow::Break(())
}
