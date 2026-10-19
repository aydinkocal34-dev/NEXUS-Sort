use macroquad::prelude::*;
use nexus_sort::game::board::Color as GemColor;
use nexus_sort::game::crystal_cavern::{CrystalCavernLevel, TARGET_SCORE};

fn window_conf() -> Conf {
    Conf {
        window_title: "NEXUS SORT - Kristal Magarasi".to_owned(),
        window_width: 450,
        window_height: 900,
        high_dpi: true,
        sample_count: 4,
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
    let mut message = String::from("Birbirine bagli 3+ kristale dokun!");
    let mut message_timer = 4.0_f32;

    loop {
        let w = screen_width();
        let h = screen_height();
        draw_background();

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
        draw_text(&format!("HAMLE: {}", level.moves()), 30.0, 194.0, 13.0, Color::new(0.72, 0.82, 1.0, 1.0));

        let cell = ((w - 34.0) / 8.0).min(62.0);
        let board_size = cell * 8.0;
        let board_x = (w - board_size) * 0.5;
        let board_y = (h * 0.30).max(222.0).min(h - board_size - 112.0);
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
            draw_text("Tekrar baslamak icin dokun", 34.0, board_y + board_size * 0.36 + 92.0, 15.0, Color::new(0.55, 0.92, 1.0, 1.0));
        } else {
            if message_timer > 0.0 {
                message_timer -= get_frame_time();
                draw_text(&message, 22.0, board_y + board_size + 38.0, 15.0, Color::new(0.68, 0.91, 1.0, 1.0));
            } else {
                draw_text("Bagli ayni renk kristallere dokun", 22.0, board_y + board_size + 38.0, 15.0, Color::new(0.68, 0.78, 0.98, 1.0));
            }
            draw_text("NEON KRISTALLER  /  BOLUM 01", 22.0, h - 24.0, 12.0, Color::new(0.38, 0.54, 0.78, 1.0));
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            if level.is_complete() {
                if my > board_y && my < board_y + board_size {
                    if let Ok(new_level) = CrystalCavernLevel::new() {
                        level = new_level;
                        message = String::from("Yeni macera basladi!");
                        message_timer = 3.0;
                    }
                }
            } else if mx >= board_x && mx < board_x + board_size && my >= board_y && my < board_y + board_size {
                let col = ((mx - board_x) / cell) as usize;
                let row = ((my - board_y) / cell) as usize;
                match level.play_at(row, col) {
                    Ok(0) => {
                        message = String::from("Bu grupta en az 3 bagli kristal olmali.");
                        message_timer = 2.2;
                    }
                    Ok(count) => {
                        message = format!("KRISTAL PATLAMASI! +{} PUAN", count * 10);
                        message_timer = 2.4;
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
