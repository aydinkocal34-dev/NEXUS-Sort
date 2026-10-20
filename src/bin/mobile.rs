use macroquad::prelude::*;
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
        high_dpi: false,
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

fn draw_background() {
    clear_background(Color::new(0.025, 0.035, 0.105, 1.0));
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
    let mut level = match CrystalCavernLevel::new() {
        Ok(level) => level,
        Err(error) => {
            loop {
                draw_background();
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
    let mut show_map = true;
    let mut completed_levels = player.level_progress;
    let mut level_finished = false;
    let mut moves_left = 25_u32;
    let mut outcome_applied = false;

    loop {
        let w = screen_width();
        let h = screen_height();
        draw_background();

        let refill = player.apply_refill(now_ms());
        if refill.lives_after != refill.lives_before {
            save_player(&player);
        }
        completed_levels = player.level_progress;

        if show_map {
            draw_text("NEXUS", 24.0, 52.0, 30.0, Color::new(0.0, 0.88, 1.0, 1.0));
            draw_text("MACERA HARITASI", 24.0, 84.0, 19.0, WHITE);
            draw_text("Kristallerin yolculugu basliyor", 24.0, 108.0, 13.0, Color::new(0.60, 0.75, 0.96, 1.0));
            draw_rectangle(16.0, 130.0, w - 32.0, h - 170.0, Color::new(0.04, 0.07, 0.17, 0.88));
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
                let (mx, my) = mouse_position();
                let (x, y) = nodes[0];
                let hit_radius = (w * 0.085).clamp(34.0, 48.0);
                if (mx - x).powi(2) + (my - y).powi(2) <= hit_radius.powi(2) && player.lives > 0 {
                    if let Ok(new_level) = CrystalCavernLevel::new() {
                        level = new_level;
                        show_map = false;
                        level_finished = false;
                        outcome_applied = false;
                        moves_left = 25;
                        message = String::from("1. Bolum: Kristal Magarasi");
                        message_timer = 3.0;
                    }
                }
            }
            next_frame().await;
            continue;
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

        let available_board_h = (h - 300.0).max(240.0);
        let cell = (((w - 28.0) / 8.0).min(available_board_h / 8.0)).max(1.0);
        let board_size = cell * 8.0;
        let board_x = (w - board_size) * 0.5;
        let board_y = 220.0_f32.min((h - board_size - 100.0).max(190.0));
        draw_rectangle(board_x - 7.0, board_y - 7.0, board_size + 14.0, board_size + 14.0, Color::new(0.05, 0.10, 0.23, 0.96));
        draw_rectangle_lines(board_x - 7.0, board_y - 7.0, board_size + 14.0, board_size + 14.0, 2.0, Color::new(0.0, 0.82, 1.0, 0.8));

        for row in 0..8 {
            for col in 0..8 {
                let x = board_x + col as f32 * cell;
                let y = board_y + row as f32 * cell;
                draw_rectangle(x + 1.5, y + 1.5, cell - 3.0, cell - 3.0, Color::new(0.10, 0.15, 0.30, 0.94));
                if let Ok(Some(color)) = level.board().cell(row, col) {
                    draw_gem(x + cell * 0.5, y + cell * 0.5, cell * 0.31, color);
                }
            }
        }

        if level.is_complete() {
            draw_rectangle(20.0, board_y + board_size * 0.36, w - 40.0, 112.0, Color::new(0.03, 0.07, 0.16, 0.96));
            draw_rectangle_lines(20.0, board_y + board_size * 0.36, w - 40.0, 112.0, 2.0, Color::new(1.0, 0.82, 0.0, 1.0));
            draw_text("KRISTAL MAGARASI", 34.0, board_y + board_size * 0.36 + 36.0, 20.0, Color::new(1.0, 0.85, 0.15, 1.0));
            draw_text("AYDINLANDI!", 34.0, board_y + board_size * 0.36 + 66.0, 26.0, WHITE);
            draw_text("Haritaya donmek icin dokun", 34.0, board_y + board_size * 0.36 + 92.0, 15.0, Color::new(0.55, 0.92, 1.0, 1.0));
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
            draw_text("NEON KRISTALLER  /  BOLUM 01", 22.0, h - 24.0, 12.0, Color::new(0.38, 0.54, 0.78, 1.0));
        }

        if just_pressed() {
            let (mx, my) = mouse_position();
            if level_finished || level.is_complete() {
                show_map = true;
            } else if !level_finished && mx >= board_x && mx < board_x + board_size && my >= board_y && my < board_y + board_size {
                let col = ((mx - board_x) / cell) as usize;
                let row = ((my - board_y) / cell) as usize;
                match level.play_at(row, col) {
                    Ok(0) => {
                        message = String::from("Bu grupta en az 3 bagli kristal olmali.");
                        message_timer = 2.2;
                    }
                    Ok(count) => {
                        moves_left = moves_left.saturating_sub(1);
                        message = format!("KRISTAL PATLAMASI! +{} PUAN", count * 10);
                        message_timer = 2.4;
                        if level.is_complete() && !outcome_applied {
                            player.apply_level_outcome(LevelOutcome::Success, now_ms());
                            completed_levels = player.level_progress;
                            outcome_applied = true;
                            save_player(&player);
                        } else if moves_left == 0 && !level.is_complete() {
                            player.apply_level_outcome(LevelOutcome::Failure, now_ms());
                            level_finished = true;
                            outcome_applied = true;
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
