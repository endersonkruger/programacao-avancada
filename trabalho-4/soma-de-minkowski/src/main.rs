use macroquad::prelude::*;
use std::time::Instant;

mod benchmark;
mod convex_hull;
mod geometry;
mod minkowski;
mod renderer;

use benchmark::run_benchmark;
use convex_hull::*;
use geometry::*;
use minkowski::minkowski_sum;
use renderer::{render_hud, render_hull, render_points, render_polygon, render_robot};

fn window_conf() -> Conf {
    Conf {
        window_title: "Envoltória Convexa e Soma de Minkowski".to_string(),
        window_width: 1280,
        window_height: 720,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // --- Estado da Aplicação ---
    let mut active_points: Vec<Point> = Vec::new(); // Pontos do polígono sendo desenhado
    let mut active_hull: Vec<Point> = Vec::new(); // Envoltória do polígono sendo desenhado
    let mut finalized_hulls: Vec<Vec<Point>> = Vec::new(); // Lista de obstáculos concluídos
    let mut c_obstacles: Vec<Vec<Point>> = Vec::new(); // Lista de C-Obstacles (zonas de não-colisão)

    // Polígono B (Robô) - usado para DESENHAR
    let robot_shape = vec![
        Point { x: 0.0, y: -40.0 },  // 1. Ponta de cima
        Point { x: 15.0, y: -5.0 },  // 2. Canto superior direito ("telhado")
        Point { x: 15.0, y: 20.0 },  // 3. Canto inferior direito
        Point { x: -15.0, y: 20.0 }, // 4. Canto inferior esquerdo
        Point { x: -15.0, y: -5.0 }, // 5. Canto superior esquerdo ("telhado")
    ];

    // Polígono -B (Robô Refletido) - usado para CÁLCULO
    // O C-Obstacle é A ⊕ (-B)
    let robot_shape_reflected: Vec<Point> = robot_shape
        .iter()
        .map(|p| Point { x: -p.x, y: -p.y }) // Inverte todas as coordenadas
        .collect();

    let mut last_exec_time: f32 = 0.0;
    let mut benchmark_message = String::new();

    loop {
        clear_background(BLACK);

        let (mx, my) = mouse_position();
        let mouse_point = Point { x: mx, y: my };

        // --- Lógica de Input ---

        // 1. Adiciona ponto ao polígono ativo
        if is_mouse_button_pressed(MouseButton::Left) {
            active_points.push(mouse_point);
            benchmark_message.clear();
        }

        // 2. Tecla Enter finaliza o polígono atual
        if is_key_pressed(KeyCode::Enter) {
            if !active_hull.is_empty() {
                finalized_hulls.push(active_hull.clone()); // Salva o hull atual
                active_points.clear(); // Limpa para o próximo
                active_hull.clear();
            }
            benchmark_message.clear();
        }

        // 3. Geradores de polígonos (R, C, T)
        //    Eles finalizam o polígono anterior e iniciam um novo
        let mut generated_points: Option<Vec<Point>> = None;
        if is_key_pressed(KeyCode::R) {
            generated_points = Some(random_points(30));
        }
        if is_key_pressed(KeyCode::C) {
            generated_points = Some(circle_points(
                30,
                Point {
                    x: screen_width() / 2.0,
                    y: screen_height() / 2.0,
                },
                200.0,
            ));
        }
        if is_key_pressed(KeyCode::T) {
            generated_points = Some(rectangle_points(
                40,
                Point {
                    x: screen_width() / 2.0,
                    y: screen_height() / 2.0,
                },
                400.0,
                250.0,
            ));
        }

        if let Some(new_points) = generated_points {
            // Finaliza o polígono antigo, se existir
            if !active_hull.is_empty() {
                finalized_hulls.push(active_hull.clone());
            }
            // Define o novo polígono ativo
            active_points = new_points;
            benchmark_message.clear();
        }

        // 4. Limpar (Espaço) - Limpa todos os polígonos
        if is_key_pressed(KeyCode::Space) {
            active_points.clear();
            active_hull.clear();
            finalized_hulls.clear();
            c_obstacles.clear();
            benchmark_message.clear();
        }

        // 5. Sair (Esc/Ctrl+C)
        if (is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl))
            && is_key_pressed(KeyCode::C)
            || is_key_pressed(KeyCode::Escape)
        {
            break;
        }

        // 6. Benchmark (B)
        if is_key_pressed(KeyCode::B) {
            benchmark_message = "Executando benchmark".to_string();

            clear_background(BLACK);
            // Mostra o estado atual antes de pausar
            render_points(&active_points);
            render_hull(&active_hull);
            for hull in &finalized_hulls {
                render_hull(hull);
            }
            render_hud(active_points.len(), last_exec_time);
            draw_text(&benchmark_message, 20.0, 90.0, 24.0, WHITE);
            next_frame().await;

            benchmark_message = run_benchmark();
        }

        // --- Lógica de Cálculo ---
        let start = Instant::now();

        // 1. Limpa os C-Obstacles (serão recalculados a cada frame)
        c_obstacles.clear();

        // 2. Calcula o hull do polígono ativo
        if !active_points.is_empty() {
            active_hull = convex_hull(active_points.clone());
        } else {
            active_hull.clear();
        }

        // 3. Calcula C-Obstacle para todos os polígonos finalizados
        for hull in &finalized_hulls {
            c_obstacles.push(minkowski_sum(hull, &robot_shape_reflected));
        }

        // 4. Calcula C-Obstacle para o polígono ativo
        if !active_hull.is_empty() {
            c_obstacles.push(minkowski_sum(&active_hull, &robot_shape_reflected));
        }

        last_exec_time = start.elapsed().as_secs_f32() * 1_000_000.0; // µs

        // --- Renderização ---

        // 1. Pontos do polígono ativo
        render_points(&active_points);

        // 2. Renderiza todos os hulls finalizados (VERMELHO)
        for hull in &finalized_hulls {
            render_hull(hull);
        }

        // 3. Renderiza o hull ativo (VERMELHO)
        render_hull(&active_hull);

        // 4. Renderiza todos os C-Obstacles (AZUL)
        for c_obs in &c_obstacles {
            render_polygon(c_obs, BLUE, 1.0);
        }

        // 5. O "Robô" na posição do mouse (AMARELO)
        render_robot(&robot_shape, mouse_point, YELLOW);

        // 6. HUD (mostra contagem de pontos ativos)
        render_hud(active_points.len(), last_exec_time);

        // Mostra a mensagem do benchmark se houver
        if !benchmark_message.is_empty() {
            draw_text(&benchmark_message, 20.0, 90.0, 24.0, WHITE);
        }

        next_frame().await;
    }
}
