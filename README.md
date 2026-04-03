```
 __      __     _      _   ____         _   __  _   
 \ \    / /    (_)    | | |  _ \       (_) / _|| |  
  \ \  / /__    _   __| | | | | | _ __  _ | |_ | |_ 
   \ \/ // _ \ | | / _` | | | | || '__|| ||  _|| __|
    \  /| (_) || || (_| | | |_| || |   | || |  | |_ 
     \/  \___/ |_| \__,_| |____/ |_|   |_||_|   \__|
```

**A space shooter where momentum is your weapon and everything explodes.**

**[Download for macOS / Windows](https://github.com/Voxinator/VoidDriftBinary/releases)**

---

## What is this?

Void Drift is a standalone desktop space shooter with LAN multiplayer for up to 4 players. Fly through a procedural WebGL plasma nebula, fight alien fleets, unlock 10 levels of escalating abilities, and chase high scores through a combo system that rewards aggressive play.

The signature mechanic is the **drift** -- hold left-click to decouple your movement from your aim, sliding through space while firing in any direction. At higher levels, drifting transforms from a movement tool into an offensive weapon.

## Download

Pre-built binaries for macOS and Windows are available on the [Releases page](https://github.com/Voxinator/VoidDriftBinary/releases).

- **macOS**: `.dmg` installer (Apple Silicon + Intel)
- **Windows**: `.msi` installer

> macOS users: if Gatekeeper blocks the app ("damaged" warning), run `xattr -cr /Applications/Void\ Drift.app` in Terminal.

## Features

### 10-Level Progression

Level up by killing enemies. Each level grants a new ability that changes how you play:

| Lvl | Name | Ability |
|-----|------|---------|
| 1 | Scout | Single cannon, nimble movement |
| 2 | Gunner | **Twin cannons** -- two alternating bullet streams |
| 3 | Ace | **Rapid fire** -- faster fire rate, 15% faster bullets |
| 4 | Vanguard | **Rear gun** -- fires backward at pursuing enemies |
| 5 | Striker | **Piercing rounds** -- bullets pass through to a second target |
| 6 | Predator | **Homing rounds** -- bullets curve toward nearby enemies |
| 7 | Warden | **Shield pulse** -- AoE knockback + damage when your shield breaks |
| 8 | Tempest | **Overdrive** -- 10s no-hit streak doubles fire rate for 5s |
| 9 | Sovereign | **Wingmen** -- two autonomous drone ships orbit and fire independently |
| 10 | Void Walker | **Void drift** -- phase through bullets + 2x damage while drifting |

### Combo System

Chain kills within 4 seconds to build a combo multiplier (up to 6x). Drifting extends the timer to 6 seconds. High combos mean faster leveling and bigger scores. The multiplier blinks when about to expire -- one more kill to keep the chain alive.

### The Drift

Hold left-click (or Space with WASD) to drift. Your ship captures its current velocity and coasts -- decouple movement from aiming, slide through enemy formations, bounce off walls. Drifting also:
- Extends combo timer (4s -> 6s)
- Doubles shield recharge rate
- At Level 10: grants bullet immunity and 2x damage (with 3s cooldown)

### Enemy Types

- **Saucers** -- laser weapons, hover oscillation
- **Fighters** -- torpedo burst attacks
- **Corvettes** -- large warships that warp in with 4 shields, alternating torpedo + laser
- **Carriers** -- mothership bosses that spawn elite enemies and fire bullet-hell torpedo barrages
- **Elites** -- carrier-spawned pursuers with extra HP, shields, and aggressive AI

Corvettes break into burning debris chunks on death. Carriers have a 6-second multi-phase death sequence with escalating internal explosions, hull breakup, and a cataclysmic final detonation.

### Multiplayer

LAN multiplayer for up to 4 players. The app runs a WebSocket server -- other players on the same network connect via browser at `http://<your-ip>:3800`. Host-client model with automatic host migration.

### Visuals

- **WebGL plasma nebula** -- procedural simplex noise background with ship repulsion and explosion displacement
- **250-star parallax starfield** -- stars streak into speed lines based on ship velocity
- **2000-particle system** -- massive explosions, engine trails, drift effects, fireworks
- **CRT post-processing** -- scanlines, vignette, neon glow (CSS, zero perf cost)
- **New high score celebration** -- fireworks and fighter flybys

## Controls

| Input | Action |
|-------|--------|
| Mouse | Move ship (auto-fires at nearest target) |
| WASD | Alternative movement |
| Left-click / Space | Drift |
| P | Pause |
| Shift+P | Performance overlay |

## Tech

| | |
|---|---|
| **Desktop** | Tauri v2 (Rust + WebView) |
| **Backend** | axum + tokio-tungstenite |
| **Rendering** | WebGL (plasma) + Canvas 2D (starfield, game) |
| **Audio** | Web Audio API with spatial stereo panning |
| **Binary size** | ~35 MB (mostly audio assets) |

## Building from Source

Requires [Rust](https://rustup.rs/) and the Tauri CLI:

```bash
cargo install tauri-cli --version "^2"
git clone https://github.com/Voxinator/VoidDriftBinary.git
cd VoidDriftBinary
cp src/index.html src-tauri/index.html
cargo tauri build
```

Output: `src-tauri/target/release/bundle/`

## Origin Story

This game started as a background decoration for a localhost dashboard. A WebGL plasma nebula, then twinkling stars, then a little ship that followed your cursor. Then the ship needed something to shoot at. Two days later the "background decoration" had a boss fight, a particle system, and LAN multiplayer.

It got its own name, its own repo, and eventually a [standalone desktop app](https://github.com/Voxinator/VoidDriftBinary) with a 10-level progression system, a combo engine, and enough particle effects to stress-test a GPU. Scope creep wins when you lean into it.

The browser version lives at [Voxinator/VoidDrift](https://github.com/Voxinator/VoidDrift).

---

A game by Taylor Creative, Copyright 2026. Built by [Voxinator](https://github.com/Voxinator).
