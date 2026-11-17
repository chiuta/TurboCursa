# Contributing to TURBOCURSA

Thanks for helping. TURBOCURSA is a clean-room project: **all code and art must be original**.

## Ground rules

- **Clean-room.** Do not copy code, sprites, sounds, track data or ROM-derived material from other games or emulators (including Cannonball, whose core is non-commercial). Do not use third-party trademarks in code, UI or docs.
- **Licence.** Contributions are accepted under the MIT licence of this repository.
- **Tests.** Run `./check.sh` (about 9 minutes) before opening a pull request; CI runs the same checks.
- **Languages.** UI strings exist in 8 languages (en, ro, fr, it, es, pt, de, hu). A new string needs all 8.
- **Dependencies.** Keep the web build dependency-free and the native build small; propose new crates in an issue first. Update `THIRD_PARTY_LICENSES.md` if dependencies change.

## Where help is most useful

1. **Playtesting**: play 15 minutes and file the "Playtest report" issue (pace, difficulty, anything boring, anything unfair).
2. **Sound**: listen on headphones and speakers and report levels or annoying sounds.
3. **Old Linux**: run the glibc 2.35 / 2.31 builds on real Ubuntu 20.04/22.04 or Debian 11 and report the result.
4. **Hand-made tracks**, **art** (portraits, vehicles), **translations review**.

Issues labelled `good first issue` are small and self-contained.
