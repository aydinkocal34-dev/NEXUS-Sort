use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GameState { MainMenu, Map, Playing, LevelComplete }

#[derive(Default)]
struct GameAssets {
    menu_bg: Option<Texture2D>, map_bg: Option<Texture2D>, board_bg: Option<Texture2D>, cavern_bg: Option<Texture2D>,
    gem_red: Option<Texture2D>, gem_blue: Option<Texture2D>, gem_green: Option<Texture2D>,
    gem_purple: Option<Texture2D>, gem_yellow: Option<Texture2D>, button: Option<Texture2D>,
    panel: Option<Texture2D>, level_complete: Option<Texture2D>, hammer: Option<Texture2D>,
    bomb: Option<Texture2D>, rainbow: Option<Texture2D>, shuffle: Option<Texture2D>,
    match_blast: Option<Texture2D>, bubble: Option<Texture2D>, sparkle: Option<Texture2D>,
    shatter: Option<Texture2D>, pipo_happy: Option<Texture2D>, pipo_sad: Option<Texture2D>,
    // Preload the remaining art pack so all committed PNG assets are available to the mobile build.
    extras: Vec<Texture2D>,
}
impl GameAssets {
    async fn load() -> Self {
        async fn optional(path: &str) -> Option<Texture2D> {
            match load_texture(path).await { Ok(t) => { t.set_filter(FilterMode::Linear); Some(t) }, Err(_) => None }
        }
        let assets = Self {
            menu_bg: optional("assets/main_menu_mockup.png").await.or(optional("assets/menu_bg.png").await),
            map_bg: optional("assets/map_bg.png").await,
            board_bg: optional("assets/cavern_background.png").await,
            cavern_bg: optional("assets/cavern_background.png").await, gem_red: optional("assets/gem_red.png").await,
            gem_blue: optional("assets/gem_blue.png").await, gem_green: optional("assets/gem_green.png").await,
            gem_purple: optional("assets/gem_purple.png").await, gem_yellow: optional("assets/gem_yellow.png").await,
            button: optional("assets/btn_play.png").await, panel: optional("assets/panel_blue.png").await,
            level_complete: optional("assets/level_complete.png").await, hammer: optional("assets/booster_hammer.png").await,
            bomb: optional("assets/booster_bomb.png").await, rainbow: optional("assets/booster_rainbow.png").await,
            shuffle: optional("assets/booster_shuffle.png").await,
            match_blast: optional("assets/effect_blast.png").await,
            bubble: optional("assets/effect_bubble.png").await,
            sparkle: optional("assets/effect_sparkle.png").await,
            shatter: optional("assets/effect_shatter.png").await,
            pipo_happy: optional("assets/pipo_happy.png").await,
            pipo_sad: optional("assets/pipo_sad.png").await,
            extras: Vec::new(),
        };
        let mut assets = assets;
        // Keep every supplied PNG path wired into the runtime asset loader. Individual screens
        // can adopt these textures incrementally without missing-file surprises.
        for path in [
            "assets/asset_atlas_reference.png", "assets/booster_plus5.png", "assets/btn_close.png",
            "assets/btn_home.png", "assets/btn_level.png", "assets/btn_ok.png", "assets/btn_settings.png",
            "assets/chest.png", "assets/coin.png", "assets/dialog_frame.png", "assets/effect_beam.png",
            "assets/effect_glow.png", "assets/effect_line.png", "assets/effect_match_blue.png",
            "assets/effect_match_pink.png", "assets/effect_swirl.png", "assets/gem_bomb.png",
            "assets/gem_chain.png", "assets/gem_ice.png", "assets/gem_iceblock.png", "assets/gem_rainbow.png",
            "assets/gem_rock.png", "assets/gem_stripe.png", "assets/gem_void.png", "assets/gem_wood.png",
            "assets/key.png", "assets/level_failed.png", "assets/main_menu_mockup.png", "assets/nexus_crystal.png",
            "assets/panel_wide.png", "assets/pipo_angry.png", "assets/pipo_idle.png", "assets/pipo_sleep.png",
            "assets/pipo_surprised.png", "assets/settings_panel.png",
        ] {
            if let Some(texture) = optional(path).await { assets.extras.push(texture); }
        }
        assets
    }
    fn gem(&self, c: GemColor) -> Option<&Texture2D> {
        match c { GemColor::Red=>self.gem_red.as_ref(), GemColor::Blue=>self.gem_blue.as_ref(),
        GemColor::Green=>self.gem_green.as_ref(), GemColor::Purple=>self.gem_purple.as_ref(),
        GemColor::Yellow=>self.gem_yellow.as_ref() }
    }
}
fn draw_cover(t: &Texture2D, w: f32, h: f32) {
    let scale=(w/t.width().max(1.0)).max(h/t.height().max(1.0));
    let size=vec2(t.width()*scale,t.height()*scale);
    draw_texture_ex(t,(w-size.x)*0.5,(h-size.y)*0.5,WHITE,DrawTextureParams{dest_size:Some(size),..Default::default()});
}
fn draw_panel_texture(t: Option<&Texture2D>, x:f32,y:f32,w:f32,h:f32,fallback:Color) {
    if let Some(t)=t { draw_texture_ex(t,x,y,WHITE,DrawTextureParams{dest_size:Some(vec2(w,h)),..Default::default()}); }
    else { draw_rectangle(x,y,w,h,fallback); }
}
use nexus_sort::game::board::Color as GemColor;
use nexus_sort::game::crystal_cavern::{CrystalCavernLevel, TARGET_SCORE};
use nexus_sort::game::lives::{ClockAnomaly, LevelOutcome, PlayerState};

