//! Barrel Roll's brain: pure standard library, no game types, so it compiles
//! and tests wherever — this crate's library, read by the game's copied
//! helpers. The roll model: while the elytra carries the player, the mouse's
//! sideways turn feeds the camera's roll instead of the yaw (the strafe keys
//! steer the yaw), the roll banks into the turn the yaw rate makes, and
//! everything settles back to level on the ground. The camera and the world
//! read a partial-tick interpolation of the roll. Every constant it runs on
//! is a setting, read fresh from the game's settings store each hook.
//!
//! Clean-room: written for Horizon from the game's own public behavior; it
//! copies nothing from any mod. The idea is enjarai's Do a Barrel Roll.
//!
//! Copyright (c) 2026 hakergeniusz. MIT.

use mod_api::Hooks;

/// The mod's registry id: the settings page and the gates the helpers run.
pub const ID: &str = "barrel_roll";

/// Radians of roll per mouse turn unit at 100% sensitivity: a full
/// across-the-screen swipe (~120 turn units) rolls about a quarter circle.
pub const ROLL_PER_TURN: f64 = 0.012;
/// Degrees one turn unit yaw turns (`Entity.turn` units, ~0.15°/unit): the
/// rudder speed setting reads in these units per tick — 20 a tick is
/// ~3.0°/tick, ~60°/s of steady turn.
pub const TURN_UNIT_DEGREES: f64 = 0.15;
/// The default rudder, in turn units a tick (≈60°/s).
pub const STRAFE_TURN: f64 = 20.0;
/// Radians of banking per degree per tick of yaw rate at 100%: banking rolls
/// into the turn the rudder makes.
pub const BANK_PER_DEGREE: f64 = 0.02;
/// The roll chases the banking target this far each gliding tick.
pub const BANK_LERP: f64 = 0.12;
/// Off the elytra, the roll settles level this far each tick.
pub const SETTLE_LERP: f64 = 0.25;
/// The roll never winds past a full circle either way.
pub const MAX_ROLL: f64 = std::f64::consts::PI;
/// The camera's roll sign: positive roll (a swipe right) rolls the wings
/// clockwise seen from behind, which the bob matrix reads as a negative
/// Z rotation.
pub const CAMERA_SIGN: f32 = -1.0;

/// The sensitivity steps the settings page cycles (fractions of 1.0).
pub const SENSITIVITY_STEPS: [f64; 5] = [0.5, 1.0, 1.5, 2.0, 3.0];
/// The rudder steps the settings page cycles, in turn units a tick.
pub const RUDDER_STEPS: [f64; 4] = [10.0, 20.0, 30.0, 40.0];
/// The bank steps the settings page cycles (fractions of 1.0).
pub const BANK_STEPS: [f64; 4] = [0.5, 1.0, 1.5, 2.0];

// The settings page's rows, as the game draws them: one setting a row, each
// choice's option a translation key the `assets/barrel_roll/lang` file
// answers. The choice steps pair with these keys one to one, in cycle order.

/// The Mouse X row's options: index 0 rolls the sideways turn, 1 yaws it.
pub const MOUSE_X_OPTIONS: [&str; 2] = ["barrel_roll.mouse_x.roll", "barrel_roll.mouse_x.turn"];
/// The Roll sensitivity row's options, for [`SENSITIVITY_STEPS`].
pub const SENSITIVITY_OPTIONS: [&str; 5] = [
    "barrel_roll.sens.50",
    "barrel_roll.sens.100",
    "barrel_roll.sens.150",
    "barrel_roll.sens.200",
    "barrel_roll.sens.300",
];
/// The Rudder speed row's options, for [`RUDDER_STEPS`]: 10..40 turn units
/// a tick, 30..120 degrees a second.
pub const RUDDER_OPTIONS: [&str; 4] = [
    "barrel_roll.rudder.30",
    "barrel_roll.rudder.60",
    "barrel_roll.rudder.90",
    "barrel_roll.rudder.120",
];
/// The Bank strength row's options, for [`BANK_STEPS`] — the sensitivity
/// steps' own names.
pub const BANK_OPTIONS: [&str; 4] = [
    "barrel_roll.sens.50",
    "barrel_roll.sens.100",
    "barrel_roll.sens.150",
    "barrel_roll.sens.200",
];

