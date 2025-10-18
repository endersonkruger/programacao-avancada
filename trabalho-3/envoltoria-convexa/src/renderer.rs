use crate::geometry::Point;
use macroquad::prelude::*;

// Desenha os pontos
pub fn render_points(points: &Vec<Point>) {
    for p in points {
        draw_circle(p.x, p.y, 5.0, WHITE);
    }
}

// Desenha as linhas da envoltória convexa
pub fn render_hull(hull: &Vec<Point>) {
    if hull.len() > 1 {
        for i in 0..hull.len() {
            let a = hull[i];
            let b = hull[(i + 1) % hull.len()];
            draw_line(a.x, a.y, b.x, b.y, 2.0, RED);
        }
    }
}

// Mostra HUD (informações)
pub fn render_hud(num_points: usize, exec_time_us: f32) {
    draw_text(&format!("Pontos: {}", num_points), 20.0, 30.0, 24.0, GREEN);
    draw_text(
        &format!("Tempo: {:.2} µs", exec_time_us),
        20.0,
        60.0,
        24.0,
        YELLOW,
    );
    draw_text(
        "Clique: Ponto | R: Aleatórios | C: Círculo | T: Retângulo | B: BENCHMARK | Espaço: Limpar | Esc/Ctrl+C: Salvar e Sair",
        20.0,
        screen_height() - 30.0,
        20.0,
        GRAY,
    );
}
