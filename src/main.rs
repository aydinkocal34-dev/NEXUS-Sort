mod game;

use game::level::{get_level, LEVELS};
use game::GameState;
use std::io::{self, Write};

fn read_line() -> io::Result<String> {
    let mut input = String::new();
    let bytes_read = io::stdin().read_line(&mut input)?;
    if bytes_read == 0 {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "stdin closed"));
    }
    Ok(input.trim().to_string())
}

fn print_levels() {
    println!("\n=== NEXUS SORT ===\nBir seviye sec:");
    for (i, level) in LEVELS.iter().enumerate() {
        println!("{}. {} - {}", i + 1, level.name, level.description);
    }
    println!("q. Cikis");
}

fn render_game(game: &GameState) {
    println!("\n=== NEXUS SORT | {} ===", game.level_name());
    println!("Skor: {} | Temizlenen: {} | Basarili hamle: {}", game.score(), game.cleared_pieces(), game.moves());
    print!("{}", game.board().render());
    println!("Hamle icin satir sutun (ornek: 2 3); m: menu; q: cikis");
}

fn run_game(game: &mut GameState) -> io::Result<bool> {
    loop {
        render_game(game);
        print!("Hamle > ");
        io::stdout().flush()?;
        let input = read_line()?;
        match input.as_str() {
            "q" | "Q" => return Ok(false),
            "m" | "M" => return Ok(true),
            "" => continue,
            _ => {}
        }
        let mut parts = input.split_whitespace();
        let row = match parts.next().and_then(|s| s.parse::<usize>().ok()) {
            Some(n) if n > 0 => n - 1,
            _ => { println!("Gecersiz satir. Ornek: 2 3"); continue; }
        };
        let col = match parts.next().and_then(|s| s.parse::<usize>().ok()) {
            Some(n) if n > 0 => n - 1,
            _ => { println!("Gecersiz sutun. Ornek: 2 3"); continue; }
        };
        if parts.next().is_some() { println!("Yalnizca satir ve sutun gir."); continue; }
        match game.play_at(row, col) {
            Ok(0) => println!("Bu hucrede temizlenecek en az 3 bagli ayni renk parca yok."),
            Ok(count) => println!("{count} parca temizlendi. +{} puan!", count * 10),
            Err(error) => println!("Hamle uygulanamadi: {error}"),
        }
    }
}

fn main() {
    loop {
        print_levels();
        print!("Secim > ");
        if let Err(error) = io::stdout().flush() { eprintln!("Terminal hatasi: {error}"); return; }
        let input = match read_line() {
            Ok(value) => value,
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => {
                println!("\nGirdi kapandi; NEXUS Sort kapatiliyor.");
                break;
            }
            Err(error) => { eprintln!("Girdi okunamadi: {error}"); return; }
        };
        if input.eq_ignore_ascii_case("q") { println!("NEXUS Sort kapatildi."); break; }
        let index = match input.parse::<usize>() {
            Ok(n) if (1..=LEVELS.len()).contains(&n) => n - 1,
            _ => { println!("Lutfen 1, 2 veya 3 gir."); continue; }
        };
        let Some(level) = get_level(index) else { println!("Seviye bulunamadi."); continue; };
        let mut game = match GameState::new(level) {
            Ok(game) => game,
            Err(error) => { eprintln!("Oyun baslatilamadi: {error}"); continue; }
        };
        match run_game(&mut game) {
            Ok(true) => {}
            Ok(false) => { println!("NEXUS Sort kapatildi."); break; }
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => {
                println!("\nGirdi kapandi; NEXUS Sort kapatiliyor.");
                break;
            }
            Err(error) => { eprintln!("Oyun dongusu hatasi: {error}"); break; }
        }
    }
}