/// A boolean setting file: missing (or anything but the exact word) keeps
/// `default`, so a hand-edited file can never wed the mod.
pub fn setting_bool(get: &impl Fn(&str) -> Option<String>, key: &str, default: bool) -> bool {
    match get(key) {
        None => default,
        Some(v) if v.trim() == "true" => true,
        Some(v) if v.trim() == "false" => false,
        Some(_) => default,
    }
}

/// The index a choice's stored file holds: a decimal index within `len`
/// reads itself; missing, unparsable or out of range reads `default` — the
/// same rule the game's own reader runs, so the helpers and the page always
/// agree on what a file holds.
pub fn setting_index(
    get: &impl Fn(&str) -> Option<String>,
    key: &str,
    len: usize,
    default: usize,
) -> usize {
    get(key)
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|i| *i < len)
        .unwrap_or(default)
}

/// Everything the model runs on, read from the game's settings store each
/// hook so the page's clicks land within the tick that follows. The toggles
/// store `true`/`false`, the choices the picked option's index as a decimal
/// string — exactly what the game's store keeps.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    /// The mod answers at all.
    pub enabled: bool,
    /// true: the mouse's sideways turn rolls (DABR's way); false: it yaws
    /// vanilla and only the rudder, banking and camera roll run.
    pub mouse_roll: bool,
    /// The mouse roll's multiplier (0.5..3.0).
    pub sensitivity: f64,
    /// The rudder's yaw, in turn units a tick while a strafe key holds.
    pub rudder: f64,
    /// The banking's multiplier (0.5..2.0).
    pub bank: f64,
    /// Whether the camera shows the roll at all (turn it off for motion
    /// sickness; the model still runs).
    pub camera: bool,
    /// A rightward swipe rolls the other way.
    pub invert: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: true,
            mouse_roll: true,
            sensitivity: 1.0,
            rudder: STRAFE_TURN,
            bank: 1.0,
            camera: true,
            invert: false,
        }
    }
}

impl Settings {
    /// The state the stored settings hold, read fresh each hook.
    pub fn read(get: impl Fn(&str) -> Option<String>) -> Self {
        let get = &get;
        Self {
            enabled: setting_bool(get, "enabled", true),
            mouse_roll: setting_index(get, "mouse_roll", MOUSE_X_OPTIONS.len(), 0) == 0,
            sensitivity: SENSITIVITY_STEPS
                [setting_index(get, "sensitivity", SENSITIVITY_STEPS.len(), 1)],
            rudder: RUDDER_STEPS[setting_index(get, "rudder", RUDDER_STEPS.len(), 1)],
            bank: BANK_STEPS[setting_index(get, "bank", BANK_STEPS.len(), 1)],
            camera: setting_bool(get, "camera", true),
            invert: setting_bool(get, "invert", false),
        }
    }
}

/// The banking target for this tick's yaw rate (degrees per tick, positive
/// turning right): banking rolls into the turn — right turn, right wing down.
pub fn banking(yaw_rate_degrees: f64, strength: f64) -> f64 {
    (-yaw_rate_degrees * BANK_PER_DEGREE * strength).clamp(-0.9, 0.9)
}

/// The strafe's rudder turn for this tick, at the setting's speed: A turns
/// left (+), D right (-).
pub fn strafe_turn(left: bool, right: bool, speed: f64) -> f64 {
    match (left, right) {
        (true, false) => speed,
        (false, true) => -speed,
        _ => 0.0,
    }
}

