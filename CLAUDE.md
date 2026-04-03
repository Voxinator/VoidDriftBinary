# Void Drift Binary

## Overview
Standalone Tauri v2 desktop app — a Canvas 2D space shooter with LAN multiplayer. Rust backend replaces Express/Socket.io relay. Single-file game client.

## Architecture
- **Tauri v2** desktop app with Rust backend
- **Frontend**: `src/index.html` — complete game client (~5,500 lines)
- **Backend**: `src-tauri/src/lib.rs` — axum + tokio-tungstenite WebSocket relay on port 3800
- **Sounds**: bundled as Tauri resources in `src-tauri/sounds/`
- **LAN multiplayer**: host runs in Tauri app, guests connect via browser at `http://<ip>:3800/`
- **Dev mode**: Rust server reads `src-tauri/index.html` directly (not from target/debug) for live iteration

## Key Commands
- `cargo tauri dev` — development mode (hot reload on file changes)
- `cargo tauri build` — production build (.app/.dmg on macOS)

## Version Management
Three files must stay in sync:
- `src/index.html` — `var GAME_VERSION = 'X.Y.Z';`
- `src-tauri/tauri.conf.json` — `"version": "X.Y.Z"`
- `src-tauri/Cargo.toml` — `version = "X.Y.Z"`

After editing `src/index.html`, copy to `src-tauri/index.html` before building.

## Build Workflow

### Local dev build
```bash
cp src/index.html src-tauri/index.html
cargo tauri build
```

### Cross-platform release (macOS + Windows)
A GitHub Actions workflow (`.github/workflows/build.yml`) builds both platforms automatically. It uses `tauri-apps/tauri-action` with a matrix strategy: macOS universal binary (ARM + Intel) and Windows x86_64.

**To cut a release:**
1. Update version in all three files (see Version Management above)
2. Copy frontend: `cp src/index.html src-tauri/index.html`
3. Commit and push to `main`
4. Tag and push: `git tag vX.Y.Z && git push origin main --tags`
5. GitHub Actions builds both platforms and creates a **draft release** with installers attached
6. Review and publish the draft at https://github.com/Voxinator/VoidDriftBinary/releases

The workflow can also be triggered manually from the Actions tab (workflow_dispatch).

**No Windows machine is needed** — GitHub Actions builds the Windows `.msi`/`.exe` on a Windows runner. All development and releases can be done from the Mac.

## Port
Uses port 3800 for WebSocket relay and static file serving. Port 3737 reserved for Project Home.

## Controls
- **Mouse** — move ship (auto-fire at nearest enemy)
- **WASD** — alternative movement
- **Left-click / Space** — drift (decouple movement from aiming)
- **P** — pause
- **Shift+P** — performance overlay
- **Backtick** — debug mode
- **1-4** (debug) — set player level
- **5** (debug) — god mode

## 10-Level Progression
Leveling is XP-based (kills fill XP bar). Each level grants a new ability. XP gain is modified by combo multiplier and drift bonus. Concentric rings around the ship show current level + XP progress.

### XP Thresholds
| Level | XP Required | Cumulative |
|-------|------------|------------|
| 1 -> 2 | 30 | 30 |
| 2 -> 3 | 50 | 80 |
| 3 -> 4 | 80 | 160 |
| 4 -> 5 | 110 | 270 |
| 5 -> 6 | 150 | 420 |
| 6 -> 7 | 200 | 620 |
| 7 -> 8 | 260 | 880 |
| 8 -> 9 | 330 | 1210 |
| 9 -> 10 | 420 | 1630 |

### Level Abilities

**Level 1 — Scout** (700ms fire rate, lerp 0.012)
Single cannon. One bullet stream, nimble movement. Learning phase.

**Level 2 — Gunner** (600ms, lerp 0.011)
Twin cannons. Two bullet streams alternating left/right with 6px spacing. Doubles effective DPS.

**Level 3 — Ace** (500ms, lerp 0.011)
Rapid fire. Fire rate drops significantly. Bullets travel 15% faster (9.2 px/frame vs 8). Twin streams become a wall of projectiles.

**Level 4 — Vanguard** (450ms, lerp 0.010)
Rear gun. A backward-firing cannon activates every 4th shot, targeting enemies behind the player within a 60-degree rear arc (300px range). Eliminates the blind spot. Drifting through enemies while the rear gun picks off pursuers becomes a core tactic.

**Level 5 — Striker** (400ms, lerp 0.010)
Piercing rounds. Bullets pass through the first enemy hit and continue to a second target at reduced effectiveness. Rewards positioning shots through enemy clusters. Shield cap reached (5 max) — survival now depends on pickup management.