fn load_player(now_ms: u64) -> PlayerState {
    // Storage is optional: a storage panic must not prevent the game from opening.
    let loaded = std::panic::catch_unwind(|| {
        quad_storage::STORAGE.lock().ok().map(|storage| {
            PlayerState {
                lives: storage.get("nexus_sort_lives").and_then(|v| v.parse::<u8>().ok()).unwrap_or(5),
                last_refill_at_ms: storage.get("nexus_sort_last_refill").and_then(|v| v.parse::<u64>().ok()).unwrap_or(now_ms),
                pending_ms: storage.get("nexus_sort_pending_ms").and_then(|v| v.parse::<u64>().ok()).unwrap_or(0),
                level_progress: storage.get("nexus_sort_level_progress").and_then(|v| v.parse::<u32>().ok()).unwrap_or(0),
            }
        })
    }).ok().flatten();
    let mut player = loaded.unwrap_or_else(|| PlayerState::new(now_ms));
    player.clamp_lives();
    player
}

fn save_player(player: &PlayerState) {
    // quad-storage 0.1.3 may panic while writing its local.data file on Android.
    // Keep optional persistence from crashing the game at startup or after a move.
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if let Ok(mut storage) = quad_storage::STORAGE.lock() {
            storage.set("nexus_sort_lives", &player.lives.to_string());
            storage.set("nexus_sort_last_refill", &player.last_refill_at_ms.to_string());
            storage.set("nexus_sort_pending_ms", &player.pending_ms.to_string());
            storage.set("nexus_sort_level_progress", &player.level_progress.to_string());
        }
    }));
}

fn window_conf() -> Conf {
    Conf {
        window_title: "NEXUS SORT - Kristal Magarasi".to_owned(),
        window_width: 450,
        window_height: 900,
        high_dpi: true,
        sample_count: 1,
        ..Default::default()
    }
}

fn gem_colors(color: GemColor) -> (Color, Color, Color) {
    match color {
        GemColor::Blue => (Color::new(0.0, 0.90, 1.0, 1.0), Color::new(0.02, 0.42, 0.95, 1.0), Color::new(0.55, 1.0, 1.0, 1.0)),
        GemColor::Purple => (Color::new(0.78, 0.20, 1.0, 1.0), Color::new(0.36, 0.06, 0.78, 1.0), Color::new(0.95, 0.68, 1.0, 1.0)),
        GemColor::Green => (Color::new(0.0, 1.0, 0.48, 1.0), Color::new(0.0, 0.48, 0.26, 1.0), Color::new(0.62, 1.0, 0.78, 1.0)),
        GemColor::Yellow => (Color::new(1.0, 0.82, 0.0, 1.0), Color::new(0.88, 0.40, 0.0, 1.0), Color::new(1.0, 0.97, 0.58, 1.0)),
        GemColor::Red => (Color::new(1.0, 0.18, 0.42, 1.0), Color::new(0.72, 0.02, 0.22, 1.0), Color::new(1.0, 0.66, 0.76, 1.0)),
    }
}