/// The mouse's sideways turn, scaled and signed per the settings: the value
/// `step` adds straight in (a frame amount).
pub fn mouse_roll(x: f64, s: &Settings) -> f64 {
    let x = if s.invert { -x } else { x };
    x * s.sensitivity
}

/// One tick of the roll: the mouse's sideways turn (a frame amount, added
/// straight in), then the chase toward the banking target while gliding —
/// or toward level when the elytra is off. Wraps past a half circle.
pub fn step(roll: f64, mouse_x: f64, bank: f64, gliding: bool) -> f64 {
    let mut roll = roll + mouse_x * ROLL_PER_TURN;
    let target = if gliding { bank } else { 0.0 };
    let lerp = if gliding { BANK_LERP } else { SETTLE_LERP };
    roll += (target - roll) * lerp;
    roll.clamp(-MAX_ROLL, MAX_ROLL)
}

/// The frame's rendered roll between the last two ticks.
pub fn rendered(prev: f64, roll: f64, partial: f32) -> f64 {
    prev + (roll - prev) * f64::from(partial)
}

/// The rotation the bob matrix reads for a roll: `CAMERA_SIGN` times the
/// roll, in degrees for `Mat4::rotate_z`.
pub fn camera_degrees(roll: f64) -> f32 {
    (roll as f32).to_degrees() * CAMERA_SIGN
}

/// The roll's shared snapshot between the tick, look and bob helpers: one
/// mutex-guarded flight, so a reader never blocks a writer beyond a copy.
pub mod state {
    use super::rendered as interpolate;
    use super::{banking, step};
    use std::sync::Mutex;

    /// One flight's roll: the last two integrated angles (the render lerps
    /// between them), the yaw the last tick saw (the banking's rate), and
    /// whether the elytra carried the player as of that tick.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Flight {
        pub prev_roll: f64,
        pub roll: f64,
        pub prev_yaw: f32,
        pub gliding: bool,
    }

    impl Default for Flight {
        fn default() -> Self {
            Self {
                prev_roll: 0.0,
                roll: 0.0,
                prev_yaw: 0.0,
                gliding: false,
            }
        }
    }

    static FLIGHT: Mutex<Flight> = Mutex::new(Flight {
        prev_roll: 0.0,
        roll: 0.0,
        prev_yaw: 0.0,
        gliding: false,
    });

    /// The flight as of the last tick.
    pub fn held() -> Flight {
        FLIGHT.lock().map(|f| *f).unwrap_or_default()
    }

    /// The mouse's sideways turn (a frame amount in turn units) rolls at once.
    pub fn add_mouse_roll(turn: f64, gliding: bool) {
        if let Ok(mut f) = FLIGHT.lock() {
            let was = f.roll;
            f.roll = step(f.roll, turn, 0.0, gliding);
            f.prev_roll = was;
            f.gliding = gliding;
        }
    }

    /// One tick's integration: the banking chase (and the level settle off the
    /// elytra) from `yaw`, the player's current yaw in degrees, at the bank
    /// strength the settings hold.
    pub fn integrate(yaw: f32, gliding: bool, bank_strength: f64) {
        if let Ok(mut f) = FLIGHT.lock() {
            let rate = f64::from(yaw - f.prev_yaw);
            let bank = banking(rate, bank_strength);
            let was = f.roll;
            f.roll = step(f.roll, 0.0, bank, gliding);
            f.prev_roll = was;
            f.prev_yaw = yaw;
            f.gliding = gliding;
        }
    }

    /// Everything level: the mod toggled off (or the world went away).
    pub fn reset() {
        if let Ok(mut f) = FLIGHT.lock() {
            *f = Flight::default();
        }
    }

    /// The frame's rendered roll, in radians — None when the last tick was not
    /// a glide (the vanilla bob runs untouched).
    pub fn rendered(partial: f32) -> Option<f64> {
        let f = held();
        f.gliding.then(|| interpolate(f.prev_roll, f.roll, partial))
    }
}

