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

    // Estado para a tabela de distâncias
    let mut robot_distances: Vec<(usize, f32)> = Vec::new(); // Armazena as distâncias calculadas (ID, Distância) para a HUD
    let mut poly_id_counter = 0; // Contador para dar ID aos polígonos
    let mut finalized_hull_ids: Vec<usize> = Vec::new(); // Armazena os IDs dos polígonos na lista `finalized_hulls`

    // Polígono B (Robô) - usado para DESENHAR
    let robot_shape = vec![
        Point { x: 0.0, y: -40.0 },  // 1. Ponta de cima
        Point { x: 15.0, y: -5.0 },  // 2. Canto superior direito ("telhado")
        Point { x: 15.0, y: 20.0 },  // 3. Canto inferior direito
        Point { x: -15.0, y: 20.0 }, // 4. Canto inferior esquerdo
        Point { x: -15.0, y: -5.0 }, // 5. Canto superior esquerdo ("telhado")
    ];

    // Polígono -B (Robô Refletido) - usado para CÁLCULO (C-Obstacle = A ⊕ (-B))
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
                finalized_hull_ids.push(poly_id_counter); // Associa ID
                poly_id_counter += 1;
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
                finalized_hull_ids.push(poly_id_counter); // Associa ID
                poly_id_counter += 1;
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
            finalized_hull_ids.clear(); // Limpa IDs
            poly_id_counter = 0; // Reseta contador
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
            // Mostra o estado atual antes de pausar para o benchmark
            render_points(&active_points);
            render_hull(&active_hull);
            for hull in &finalized_hulls {
                render_hull(hull);
            }
            // Renderiza a HUD uma vez com a msg do benchmark
            render_hud(
                active_points.len(),
                last_exec_time,
                &Vec::new(),
                &benchmark_message,
            );
            next_frame().await;

            benchmark_message = run_benchmark();
        }

        // --- Lógica de Cálculo ---
        let start = Instant::now();

        c_obstacles.clear();
        robot_distances.clear(); // Limpa a tabela de distâncias

        // 1. Calcula o hull do polígono ativo
        if !active_points.is_empty() {
            active_hull = convex_hull(active_points.clone());
        } else {
            active_hull.clear();
        }

        // 2. Calcula C-Obstacle para todos os polígonos finalizados
        for (i, hull) in finalized_hulls.iter().enumerate() {
            let c_obs = minkowski_sum(hull, &robot_shape_reflected);

            // Calcula a distância do mouse ao C-Obstacle
            let dist = min_distance_from_point(&c_obs, mouse_point);
            let id = finalized_hull_ids[i]; // Pega o ID
            robot_distances.push((id, dist)); // Salva (ID, Dist)

            c_obstacles.push(c_obs);
        }

        // 3. Calcula C-Obstacle para o polígono ativo
        if !active_hull.is_empty() {
            let c_obs_active = minkowski_sum(&active_hull, &robot_shape_reflected);

            // Calcula a distância do mouse ao C-Obstacle ativo
            let dist_active = min_distance_from_point(&c_obs_active, mouse_point);
            robot_distances.push((poly_id_counter, dist_active)); // Usa o próximo ID

            c_obstacles.push(c_obs_active);
        }

        last_exec_time = start.elapsed().as_secs_f32() * 1_000_000.0; // µs

        // --- Renderização ---

        // Desenha os pontos do polígono que está sendo criado
        render_points(&active_points);

        // Desenha todos os obstáculos vermelhos (finalizados e o ativo)
        for hull in &finalized_hulls {
            render_hull(hull);
        }
        render_hull(&active_hull);

        // Desenha todas as zonas de não-colisão azuis
        for c_obs in &c_obstacles {
            render_polygon(c_obs, BLUE, 1.0);
        }

        // Desenha o robô na posição do mouse
        render_robot(&robot_shape, mouse_point, YELLOW);

        // Renderiza a HUD principal com todos os dados
        render_hud(
            active_points.len(),
            last_exec_time,
            &robot_distances,
            &benchmark_message,
        );

        next_frame().await;
    }
}