fn draw_gem(cx: f32, cy: f32, radius: f32, color: GemColor) {
    let (bright, dark, shine) = gem_colors(color);
    draw_poly(cx, cy, 6, radius * 1.28, 30.0, Color::new(bright.r, bright.g, bright.b, 0.12));
    draw_poly(cx, cy + 1.5, 6, radius, 30.0, dark);
    draw_poly(cx, cy, 6, radius * 0.82, 30.0, bright);
    draw_poly(cx - radius * 0.12, cy - radius * 0.12, 3, radius * 0.42, 20.0, shine);
    draw_circle(cx - radius * 0.25, cy - radius * 0.28, radius * 0.09, WHITE);
}

fn just_pressed() -> bool {
    is_mouse_button_pressed(MouseButton::Left)
        || touches().iter().any(|touch| touch.phase == TouchPhase::Started)
}

// Android touch coordinates must come from the touch event itself.
// mouse_position() can remain stale on devices when Macroquad does not
// synthesize mouse motion from touch input.
fn pointer_position() -> Vec2 {
    touches()
        .iter()
        .find(|touch| touch.phase == TouchPhase::Started)
        .map(|touch| touch.position)
        .unwrap_or_else(|| { let (x, y) = mouse_position(); vec2(x, y) })
}

fn draw_background(assets: &GameAssets, state: GameState) {
    clear_background(Color::new(0.025, 0.035, 0.105, 1.0));
    let bg = match state { GameState::MainMenu => assets.menu_bg.as_ref(), GameState::Map => assets.map_bg.as_ref(), GameState::Playing | GameState::LevelComplete => assets.cavern_bg.as_ref().or(assets.board_bg.as_ref()) };
    if let Some(t) = bg { draw_cover(t, screen_width(), screen_height()); }
    draw_circle(screen_width() * 0.12, screen_height() * 0.20, 190.0, Color::new(0.0, 0.65, 1.0, 0.055));
    draw_circle(screen_width() * 0.90, screen_height() * 0.48, 220.0, Color::new(0.65, 0.05, 1.0, 0.045));
    draw_circle(screen_width() * 0.50, screen_height() * 0.92, 190.0, Color::new(0.0, 1.0, 0.55, 0.035));
    for i in 0..34 {
        let x = ((i * 73 + 19) as f32 % screen_width().max(1.0));
        let y = ((i * 137 + 47) as f32 % screen_height().max(1.0));
        draw_circle(x, y, if i % 5 == 0 { 1.8 } else { 0.8 }, Color::new(0.45, 0.82, 1.0, 0.32));
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let assets = GameAssets::load().await;
    let mut game_state = GameState::MainMenu;
    let mut level = match CrystalCavernLevel::new() {
        Ok(level) => level,
        Err(error) => {
            loop {
                draw_background(&assets, game_state);
                draw_text("NEXUS SORT", 24.0, 70.0, 36.0, WHITE);
                draw_text(&format!("Level error: {error}"), 24.0, 120.0, 20.0, RED);
                next_frame().await;
            }
        }
    };
    let now_ms = || -> u64 { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis().min(u64::MAX as u128) as u64).unwrap_or(0) };
    let mut player = load_player(now_ms());
    let initial_refill = player.apply_refill(now_ms());
    if initial_refill.clock_anomaly == ClockAnomaly::None {
        save_player(&player);
    }
    let mut message = String::from("Birbirine bagli 3+ kristale dokun!");
    let mut message_timer = 4.0_f32;
    // Start at the designed main menu; the previous shadowed Map assignment skipped it.
    let mut completed_levels = player.level_progress;
    let mut level_finished = false;
    let mut moves_left = 25_u32;
    let mut outcome_applied = false;
    let mut timed_mode = true;
    let mut time_left = 90.0_f32;
    let mut level_started = false;
    let mut stars = 0_u8;
    let mut burst_timer = 0.0_f32;
    let mut burst_center = vec2(0.0, 0.0);

    loop {
        let w = screen_width();
        let h = screen_height();
        burst_timer = (burst_timer - get_frame_time()).max(0.0);
        draw_background(&assets, game_state);

        let refill = player.apply_refill(now_ms());
        if refill.lives_after != refill.lives_before {
            save_player(&player);
        }
        completed_levels = player.level_progress;

        if game_state == GameState::MainMenu {
            let w=screen_width(); let h=screen_height();
            draw_text("NEXUS SORT",w*0.5-105.0,h*0.25,38.0,Color::new(0.0,0.88,1.0,1.0));
            draw_text("KRISTAL MAGARASI",w*0.5-98.0,h*0.25+34.0,17.0,WHITE);
            let bw=(w*0.72).min(330.0); let bh=62.0; let bx=(w-bw)*0.5; let py=h*0.53; let my=py+bh+20.0;
            draw_panel_texture(assets.button.as_ref(),bx,py,bw,bh,Color::new(0.02,0.48,0.30,0.98));
            draw_text("OYNA",bx+bw*0.5-33.0,py+39.0,27.0,WHITE);
            draw_panel_texture(assets.button.as_ref(),bx,my,bw,bh,Color::new(0.04,0.22,0.45,0.98));
            draw_text("MACERA HARITASI",bx+bw*0.5-94.0,my+38.0,21.0,WHITE);
            if just_pressed(){let p=pointer_position();if p.x>=bx&&p.x<=bx+bw&&p.y>=py&&p.y<=py+bh{game_state=GameState::Playing;}else if p.x>=bx&&p.x<=bx+bw&&p.y>=my&&p.y<=my+bh{game_state=GameState::Map;}}
            next_frame().await; continue;
        }

        if game_state == GameState::Map {
            draw_text("NEXUS", 24.0, 52.0, 30.0, Color::new(0.0, 0.88, 1.0, 1.0));
            draw_text("MACERA HARITASI", 24.0, 84.0, 19.0, WHITE);
            draw_text("Kristallerin yolculugu basliyor", 24.0, 108.0, 13.0, Color::new(0.60, 0.75, 0.96, 1.0));
            // Keep the map artwork visible; only add a light tint instead of covering it with an opaque panel.
            draw_rectangle(16.0, 130.0, w - 32.0, h - 170.0, Color::new(0.015, 0.025, 0.08, 0.24));
            // Winding path connecting level nodes.
            let map_top = 190.0_f32.min(h * 0.18);
            let map_bottom = h - 150.0;
            let map_span = (map_bottom - map_top).max(300.0);
            let nodes: [(f32, f32); 7] = [
                (w * 0.27, map_top + map_span * 0.02), (w * 0.70, map_top + map_span * 0.17),
                (w * 0.30, map_top + map_span * 0.33), (w * 0.70, map_top + map_span * 0.49),
                (w * 0.30, map_top + map_span * 0.65), (w * 0.70, map_top + map_span * 0.81),
                (w * 0.48, map_top + map_span * 0.97),
            ];
            for pair in nodes.windows(2) {
                draw_line(pair[0].0, pair[0].1, pair[1].0, pair[1].1, 8.0, Color::new(0.13, 0.24, 0.43, 1.0));
                draw_line(pair[0].0, pair[0].1, pair[1].0, pair[1].1, 3.0, Color::new(0.0, 0.72, 0.90, 0.55));
            }
            let names = ["Kristal Magarasi", "Neon Ormani", "Buz Tapinagi", "Lav Vadisi", "Yildiz Koprusu", "Golge Kalesi", "NEXUS Kapisi"];
            for (i, (x, y)) in nodes.iter().enumerate() {
                let unlocked = i == 0 || (i == 1 && completed_levels >= 1);
                let done = i == 0 && completed_levels >= 1;
                let c = if done { Color::new(0.0, 0.95, 0.48, 1.0) } else if unlocked { Color::new(0.0, 0.82, 1.0, 1.0) } else { Color::new(0.22, 0.29, 0.43, 1.0) };
                let node_r = (w * 0.065).clamp(25.0, 38.0);
                draw_circle(*x, *y, node_r + 10.0, Color::new(c.r, c.g, c.b, 0.16));
                draw_circle(*x, *y, node_r, c);
                draw_circle(*x, *y, node_r * 0.70, Color::new(0.04, 0.08, 0.18, 1.0));
                let label = if done { "✓".to_string() } else if unlocked { (i + 1).to_string() } else { "X".to_string() };
                draw_text(&label, *x - node_r * 0.30, *y + node_r * 0.32, node_r * 0.95, WHITE);
                let label_x = if *x > w * 0.5 { *x - w * 0.42 } else { *x + node_r + 12.0 };
                draw_text(names[i], label_x, *y + 5.0, 16.0, if unlocked { WHITE } else { Color::new(0.62, 0.68, 0.80, 1.0) });
            }
            draw_text("1. BOLUM", 24.0, h - 58.0, 15.0, Color::new(0.0, 0.88, 1.0, 1.0));
            if player.lives == 0 {
                draw_rectangle(24.0, h * 0.40, w - 48.0, 150.0, Color::new(0.025, 0.035, 0.10, 0.98));
                draw_rectangle_lines(24.0, h * 0.40, w - 48.0, 150.0, 2.0, Color::new(1.0, 0.22, 0.42, 1.0));
                draw_text("CANLAR BITTI!", 42.0, h * 0.40 + 35.0, 24.0, WHITE);
                let remain = player.remaining_refill_ms().saturating_add(999) / 1000;
                draw_text(&format!("Yeni can: {:02}:{:02}", remain / 60, remain % 60), 42.0, h * 0.40 + 68.0, 18.0, Color::new(0.0, 0.88, 1.0, 1.0));
                draw_text("Can satin alma: Google Play gerekli", 42.0, h * 0.40 + 103.0, 13.0, Color::new(1.0, 0.80, 0.18, 1.0));
                draw_text("Su an satin alma aktif degil", 42.0, h * 0.40 + 127.0, 12.0, Color::new(0.66, 0.73, 0.86, 1.0));
            }
            draw_text(if completed_levels > 0 { "Tamamlandi - sonraki bolum yakinda" } else { "Baslamak icin 1. bolume dokun" }, 24.0, h - 34.0, 12.0, Color::new(0.72, 0.82, 1.0, 1.0));
            if just_pressed() {
                let pointer = pointer_position();
                let (mx, my) = (pointer.x, pointer.y);
                let (x, y) = nodes[0];
                let hit_radius = (w * 0.085).clamp(34.0, 48.0);
                if (mx - x).powi(2) + (my - y).powi(2) <= hit_radius.powi(2) && player.lives > 0 {
                    if let Ok(new_level) = CrystalCavernLevel::new() {
                        level = new_level;
                        game_state = GameState::Playing;
                        level_finished = false;
                        outcome_applied = false;
                        moves_left = 25;
                        time_left = 90.0;
                        level_started = false;
                        stars = 0;
                        message = String::from("1. Bolum: Kristal Magarasi");
                        message_timer = 3.0;
                    }
                }
            }
            next_frame().await;
            continue;
        }

        if game_state == GameState::LevelComplete {
            let w = screen_width(); let h = screen_height();
            let pw = (w * 0.86).min(420.0); let ph = 300.0; let px = (w-pw)*0.5; let py = (h-ph)*0.5;
            draw_panel_texture(assets.level_complete.as_ref().or(assets.panel.as_ref()), px, py, pw, ph, Color::new(0.025, 0.055, 0.16, 0.98));
            draw_rectangle_lines(px, py, pw, ph, 2.0, Color::new(1.0, 0.80, 0.12, 1.0));
            draw_text("BOLUM TAMAMLANDI!", px+20.0, py+48.0, 24.0, Color::new(1.0, 0.86, 0.20, 1.0));
            draw_text(&format!("Skor: {}", level.score()), px+24.0, py+94.0, 22.0, WHITE);
            draw_text(&format!("Yildizlar: {} / 3", stars.max(1)), px+24.0, py+132.0, 20.0, Color::new(1.0, 0.86, 0.20, 1.0));
            let bw = pw-48.0; let by = py+ph-88.0;
            draw_panel_texture(assets.button.as_ref(), px+24.0, by, bw, 58.0, Color::new(0.02, 0.46, 0.30, 1.0));
            draw_text("HARITAYA DON", px+pw*0.5-76.0, by+37.0, 22.0, WHITE);
            draw_text("Sonraki bolum yeni icerik olarak eklenecek", px+24.0, py+ph-12.0, 11.0, Color::new(0.72, 0.82, 1.0, 1.0));
            if just_pressed() { let p = pointer_position(); if p.x >= px+24.0 && p.x <= px+24.0+bw && p.y >= by && p.y <= by+58.0 { game_state = GameState::Map; } }
            next_frame().await; continue;
        }

        draw_text("NEXUS", 22.0, 48.0, 24.0, Color::new(0.0, 0.88, 1.0, 1.0));
        draw_text("SORT", 112.0, 48.0, 24.0, Color::new(0.78, 0.28, 1.0, 1.0));
        draw_text("KRISTAL MAGARASI", 22.0, 78.0, 17.0, Color::new(0.82, 0.88, 1.0, 1.0));

        let card_y = 100.0;
        draw_rectangle(16.0, card_y, w - 32.0, 102.0, Color::new(0.06, 0.09, 0.22, 0.88));
        draw_rectangle_lines(16.0, card_y, w - 32.0, 102.0, 1.4, Color::new(0.0, 0.78, 1.0, 0.55));
        draw_text("SKOR", 30.0, 126.0, 14.0, Color::new(0.60, 0.72, 0.92, 1.0));
        draw_text(&format!("{}", level.score()), 30.0, 159.0, 31.0, WHITE);
        draw_text("HEDEF", w - 118.0, 126.0, 14.0, Color::new(0.60, 0.72, 0.92, 1.0));
        draw_text(&format!("{}", TARGET_SCORE), w - 118.0, 159.0, 31.0, Color::new(1.0, 0.83, 0.10, 1.0));
        let progress = (level.score() as f32 / TARGET_SCORE as f32).clamp(0.0, 1.0);
        draw_rectangle(30.0, 174.0, w - 60.0, 10.0, Color::new(0.12, 0.16, 0.30, 1.0));
        draw_rectangle(30.0, 174.0, (w - 60.0) * progress, 10.0, Color::new(0.0, 0.88, 1.0, 1.0));
        draw_text(&format!("HAMLE: {}  KALAN: {}", level.moves(), moves_left), 30.0, 194.0, 13.0, Color::new(0.72, 0.82, 1.0, 1.0));
        draw_text(&format!("CAN: {}/5", player.lives), w - 110.0, 194.0, 13.0, Color::new(1.0, 0.38, 0.52, 1.0));
        if timed_mode && !level_finished && !level.is_complete() && level_started {
            time_left = (time_left - get_frame_time()).max(0.0);
            let timer_color = if time_left <= 15.0 { RED } else { Color::new(1.0, 0.72, 0.18, 1.0) };
            draw_text(&format!("SURE: {:02}:{:02}", time_left as u32 / 60, time_left as u32 % 60), w * 0.5 - 48.0, 194.0, 14.0, timer_color);
            if time_left <= 0.0 {
                level_finished = true;
                if !outcome_applied {
                    player.apply_level_outcome(LevelOutcome::Failure, now_ms());
                    outcome_applied = true;
                    save_player(&player);
                    message = String::from("Sure bitti! Bir daha dene.");
                }
            }
        } else if timed_mode {
            draw_text("SURE: 01:30", w * 0.5 - 48.0, 194.0, 14.0, Color::new(1.0, 0.72, 0.18, 1.0));
        }

        // Responsive layout: size the 8x8 board from the actual available
        // viewport, keeping the HUD above it and the instruction/footer below.
        let horizontal_margin = (w * 0.035).clamp(8.0, 18.0);
        let board_top = (h * 0.115).clamp(118.0, 205.0);
        let footer_reserve = (h * 0.075).clamp(52.0, 88.0);
        let available_board_h = (h - board_top - footer_reserve - 28.0).max(160.0);
        let cell = (((w - horizontal_margin * 2.0) / 8.0)
            .min(available_board_h / 8.0))
            .max(1.0);
        let board_size = cell * 8.0;
        let board_x = (w - board_size) * 0.5;
        let board_y = board_top + ((available_board_h - board_size) * 0.5).max(0.0);
        draw_rectangle(board_x - 7.0, board_y - 7.0, board_size + 14.0, board_size + 14.0, Color::new(0.05, 0.10, 0.23, 0.96));
        draw_rectangle_lines(board_x - 7.0, board_y - 7.0, board_size + 14.0, board_size + 14.0, 2.0, Color::new(0.0, 0.82, 1.0, 0.8));

        for row in 0..8 {
            for col in 0..8 {
                let x = board_x + col as f32 * cell;
                let y = board_y + row as f32 * cell;
                draw_rectangle(x + 1.5, y + 1.5, cell - 3.0, cell - 3.0, Color::new(0.10, 0.15, 0.30, 0.94));
                if let Ok(Some(color)) = level.board().cell(row, col) {
                    if let Some(t)=assets.gem(color) { let side=cell*0.84; draw_texture_ex(t,x+(cell-side)*0.5,y+(cell-side)*0.5,WHITE,DrawTextureParams{dest_size:Some(vec2(side,side)),..Default::default()}); } else { draw_gem(x+cell*0.5,y+cell*0.5,cell*0.31,color); }
                }
            }
        }

        // Optional generated sprite overlays; procedural particles remain the fallback.
        if burst_timer > 0.0 {
            let t = 1.0 - burst_timer / 0.62;
            let effect_side = cell * (1.1 + t * 1.2);
            if let Some(texture) = assets.match_blast.as_ref().or(assets.shatter.as_ref()) {
                draw_texture_ex(texture, burst_center.x-effect_side*0.5, burst_center.y-effect_side*0.5,
                    Color::new(1.0, 1.0, 1.0, (1.0-t).clamp(0.0,1.0)),
                    DrawTextureParams { dest_size: Some(vec2(effect_side,effect_side)), ..Default::default() });
            }
            if let Some(texture) = assets.sparkle.as_ref() {
                let sparkle_side = cell * 0.65;
                for i in 0..4 {
                    let angle = i as f32 * std::f32::consts::FRAC_PI_2 + t;
                    let x = burst_center.x + angle.cos() * effect_side * 0.45 - sparkle_side*0.5;
                    let y = burst_center.y + angle.sin() * effect_side * 0.45 - sparkle_side*0.5;
                    draw_texture_ex(texture,x,y,Color::new(1.0,1.0,1.0,(1.0-t).clamp(0.0,1.0)),
                        DrawTextureParams { dest_size: Some(vec2(sparkle_side,sparkle_side)), ..Default::default() });
                }
            }
        }
        // Procedural match burst: sparkling rings and colorful shards around the last successful tap.
        if burst_timer > 0.0 {
            let t = 1.0 - burst_timer / 0.62;
            let radius = 18.0 + t * (cell * 2.2);
            let alpha = (1.0 - t).clamp(0.0, 1.0);
            for i in 0..12 {
                let a = i as f32 * std::f32::consts::TAU / 12.0 + t * 1.8;
                let px = burst_center.x + a.cos() * radius;
                let py = burst_center.y + a.sin() * radius;
                let color = match i % 5 {
                    0 => Color::new(1.0, 0.20, 0.42, alpha),
                    1 => Color::new(0.0, 0.88, 1.0, alpha),
                    2 => Color::new(0.20, 1.0, 0.48, alpha),
                    3 => Color::new(0.78, 0.25, 1.0, alpha),
                    _ => Color::new(1.0, 0.82, 0.08, alpha),
                };
                draw_circle(px, py, (cell * 0.045).max(1.5) * (1.0 - t * 0.45), color);
                draw_line(burst_center.x, burst_center.y, px, py, 1.0, Color::new(color.r, color.g, color.b, alpha * 0.35));
            }
            draw_circle_lines(burst_center.x, burst_center.y, radius * 0.65, 2.0, Color::new(0.45, 0.95, 1.0, alpha * 0.65));
        }

        if level.is_complete() {
            draw_rectangle(20.0, board_y + board_size * 0.36, w - 40.0, 112.0, Color::new(0.03, 0.07, 0.16, 0.96));
            draw_rectangle_lines(20.0, board_y + board_size * 0.36, w - 40.0, 112.0, 2.0, Color::new(1.0, 0.82, 0.0, 1.0));
            draw_text("KRISTAL MAGARASI", 34.0, board_y + board_size * 0.36 + 36.0, 20.0, Color::new(1.0, 0.85, 0.15, 1.0));
            draw_text("AYDINLANDI!", 34.0, board_y + board_size * 0.36 + 66.0, 26.0, WHITE);
            draw_text(&format!("YILDIZ: {} / 3", stars.max(1)), 34.0, board_y + board_size * 0.36 + 88.0, 15.0, Color::new(1.0, 0.84, 0.16, 1.0));
            draw_text("Haritaya donmek icin dokun", 34.0, board_y + board_size * 0.36 + 108.0, 13.0, Color::new(0.55, 0.92, 1.0, 1.0));
        } else if level_finished {
            draw_rectangle(20.0, board_y + board_size * 0.36, w - 40.0, 112.0, Color::new(0.03, 0.04, 0.12, 0.97));
            draw_rectangle_lines(20.0, board_y + board_size * 0.36, w - 40.0, 112.0, 2.0, Color::new(1.0, 0.22, 0.42, 1.0));
            draw_text("BOLUM TAMAMLANAMADI", 32.0, board_y + board_size * 0.36 + 35.0, 18.0, Color::new(1.0, 0.38, 0.52, 1.0));
            draw_text("Haritaya donmek icin dokun", 32.0, board_y + board_size * 0.36 + 72.0, 14.0, WHITE);
        } else {
            if message_timer > 0.0 {
                message_timer -= get_frame_time();
                draw_text(&message, 22.0, board_y + board_size + 38.0, 15.0, Color::new(0.68, 0.91, 1.0, 1.0));
            } else {
                draw_text("Bagli ayni renk kristallere dokun", 22.0, board_y + board_size + 38.0, 15.0, Color::new(0.68, 0.78, 0.98, 1.0));
            }
            draw_text("NEON KRISTALLER  /  BOLUM 01", 22.0, h - 82.0, 12.0, Color::new(0.38, 0.54, 0.78, 1.0));
            // Reserved, responsive power-up strip. Icons appear automatically when assets are supplied.
            let power_y = h - 68.0; let icon_side = (w * 0.14).clamp(38.0, 54.0); let gap = (w - icon_side * 4.0) / 5.0;
            let power_assets = [&assets.hammer, &assets.bomb, &assets.rainbow, &assets.shuffle];
            let power_labels = ["CEKIC", "BOMBA", "YILDIZ", "KARISTIR"];
            for i in 0..4 {
                let x = gap + i as f32 * (icon_side + gap);
                draw_panel_texture(power_assets[i].as_ref(), x, power_y, icon_side, icon_side, Color::new(0.07, 0.12, 0.27, 0.95));
                if power_assets[i].is_none() { draw_text(power_labels[i], x, power_y + icon_side + 12.0, 9.0, WHITE); }
            }
        }

        if just_pressed() {
            let pointer = pointer_position();
                let (mx, my) = (pointer.x, pointer.y);
            if level_finished || level.is_complete() {
                game_state = GameState::Map;
            } else if !level_finished && mx >= board_x && mx < board_x + board_size && my >= board_y && my < board_y + board_size {
                level_started = true;
                let col = ((mx - board_x) / cell) as usize;
                let row = ((my - board_y) / cell) as usize;
                match level.play_at(row, col) {
                    Ok(0) => {
                        message = String::from("Bu grupta en az 3 bagli kristal olmali.");
                        message_timer = 2.2;
                    }
                    Ok(count) => {
                        moves_left = moves_left.saturating_sub(1);
                        burst_center = vec2(mx, my);
                        burst_timer = 0.62;
                        let bonus = if timed_mode && count >= 5 { 5.0 } else if timed_mode && count >= 4 { 3.0 } else { 0.0 };
                        time_left = (time_left + bonus).min(120.0);
                        stars = if level.score() >= TARGET_SCORE { if time_left >= 45.0 { 3 } else if time_left >= 20.0 { 2 } else { 1 } } else { 0 };
                        message = if bonus > 0.0 { format!("KRISTAL PATLAMASI! +{} PUAN  +{} sn", count * 10, bonus as u32) } else { format!("KRISTAL PATLAMASI! +{} PUAN", count * 10) };
                        message_timer = 2.4;
                        if level.is_complete() && !outcome_applied {
                            player.apply_level_outcome(LevelOutcome::Success, now_ms());
                            completed_levels = player.level_progress;
                            outcome_applied = true;
                            game_state = GameState::LevelComplete;
                            save_player(&player);
                        } else if moves_left == 0 && !level.is_complete() {
                            player.apply_level_outcome(LevelOutcome::Failure, now_ms());
                            level_finished = true;
                            outcome_applied = true;
                            game_state = GameState::LevelComplete;
                            save_player(&player);
                        }
                    }
                    Err(error) => {
                        message = format!("Hamle hatasi: {error}");
                        message_timer = 2.4;
                    }
                }
            }
        }
        next_frame().await;
    }
}