#[derive(Default)]
pub struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// The mod's page in Options > Mods: seven rows, one a setting — the
    /// Enabled toggle, the Mouse X roll-or-turn choice, the roll
    /// sensitivity, the rudder speed, the bank strength, the camera roll's
    /// toggle and the invert toggle. The game draws and cycles them; the
    /// helpers read the stored files fresh each hook.
    fn register(&self, registrar: &mut dyn mod_api::Registrar, _: &mod_api::RegistrationContext) {
        use mod_api::SettingSpec;
        let _ = registrar.register_setting(SettingSpec::toggle(
            "enabled",
            "barrel_roll.setting.enabled",
            true,
        ));
        let _ = registrar.register_setting(SettingSpec::choice(
            "mouse_roll",
            "barrel_roll.setting.mouse_x",
            MOUSE_X_OPTIONS,
            0,
        ));
        let _ = registrar.register_setting(SettingSpec::choice(
            "sensitivity",
            "barrel_roll.setting.sensitivity",
            SENSITIVITY_OPTIONS,
            1,
        ));
        let _ = registrar.register_setting(SettingSpec::choice(
            "rudder",
            "barrel_roll.setting.rudder",
            RUDDER_OPTIONS,
            1,
        ));
        let _ = registrar.register_setting(SettingSpec::choice(
            "bank",
            "barrel_roll.setting.bank",
            BANK_OPTIONS,
            1,
        ));
        let _ = registrar.register_setting(SettingSpec::toggle(
            "camera",
            "barrel_roll.setting.camera",
            true,
        ));
        let _ = registrar.register_setting(SettingSpec::toggle(
            "invert",
            "barrel_roll.setting.invert",
            false,
        ));
    }

    /// Implemented as a no-op so the game always calls the
    /// `native_client_tick_pre` dispatch the tick mixin lives in: the game
    /// skips dispatches no mod listens to, and the mixin must run.
    fn client_tick_pre(
        &self,
        _: &mut dyn mod_api::ClientHost,
        _: &mut mod_api::ClientTickPreEvent,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BANK_OPTIONS, BANK_STEPS, MAX_ROLL, MOUSE_X_OPTIONS, RUDDER_OPTIONS, RUDDER_STEPS,
        SENSITIVITY_OPTIONS, SENSITIVITY_STEPS, STRAFE_TURN, Settings, TURN_UNIT_DEGREES, banking,
        camera_degrees, mouse_roll, rendered, setting_bool, setting_index, state as barrel_state,
        step,
    };

    /// The stored files the test reads: the toggles as `true`/`false`, the
    /// choices as the picked option's index, exactly the game's form —
    /// with the Mouse X on turn, the sensitivity at 150%, the rudder at
    /// 30 units and the bank at 200%.
    fn get(key: &str) -> Option<String> {
        match key {
            "enabled" => Some("false".into()),
            "mouse_roll" => Some("1".into()),
            "sensitivity" => Some("2".into()),
            "rudder" => Some("2".into()),
            "bank" => Some("3".into()),
            "camera" => Some("false".into()),
            "invert" => Some("true".into()),
            _ => None,
        }
    }

    /// The settings read the stored files: the toggles by the exact word,
    /// the choices by their index — and anything else (a missing file, a
    /// typo, a wild index) reads the default, so a hand-edited file can
    /// never wed the mod.
    #[test]
    fn the_settings_read_the_stored_files() {
        let s = Settings::read(get);
        assert!(!s.enabled);
        assert!(!s.mouse_roll, "index 1 is the turn");
        assert!((s.sensitivity - 1.5).abs() < 1e-9);
        assert!((s.rudder - 30.0).abs() < 1e-9);
        assert!((s.bank - 2.0).abs() < 1e-9);
        assert!(!s.camera);
        assert!(s.invert);
        let d = Settings::default();
        assert!(d.enabled && d.mouse_roll && d.camera && !d.invert);
        assert!((d.rudder - STRAFE_TURN).abs() < 1e-9);
        assert!((d.sensitivity - 1.0).abs() < 1e-9);
        assert!((d.bank - 1.0).abs() < 1e-9);
        // A missing file means the default; a typo means the default; an
        // out-of-range index means the default.
        assert!(setting_bool(&|_| None, "x", true));
        assert!(!setting_bool(
            &|k| get(k).filter(|v| v == "weird"),
            "x",
            false
        ));
        assert_eq!(setting_index(&|_| None, "x", 5, 2), 2);
        assert_eq!(setting_index(&|_| Some("potato".into()), "x", 5, 2), 2);
        assert_eq!(setting_index(&|_| Some("9".into()), "x", 5, 2), 2);
        assert_eq!(setting_index(&|_| Some(" 3 ".into()), "x", 5, 2), 3);
        assert!((Settings::read(|_| None).sensitivity - 1.0).abs() < 1e-9);
    }

    /// The brain's reader agrees with the game's own: every index the page
    /// can store (`SettingSpec::read`'s answer) lands on the step the model
    /// runs, and the choice/step arrays pair up one to one.
    #[test]
    fn the_choices_read_as_the_game_reads_them() {
        use mod_api::{SettingSpec, SettingValue};
        assert_eq!(MOUSE_X_OPTIONS.len(), 2);
        assert_eq!(SENSITIVITY_OPTIONS.len(), SENSITIVITY_STEPS.len());
        assert_eq!(RUDDER_OPTIONS.len(), RUDDER_STEPS.len());
        assert_eq!(BANK_OPTIONS.len(), BANK_STEPS.len());
        // The Mouse X choice: what the page stores for each option reads
        // back as the behavior the row names.
        let spec = SettingSpec::choice("k", "l", MOUSE_X_OPTIONS, 0);
        let stored = |index: usize| spec.store(SettingValue::Choice(index)).unwrap();
        let rolls = |text: Option<String>| {
            Settings::read(|k| k.eq("mouse_roll").then(|| text.clone()).flatten()).mouse_roll
        };
        assert!(rolls(Some(stored(0))), "index 0 rolls");
        assert!(!rolls(Some(stored(1))), "index 1 turns");
        assert!(rolls(None), "missing reads the default (roll)");
        assert!(rolls(Some("garbage".into())), "garbage reads the default");
        assert!(rolls(Some("7".into())), "out of range reads the default");
        // The sensitivity choice: each stored index lands on its step.
        let spec = SettingSpec::choice("k", "l", SENSITIVITY_OPTIONS, 1);
        let at = |text: &str| {
            Settings::read(|k| (k == "sensitivity").then(|| text.to_owned())).sensitivity
        };
        for (index, step) in SENSITIVITY_STEPS.iter().enumerate() {
            let stored = spec.store(SettingValue::Choice(index)).unwrap();
            assert!((at(&stored) - step).abs() < 1e-9, "index {index}");
        }
        // An out-of-range or missing file reads the default (100%).
        assert!((at("9") - 1.0).abs() < 1e-9);
        assert!((Settings::read(|_| None).rudder - 20.0).abs() < 1e-9);
    }

    /// The rudder is a sane speed, and banking rolls into the turn: a right
    /// turn banks the right wing down, doubled strength doubles the bank,
    /// and the bank clamps at a wing-over.
    #[test]
    fn the_rudder_is_a_sane_speed() {
        // The old default spun 9°/tick (180°/s); 20 units a tick is ~60°/s.
        let per_tick = STRAFE_TURN * TURN_UNIT_DEGREES;
        assert!((per_tick - 3.0).abs() < 0.1, "{per_tick}°/tick");
        assert!(
            banking(1.2, 1.0) < 0.0,
            "a right turn banks the right wing down"
        );
        assert!(banking(-1.2, 1.0) > 0.0);
        assert_eq!(banking(0.0, 1.0), 0.0);
        assert!((banking(1.2, 2.0) - banking(1.2, 1.0) * 2.0).abs() < 1e-9);
        assert!(
            banking(1000.0, 1.0).abs() <= 0.9,
            "the bank clamps at a wing-over"
        );
    }

    /// The roll chases the bank while gliding, settles level on the ground,
    /// clamps at a half circle, and a rightward swipe rolls right.
    #[test]
    fn the_roll_chases_the_bank_while_gliding() {
        let mut roll = 0.0;
        let bank = banking(10.0, 1.0);
        for _ in 0..60 {
            roll = step(roll, 0.0, bank, true);
        }
        assert!(
            (roll - bank).abs() < 0.01,
            "a held turn banks fully: {roll} vs {bank}"
        );
        let mut roll = 2.0;
        for _ in 0..120 {
            roll = step(roll, 0.0, 0.5, false);
        }
        assert!(roll.abs() < 0.01, "grounded roll settles: {roll}");
        let mut roll = 0.0;
        for _ in 0..100_000 {
            roll = step(roll, 120.0, 0.0, true);
        }
        assert!(roll.abs() <= MAX_ROLL, "the roll clamps: {roll}");
        assert!(roll > 0.0, "a rightward swipe rolls right: {roll}");
    }

    /// The mouse roll scales by the sensitivity and inverts.
    #[test]
    fn the_mouse_roll_scales_and_inverts() {
        let mut s = Settings::default();
        assert!((mouse_roll(10.0, &s) - 10.0).abs() < 1e-9);
        s.invert = true;
        assert!((mouse_roll(10.0, &s) + 10.0).abs() < 1e-9);
        s.invert = false;
        s.sensitivity = 2.0;
        assert!((mouse_roll(10.0, &s) - 20.0).abs() < 1e-9);
    }

    /// The render interpolates between ticks, and the camera reads a
    /// positive roll as a negative Z rotation.
    #[test]
    fn the_render_interpolates_between_ticks() {
        assert_eq!(rendered(0.0, 1.0, 0.0), 0.0);
        assert_eq!(rendered(0.0, 1.0, 1.0), 1.0);
        assert!((rendered(0.0, 1.0, 0.5) - 0.5).abs() < 1.0e-9);
        let degrees = camera_degrees(1.0);
        assert!(
            degrees < 0.0,
            "positive roll reads as a negative Z rotation"
        );
    }

    /// The shared snapshot: the mouse's turn rolls at once, a tick's
    /// integration banks from the yaw rate, and off the elytra the frame
    /// reads none of it — the vanilla bob runs — until reset, and a reset
    /// starts the next flight level.
    #[test]
    fn the_state_round_trips_between_the_helpers() {
        barrel_state::reset();
        assert!(!barrel_state::held().gliding, "level out of the box");
        assert_eq!(barrel_state::rendered(0.5), None, "no glide, no roll");
        // The look helper's mouse turn rolls at once, mid-glide.
        barrel_state::add_mouse_roll(120.0, true);
        let held = barrel_state::held();
        assert!(held.gliding);
        assert!(held.roll > 0.0, "a rightward swipe rolled: {}", held.roll);
        assert!(
            barrel_state::rendered(1.0).unwrap() > 0.0,
            "the frame's end of the lerp is the roll"
        );
        // The tick helper's integration: a rightward yaw rate banks, and
        // the yaw is remembered for the next tick's rate.
        barrel_state::integrate(held.prev_yaw + 10.0, true, 1.0);
        let after = barrel_state::held();
        assert_eq!(after.prev_yaw, held.prev_yaw + 10.0, "the yaw is kept");
        // Off the elytra the frame reads none of it.
        barrel_state::integrate(after.prev_yaw, false, 1.0);
        assert_eq!(barrel_state::rendered(0.5), None);
        // Reset: everything level again.
        barrel_state::reset();
        assert_eq!(barrel_state::held(), barrel_state::Flight::default());
    }
}
