// Barrel Roll's brain: pure standard library, no game types, so it compiles
// and tests wherever — as this crate's library and, injected, as a module of
// the game. The roll model: while the elytra carries the player, the mouse's
// sideways turn feeds the camera's roll instead of the yaw (the strafe keys
// steer the yaw), the roll banks into the turn the yaw rate makes, and
// everything settles back to level on the ground. The camera and the world
// read a partial-tick interpolation of the roll. Every constant it runs on
// is a setting, read fresh from the files each hook.
//
// Clean-room: written for Horizon from the game's own public behavior; it
// copies nothing from any mod. The idea is enjarai's Do a Barrel Roll.
//
// Copyright (c) 2026 hakergeniusz. MIT.

/// The mod's registry id: the Options > Mods row and the injection gates.
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

/// The version and description the Options > Mods row shows.
pub const VERSION: &str = "0.2.0";
pub const DESCRIPTION: &str =
    "Barrel rolls for the elytra: the mouse rolls, the strafe keys steer, the camera banks.";

/// Everything the model runs on, read from the setting files each hook so
/// the page's clicks land within the tick that follows.
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

/// A number setting file: missing or unparsable keeps `default`, clamped to
/// `min`..`max` so a typo can't spin the camera into a drill.
pub fn setting_num(
    get: &impl Fn(&str) -> Option<String>,
    key: &str,
    default: f64,
    min: f64,
    max: f64,
) -> f64 {
    get(key)
        .and_then(|v| v.trim().parse::<f64>().ok())
        .map_or(default, |v| v.clamp(min, max))
}

