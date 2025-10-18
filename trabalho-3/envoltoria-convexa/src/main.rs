use macroquad::prelude::*;
use std::time::Instant;

mod convex_hull;
mod geometry;
mod renderer;

use convex_hull::*;
use geometry::*;
use renderer::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Envoltória Convexa".to_string(),
        window_width: 1280,
        window_height: 720,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut points: Vec<Point> = Vec::new();
    let mut hull: Vec<Point> = Vec::new();
    let mut last_exec_time: f32 = 0.0;

    loop {
        clear_background(BLACK);

        // Adiciona ponto ao clicar com o mouse
        if is_mouse_button_pressed(MouseButton::Left) {
            // Se clicar com botão esquerdo
            let (x, y) = mouse_position(); // Salva a posição do mouse em x e y
            points.push(Point { x, y }); // Cria um ponto nas coordenadas (x,y)
        }

        // Teclas de controle

        // R: Pontos aleatórios
        if is_key_pressed(KeyCode::R) {
            points = random_points(30);
        }
        // C: Pontos em círculo
        if is_key_pressed(KeyCode::C) {
            points = circle_points(
                30,
                Point {
                    x: screen_width() / 2.0,
                    y: screen_height() / 2.0,
                },
                200.0,
            );
        }
        // T: Pontos em retângulo
        if is_key_pressed(KeyCode::T) {
            points = rectangle_points(
                40,
                Point {
                    x: screen_width() / 2.0,
                    y: screen_height() / 2.0,
                },
                400.0,
                250.0,
            );
        }
        // Espaço: Limpa a tela
        if is_key_pressed(KeyCode::Space) {
            points.clear();
            hull.clear();
        }
        // Ctrl+C ou ESC: Salva e fecha
        if (is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl))
            && is_key_pressed(KeyCode::C)
            || is_key_pressed(KeyCode::Escape)
        {
            break;
        }

        // Recalcula a envoltória convexa automaticamente e mede o tempo
        if !points.is_empty() {
            // Verifica se tem pontos
            let start = Instant::now();
            hull = convex_hull(points.clone());
            last_exec_time = start.elapsed().as_secs_f32() * 1_000_000.0; // µs
        }

        // Renderização
        render_points(&points);
        render_hull(&hull);
        render_hud(points.len(), last_exec_time);

        next_frame().await;
    }
}
