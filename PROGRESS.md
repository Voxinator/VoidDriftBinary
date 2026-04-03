# Progress — 10-Level Progression System

## Current Status
v0.3.0 — All phases complete and verified.

## Completed

### Phase 1: Core Systems
- [x] XP-based leveling (replace shield-collection leveling)
- [x] Combo multiplier system (4s timer, 6s while drifting, 1.5x-6x)
- [x] Reverse movement floatiness (tighter at higher levels)
- [x] Time-based difficulty scaling (enemy shields/count scale with minutes)

### Phase 2: Abilities L1-5
- [x] L2: Twin cannons (2 bullet streams, alternating)
- [x] L3: Rapid fire + 15% faster bullets
- [x] L4: Rear gun (fires backward every 4th shot)
- [x] L5: Piercing rounds (pass through 1 enemy)

### Phase 3: Abilities L6-10
- [x] L6: Homing rounds (bullets curve toward enemies within 60px)
- [x] L7: Shield pulse (60px AoE knockback + 1 damage on shield hit)
- [x] L8: Overdrive (10s no-hit = 5s doubled fire rate)
- [x] L9: Wingmen (2 orbiting drone ships, independent targeting)
- [x] L10: Void drift (phase through bullets + 2x damage, 3s cooldown)

### Phase 4: Polish
- [x] Level-up celebration (ring waves, screen flash, ability name flash)
- [x] Combo UI (multiplier display, blink on expiry)
- [x] Score popups (floating +XP numbers per kill)
- [x] Death screen (level, score, best combo, kills, time)
- [x] XP bar under ship (pulses when near full)
- [x] Overdrive glow indicator
- [x] Level name + number in HUD

## Verification
- Judge agent verified 14/15 systems PASS, 1 minor fix applied (voidDrifting field init)
- Production build successful: Void Drift_0.3.0_aarch64.dmg

## Decisions Made
- XP replaces shield-collection for leveling; shields remain as survival resource only
- Difficulty scales with time (minutes elapsed), abilities scale with player level
- Movement gets tighter (not floatier) at higher levels
- Combo timer: 4s base, 6s while drifting
- Single-ship rendering (abilities expressed through weapons, not formation count)
- Carrier/corvette spawns gated by BOTH time and player level
