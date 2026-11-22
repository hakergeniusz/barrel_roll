// Every game tick, on the window's game only: while the elytra carries the
// player, the strafe keys are the rudder (a yaw turn through the game's own
// `turn_player`, so ropes, the boat clamp and every other look side effect
// stay vanilla) and the roll banks into the resulting yaw rate; off the
// elytra, everything settles level. The mod toggled off resets the state.
// Copyright (c) 2026 hakergeniusz. MIT.
fn barrel_tick(game: &mut crate::game::Game<'_>) {
    // Whether the page's Enabled toggle is on: a missing setting means on;
    // only `false` turns it off.
    fn enabled(game: &crate::game::Game<'_>) -> bool {
        game
            .stored_setting(barrel_roll::ID, "enabled")
            .is_none_or(|value| value.trim() != "false")
    }
    if !enabled(game) {
        barrel_roll::state::reset();
        return;
    }
    if !game.native_window() {
        return;
    }
    let s = barrel_roll::Settings::read(|key| game.stored_setting(barrel_roll::ID, key));
    let gliding = game.playing()
        && game
            .player
            .as_ref()
            .is_some_and(|p| p.fall_flying);
    if !gliding {
        barrel_roll::state::integrate(game.yaw, false, s.bank);
        return;
    }
    // The rudder: the strafe keys turn the yaw through the game's own turn,
    // at the speed the settings hold.
    let turn = barrel_roll::strafe_turn(
        game.mapped_key_down("key.left"),
        game.mapped_key_down("key.right"),
        s.rudder,
    );
    if turn != 0.0 {
        game.turn_player(turn, 0.0);
    }
    barrel_roll::state::integrate(game.yaw, true, s.bank);
}
