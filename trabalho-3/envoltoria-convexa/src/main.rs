use macroquad::prelude::*;
use std::time::Instant;

mod benchmark;
mod convex_hull;
mod geometry;
mod renderer;

use benchmark::run_benchmark;
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

    let mut benchmark_message = String::new();

    loop {
        clear_background(BLACK);

        // Adiciona ponto ao clicar com o mouse
        if is_mouse_button_pressed(MouseButton::Left) {
            // Se clicar com botão esquerdo
            let (x, y) = mouse_position(); // Salva a posição do mouse em x e y
            points.push(Point { x, y }); // Cria um ponto nas coordenadas (x,y)
            benchmark_message.clear();
        }

        // Teclas de controle

        // R: Pontos aleatórios
        if is_key_pressed(KeyCode::R) {
            points = random_points(30);
            benchmark_message.clear();
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
            benchmark_message.clear();
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
            benchmark_message.clear();
        }
        // Espaço: Limpa a tela
        if is_key_pressed(KeyCode::Space) {
            points.clear();
            hull.clear();
            benchmark_message.clear();
        }
        // Ctrl+C ou ESC: Salva e fecha
        if (is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl))
            && is_key_pressed(KeyCode::C)
            || is_key_pressed(KeyCode::Escape)
        {
            break;
        }

        if is_key_pressed(KeyCode::B) {
            benchmark_message = "Executando benchmark".to_string();

            clear_background(BLACK);
            render_points(&points);
            render_hull(&hull);
            render_hud(points.len(), last_exec_time);
            draw_text(&benchmark_message, 20.0, 90.0, 24.0, WHITE);
            next_frame().await; // Mostra a mensagem "Executando..."

            // O app vai pausar aqui enquanto o benchmark é executado
            benchmark_message = run_benchmark();
        }

        // Recalcula a envoltória convexa automaticamente e mede o tempo
        if !points.is_empty() {
            // Verifica se tem pontos
            let start = Instant::now();
            hull = convex_hull(points.clone());
            last_exec_time = start.elapsed().as_secs_f32() * 1_000_000.0; // µs
        } else {
            last_exec_time = 0.0;
            hull.clear();
        }

        // Renderização
        render_points(&points);
        render_hull(&hull);
        render_hud(points.len(), last_exec_time);

        // Mostra a mensagem do benchmark se houver
        if !benchmark_message.is_empty() {
            draw_text(&benchmark_message, 20.0, 90.0, 24.0, WHITE);
        }

        next_frame().await;
    }
}
