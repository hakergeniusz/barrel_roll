# Barrel Roll

A Horizon native mod that unlocks the camera in elytra flight: the mouse's
sideways turn **rolls** (barrel rolls included), the strafe keys **steer**
the yaw, the vertical turn stays the pitch, and the camera **banks** into
its turns. The port of enjarai's [Do a Barrel Roll](https://codeberg.org/enjarai/do-a-barrel-roll)
idea to Horizon — an independent clean-room reimplementation (no original
code; MIT).

While the elytra carries you:

- **Mouse X** → roll (a swipe right rolls the wings clockwise)
- **Mouse Y** → pitch, as always
- **A / D** → yaw (the rudder; the vanilla elytra ignores strafe)
- the roll **banks** into the turn your rudder makes, and settles level
  the moment you land

The camera pose is the game's own bob (hurt tilt, view bobbing, shake) with
the roll read on top, so the ground game never changes. The flight physics
are untouched — the server sees an ordinary elytra flight steered with A/D.

## Settings

Options → Mods → Barrel Roll, seven rows: **Enabled**, **Mouse X**
(roll or turn), **Roll sensitivity** (50–300%), **Rudder speed**
(30–120°/s), **Bank strength** (50–200%), **Camera roll** (off for motion
sickness; the model still runs) and **Invert mouse roll**. The values live
under `horizon-mods/.horizon-mod-data/barrel_roll/` in the game's data dir.

## How it works

- A per-tick hook steers the rudder through the game's own `turn_player`
  (so ropes, the boat clamp and every other look side effect stay vanilla)
  and integrates the roll: the banking chase while gliding, the level
  settle on the ground.
- A look hook reroutes the mouse's sideways turn into the roll while
  gliding; the vertical turn passes through untouched.
- A frame hook rebuilds the game's own bob matrix from the same public
  camera helpers and adds the roll as a Z rotation — between ticks it
  interpolates by the frame's partial tick.

The brain (`src/lib.rs`) is pure standard library — the roll model and the
setting files' reader, with its own tests; the helpers (`src/mixins/`) are
the thin copies the launcher stages into the game's tick, look and camera
functions, wired by `horizon-mod.toml`.

## Building

`cargo test` runs the brain's tests (the `.horizon/sdk/api` symlink points
them at the pinned SDK's API crate). The launcher compiles the `.hmod`
against the pinned Horizon revision on install.

## License

MIT — see LICENSE. The upstream idea is enjarai's Do a Barrel Roll; this
implementation is independent and contains none of its code.