**Level 6 — Predator** (360ms, lerp 0.009)
Homing rounds. Bullets gently curve toward the nearest enemy within 60px of their flight path (max turn rate ~4.5 deg/frame). Fewer missed shots. Combined with piercing, a homing bullet that pierces its first target and curves into a second is devastating.

**Level 7 — Warden** (320ms, lerp 0.009)
Shield pulse. When a shield breaks from damage, emits a 60px knockback wave that pushes enemies away and deals 1 damage. Taking a hit becomes a tactical choice — wade into a cluster, let them break a shield, and the pulse clears space. Combined with drift's faster shield recharge, enables aggressive play.

**Level 8 — Tempest** (280ms, lerp 0.008)
Overdrive. After 10 consecutive seconds without taking damage, fire rate doubles for 5 seconds (280ms -> 140ms). Ship glows white when active. Creates a dodge-dodge-dodge-UNLEASH rhythm. Taking any damage resets the timer. High skill expression — clean play is rewarded exponentially.

**Level 9 — Sovereign** (260ms, lerp 0.008)
Wingmen. Two autonomous drone ships orbit the player at 50px radius, each firing independently at nearby enemies (500ms fire rate, 300px range). The player becomes a squadron. Between twin homing piercing cannons, rear gun, and two wingmen, enemies melt. Drones are invulnerable but disappear on player death.

**Level 10 — Void Walker** (240ms, lerp 0.008)
Void drift. The drift mechanic transforms. While drifting at L10, the player becomes semi-transparent (40% opacity), all enemy projectiles pass through them (phase shift), and their bullets deal 2x damage. 3-second cooldown after exiting drift prevents chain-drifting. The ultimate risk/reward tool — phase through a carrier's torpedo barrage, shred it with 2x piercing homing rounds, exit drift, survive the cooldown, then drift again.

## Core Systems

### Combo Multiplier
Kills within 4 seconds build a combo counter. Drifting extends the timer to 6 seconds. The combo multiplier applies to XP gain:

| Combo Count | Multiplier |
|-------------|-----------|
| 0-2 | 1x |
| 3-5 | 1.5x |
| 6-9 | 2x |
| 10-14 | 3x |
| 15-19 | 4x |
| 20-29 | 5x |
| 30+ | 6x |

Drift bonus: additional 1.5x multiplier to XP while drifting. Stacks with combo.

Combo display in HUD shows current multiplier, blinks when about to expire. Best combo tracked per life for death screen.

### XP Sources
- Enemy kill: 10 XP
- Corvette kill: 20 XP
- Elite kill: 25 XP
- Carrier kill: 100 XP
- All modified by combo multiplier and drift bonus

### Time-Based Difficulty
Enemy shields, count, and spawn rates scale with minutes elapsed, not player level:

| Minute | Shield Chance | Max Enemies | Carriers | Barrages |
|--------|-------------|-------------|----------|----------|
| 0-1 | 0% | 8 | 0 | No |
| 1-2 | 10% | 10 | 0 | No |
| 2-3 | 20% | 12 | 0 | No |
| 3-5 | 30% | 14 | 1 | No |
| 5-7 | 40% | 16 | 1 | Small |
| 7-9 | 50% | 18 | 1-2 | Medium |
| 9-12 | 60% | 20 | 2 | Large |
| 12-15 | 70% | 22 | 2 | Full |
| 15+ | 80% | 24 | 2-3 | Continuous |

Corvettes begin spawning at 2 minutes (15% chance, rising to 35% at 9min+).

### Movement
Movement gets tighter (more responsive) at higher levels — power fantasy, not punishment:

| Level | Lerp | Feel |
|-------|------|------|
| 1 | 0.012 | Nimble, slightly twitchy |
| 2-3 | 0.011 | Smooth |
| 4-5 | 0.010 | Precise |
| 6-7 | 0.009 | Responsive |
| 8-10 | 0.008 | Tight, surgical |

### HUD Elements
- **XP rings**: concentric circles around ship (dim = completed levels, bright arc = current level filling)
- **Combo display**: bottom-right, multiplier + count, color-coded, blinks on expiry
- **Level name**: bottom-left, e.g. "L5 Striker"
- **Score popups**: floating +XP numbers per kill, color by combo tier
- **Death screen**: level, score, best combo, kills, time survived
- **Overdrive glow**: white pulsing aura when active (L8+)
- **Version**: top-left corner, small text

## Enemy Types
- **Saucer** — laser weapon, hover oscillation
- **Fighter** — torpedo burst weapon
- **Corvette** (2min+) — large, tanky (4HP/4 shields), alternates torpedo + laser, warp-in spawn, breaks into chunks on death
- **Carrier** (3min+) — boss, spawns elite enemies, torpedo barrages, multi-phase death sequence
- **Elite** — carrier-spawned, pursuit AI, extra HP/shields