impl Settings {
    /// The state the setting files hold, read fresh each hook.
    pub fn read(get: impl Fn(&str) -> Option<String>) -> Self {
        let get = &get;
        Self {
            enabled: setting_bool(get, "enabled", true),
            mouse_roll: setting_bool(get, "mouse_roll", true),
            sensitivity: setting_num(get, "sensitivity", 1.0, 0.5, 3.0),
            rudder: setting_num(get, "rudder", STRAFE_TURN, 2.0, 60.0),
            bank: setting_num(get, "bank", 1.0, 0.5, 2.0),
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

// The settings page: seven rows, each cycling its own value. The rows read
// and store plain setting-file strings, so the gates and the page always
// agree on what the files hold.

/// The settings page's rows, in the state the files hold.
pub fn page_labels(s: &Settings) -> Vec<String> {
    let sens = |v: f64| {
        if (v - 1.0).abs() < f64::EPSILON {
            "100%".into()
        } else {
            format!("{}%", (v * 100.0).round() as i64)
        }
    };
    let rudder = format!("{}°/s", (s.rudder * TURN_UNIT_DEGREES * 20.0).round() as i64);
    vec![
        format!("Enabled: {}", on_off(s.enabled)),
        format!("Mouse X: {}", if s.mouse_roll { "roll" } else { "turn" }),
        format!("Roll sensitivity: {}", sens(s.sensitivity)),
        format!("Rudder speed: {rudder}"),
        format!("Bank strength: {}", sens(s.bank)),
        format!("Camera roll: {}", on_off(s.camera)),
        format!("Invert mouse roll: {}", on_off(s.invert)),
    ]
}

fn on_off(on: bool) -> &'static str {
    if on {
        "on"
    } else {
        "off"
    }
}

/// The next value `value` takes when the page cycles it through `steps`:
/// the first step bigger than the current one, else back to the first.
fn cycle(value: f64, steps: &[f64]) -> f64 {
    let next = steps.iter().copied().find(|s| *s > value + 1.0e-9);
    next.unwrap_or(steps[0])
}

/// What a click on the page's row `i` stores: `(key, value)` for the rows
/// that change, None for a row that is not ours.
pub fn page_click(i: usize, s: &Settings) -> Option<(&'static str, String)> {
    match i {
        0 => Some(("enabled", (!s.enabled).to_string())),
        1 => Some(("mouse_roll", (!s.mouse_roll).to_string())),
        2 => Some(("sensitivity", cycle(s.sensitivity, &SENSITIVITY_STEPS).to_string())),
        3 => Some(("rudder", cycle(s.rudder, &RUDDER_STEPS).to_string())),
        4 => Some(("bank", cycle(s.bank, &BANK_STEPS).to_string())),
        5 => Some(("camera", (!s.camera).to_string())),
        6 => Some(("invert", (!s.invert).to_string())),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        banking, camera_degrees, cycle, mouse_roll, page_click, page_labels, rendered,
        setting_bool, setting_num, Settings, MAX_ROLL, RUDDER_STEPS, SENSITIVITY_STEPS,
        STRAFE_TURN, TURN_UNIT_DEGREES,
    };

    fn get(key: &str) -> Option<String> {
        match key {
            "enabled" => Some("false".into()),
            "mouse_roll" => Some("false".into()),
            "sensitivity" => Some("1.5".into()),
            "rudder" => Some("30".into()),
            "bank" => Some("2".into()),
            "camera" => Some("false".into()),
            "invert" => Some("true".into()),
            _ => None,
        }
    }

    #[test]
    fn the_settings_read_and_clamp() {
        let s = Settings::read(get);
        assert!(!s.enabled);
        assert!(!s.mouse_roll);
        assert!((s.sensitivity - 1.5).abs() < 1e-9);
        assert!((s.rudder - 30.0).abs() < 1e-9);
        assert!((s.bank - 2.0).abs() < 1e-9);
        assert!(!s.camera);
        assert!(s.invert);
        let d = Settings::default();
        assert!(d.enabled && d.mouse_roll && d.camera && !d.invert);
        assert!((d.rudder - STRAFE_TURN).abs() < 1e-9);
        // A missing file means the default; a typo means the default; a
        // wild number means the clamp.
        assert!(setting_bool(&|_| None, "x", true));
        assert!(!setting_bool(&|k| get(k).filter(|v| v == "weird"), "x", false));
        assert!((setting_num(&|_| Some("900".into()), "x", 1.0, 0.5, 3.0) - 3.0).abs() < 1e-9);
        assert!((setting_num(&|_| Some("potato".into()), "x", 1.0, 0.5, 3.0) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn the_rudder_is_a_sane_speed() {
        // The old default spun 9°/tick (180°/s); 20 units a tick is ~60°/s.
        let per_tick = STRAFE_TURN * TURN_UNIT_DEGREES;
        assert!((per_tick - 3.0).abs() < 0.1, "{per_tick}°/tick");
        assert!(banking(1.2, 1.0) < 0.0, "a right turn banks the right wing down");
        assert!(banking(-1.2, 1.0) > 0.0);
        assert_eq!(banking(0.0, 1.0), 0.0);
        assert!((banking(1.2, 2.0) - banking(1.2, 1.0) * 2.0).abs() < 1e-9);
        assert!(banking(1000.0, 1.0).abs() <= 0.9, "the bank clamps at a wing-over");
    }

    #[test]
    fn the_roll_chases_the_bank_while_gliding() {
        let mut roll = 0.0;
        let bank = banking(10.0, 1.0);
        for _ in 0..60 {
            roll = super::step(roll, 0.0, bank, true);
        }
        assert!((roll - bank).abs() < 0.01, "a held turn banks fully: {roll} vs {bank}");
        let mut roll = 2.0;
        for _ in 0..120 {
            roll = super::step(roll, 0.0, 0.5, false);
        }
        assert!(roll.abs() < 0.01, "grounded roll settles: {roll}");
        let mut roll = 0.0;
        for _ in 0..100_000 {
            roll = super::step(roll, 120.0, 0.0, true);
        }
        assert!(roll.abs() <= MAX_ROLL, "the roll clamps: {roll}");
        assert!(roll > 0.0, "a rightward swipe rolls right: {roll}");
    }

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

    #[test]
    fn the_page_labels_and_clicks_agree() {
        let mut s = Settings::default();
        let labels = page_labels(&s);
        assert_eq!(labels.len(), 7, "seven rows");
        assert!(labels[0].starts_with("Enabled: on"));
        assert!(labels[1].contains("roll"));
        assert!(labels[3].contains("60°/s"), "{}", labels[3]);
        for i in 0..7usize {
            let Some((key, value)) = page_click(i, &s) else {
                panic!("row {i} must answer");
            };
            let stored = |k: &str| (k == key).then(|| value.clone());
            let after = Settings::read(stored);
            match i {
                0 => assert_eq!(after.enabled, false),
                1 => assert_eq!(after.mouse_roll, false),
                2 => assert!((after.sensitivity - SENSITIVITY_STEPS[2]).abs() < 1e-9),
                3 => assert!((after.rudder - RUDDER_STEPS[2]).abs() < 1e-9),
                4 => assert!((after.bank - 1.5).abs() < 1e-9),
                5 => assert_eq!(after.camera, false),
                _ => assert!(after.invert),
            }
            s = after;
        }
        assert!(page_click(7, &s).is_none(), "past the rows is not ours");
        // Cycling wraps: 3.0 sensitivity goes back to 50%.
        s.sensitivity = 3.0;
        let (key, value) = page_click(2, &s).unwrap();
        assert_eq!(key, "sensitivity");
        assert_eq!(value, "0.5");
    }

    #[test]
    fn the_render_interpolates_between_ticks() {
        assert_eq!(rendered(0.0, 1.0, 0.0), 0.0);
        assert_eq!(rendered(0.0, 1.0, 1.0), 1.0);
        assert!((rendered(0.0, 1.0, 0.5) - 0.5).abs() < 1.0e-9);
        let degrees = camera_degrees(1.0);
        assert!(degrees < 0.0, "positive roll reads as a negative Z rotation");
        let _ = cycle(0.5, &SENSITIVITY_STEPS);
    }
}
