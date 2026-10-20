# NEXUS Sort — generated PNG asset pack

The game now has code-side texture paths for the principal screens, gems, boosters, and match effects. **The PNG binaries from the generated ZIP are not yet stored in this GitHub directory**; this README documents the exact filenames expected from the pack. Until the files are uploaded, the game intentionally falls back to its procedural drawings.

## Screen art
- `menu_bg.png` — main menu background
- `map_bg.png` — adventure map background
- `gameplay_mockup.png` — gameplay background fallback
- `main_menu_mockup.png` — main-menu reference mockup
- `cavern_background.png` — crystal cavern art
- `level_complete.png`, `level_failed.png` — end-state panels
- `settings_panel.png`, `panel_blue.png`, `panel_wide.png`, `dialog_frame.png`

## Gems and obstacles
- `gem_red.png`, `gem_blue.png`, `gem_green.png`, `gem_purple.png`, `gem_yellow.png`, `gem_pink.png`
- `gem_ice.png`, `gem_stripe.png`, `gem_vstripe.png`, `gem_rainbow.png`, `gem_bomb.png`
- `gem_rock.png`, `gem_iceblock.png`, `gem_chain.png`, `gem_wood.png`, `gem_void.png`

## Boosters and buttons
- `booster_hammer.png`, `booster_bomb.png`, `booster_shuffle.png`, `booster_rainbow.png`, `booster_plus5.png`
- `btn_play.png`, `btn_level.png`, `btn_settings.png`, `btn_home.png`, `btn_close.png`, `btn_ok.png`
- `coin.png`, `nexus_crystal.png`, `chest.png`, `key.png`

## Effects and character
- `effect_match_pink.png`, `effect_match_blue.png`, `effect_blast.png`, `effect_bubble.png`, `effect_line.png`
- `effect_swirl.png`, `effect_shatter.png`, `effect_glow.png`, `effect_sparkle.png`, `effect_beam.png`
- `pipo_idle.png`, `pipo_happy.png`, `pipo_sad.png`, `pipo_surprised.png`, `pipo_sleep.png`, `pipo_angry.png`

## Integration status
- `src/bin/mobile.rs` loads the principal available art paths and displays optional blast/shatter and sparkle overlays during successful matches; procedural particles are the fallback.
- The source-generated ZIP contains approximate crops from a concept atlas; inspect/correct crop boundaries before treating them as final transparent sprites.
- Binary upload of the generated PNG files remains a separate step. A green CI build alone does not mean these images are bundled into the APK.
