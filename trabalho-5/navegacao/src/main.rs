use macroquad::prelude::*;

// --- Módulos do Projeto ---
mod agent; // Define a lógica do Agente (movimento)
mod benchmark;
mod grid; // Define a estrutura do Grid (células, obstáculos)
mod pathfinding; // Implementa o algoritmo A* (A-Star)
mod renderer; // Contém todas as funções de desenho // Contém a lógica para rodar o benchmark de A*

use agent::Agent;
use benchmark::run_benchmark;
use grid::{CellType, Grid};
use pathfinding::a_star_search; // Importa a função de benchmark

// --- Constantes da Simulação ---
const CELL_SIZE: f32 = 20.0; // Tamanho (em pixels) de cada célula do grid
const GRID_WIDTH: usize = 60; // Largura do grid (em número de células)
const GRID_HEIGHT: usize = 36; // Altura do grid (em número de células)
const AGENT_SPEED: f32 = 150.0; // Velocidade de movimento do agente (pixels/seg)

/// Define o modo de interação atual do usuário com o mouse.
#[derive(PartialEq, Debug)]
enum InputMode {
    DrawObstacle, // Clicar/arrastar desenha obstáculos
    SetStart,     // O próximo clique define o ponto inicial de um agente
    SetEnd,       // O próximo clique define o ponto final de um agente
}

/// Converte coordenadas de tela (pixels) para coordenadas do grid (índice da célula).
fn screen_to_grid(screen_pos: Vec2) -> (usize, usize) {
    let x = (screen_pos.x / CELL_SIZE).floor() as usize;
    let y = (screen_pos.y / CELL_SIZE).floor() as usize;

    // Garante que o resultado esteja dentro dos limites do grid
    (x.clamp(0, GRID_WIDTH - 1), y.clamp(0, GRID_HEIGHT - 1))
}

/// Converte coordenadas do grid (índice) para a posição central da célula na tela (pixels).
fn grid_to_screen_center(grid_pos: (usize, usize)) -> Vec2 {
    Vec2::new(
        grid_pos.0 as f32 * CELL_SIZE + CELL_SIZE * 0.5,
        grid_pos.1 as f32 * CELL_SIZE + CELL_SIZE * 0.5,
    )
}

/// Gera `n` agentes com posições e destinos aleatórios.
/// Encontra caminhos para eles e os adiciona à lista de agentes.
fn spawn_random_agents(n: usize, grid: &Grid, agents: &mut Vec<Agent>) {
    let mut count = 0;
    for _ in 0..n {
        // Tenta encontrar um início e fim válidos (células não-obstáculo)
        if let (Some(start_pos), Some(end_pos)) =
            (grid.get_random_empty_cell(), grid.get_random_empty_cell())
        {
            // Calcula o caminho usando A*
            if let Some(path_nodes) = a_star_search(grid, start_pos, end_pos) {
                // Converte o caminho de nós do grid (usize) para posições de pixel (Vec2)
                let pixel_path = path_nodes.into_iter().map(grid_to_screen_center).collect();

                let start_pixel_pos = grid_to_screen_center(start_pos);
                agents.push(Agent::new(start_pixel_pos, pixel_path, AGENT_SPEED));
                count += 1;
            }
        }
    }
    println!("Gerado {} agentes aleatórios.", count);
}

/// Configurações da janela do Macroquad.
fn window_conf() -> Conf {
    Conf {
        window_title: "Navegação".to_string(),
        window_width: (GRID_WIDTH as f32 * CELL_SIZE) as i32,
        window_height: (GRID_HEIGHT as f32 * CELL_SIZE) as i32,
        ..Default::default()
    }
}

/// Ponto de entrada principal da aplicação.
#[macroquad::main(window_conf)]
async fn main() {
    // --- Estado da Aplicação ---
    let mut grid = Grid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut agents: Vec<Agent> = Vec::new();
    let mut mode = InputMode::DrawObstacle;

    // Armazena o ponto inicial enquanto o usuário escolhe o ponto final
    let mut pending_start: Option<(usize, usize)> = None;
    // Armazena a mensagem de resultado do benchmark
    let mut benchmark_message = String::new();

    // --- Loop Principal ---
    loop {
        let dt = get_frame_time(); // Delta time (tempo desde o último frame)
        let mouse_pos = mouse_position();
        let (grid_x, grid_y) = screen_to_grid(Vec2::new(mouse_pos.0, mouse_pos.1));

        // --- 1. Lógica de Input (Teclado) ---

        // [O] - Modo Obstáculo
        if is_key_pressed(KeyCode::O) {
            mode = InputMode::DrawObstacle;
            pending_start = None;
            println!("Modo: Desenhar Obstáculos");
        }

        // [A] - Modo Agente (inicia o processo de definição)
        if is_key_pressed(KeyCode::A) {
            mode = InputMode::SetStart;
            pending_start = None;
            println!("Modo: Definir Agente (Clique no Início)");
        }

        // [C] - Limpar (Clear)
        if is_key_pressed(KeyCode::C) {
            grid.clear();
            agents.clear();
            pending_start = None;
            mode = InputMode::DrawObstacle;
            benchmark_message.clear(); // Limpa a msg do benchmark
            println!("Grid e agentes limpos.");
        }

        // [R] - Gerar Agentes Aleatórios (Random)
        if is_key_pressed(KeyCode::R) {
            spawn_random_agents(20, &grid, &mut agents);
            benchmark_message.clear();
        }

        // [B] - Executar Benchmark
        if is_key_pressed(KeyCode::B) {
            mode = InputMode::DrawObstacle; // Entra no modo de obstáculo
            pending_start = None;
            benchmark_message = "Executando benchmark...".to_string();

            // Renderiza a tela uma vez para mostrar a msg de "Executando..."
            clear_background(Color::from_hex(0x111111));
            renderer::draw_grid(GRID_WIDTH, GRID_HEIGHT, CELL_SIZE);
            renderer::draw_cells(&grid, CELL_SIZE);
            renderer::draw_agents(&agents);
            renderer::draw_hud(&mode, agents.len(), &benchmark_message); // Passa a msg
            next_frame().await; // Espera o frame ser desenhado

            // Executa o benchmark (dá uma travada)
            benchmark_message = run_benchmark();
        }

        // --- 2. Lógica de Input (Mouse) ---
        match mode {
            InputMode::DrawObstacle => {
                // Desenhar Obstáculo (clique esquerdo)
                if is_mouse_button_down(MouseButton::Left) {
                    grid.set_cell(grid_x, grid_y, CellType::Obstacle);
                }
                // Apagar Obstáculo (clique direito)
                if is_mouse_button_down(MouseButton::Right) {
                    grid.set_cell(grid_x, grid_y, CellType::Empty);
                }
            }
            InputMode::SetStart => {
                // Define o ponto inicial se não for um obstáculo
                if is_mouse_button_pressed(MouseButton::Left) && !grid.is_obstacle(grid_x, grid_y) {
                    pending_start = Some((grid_x, grid_y));
                    mode = InputMode::SetEnd; // Progride para o próximo estado
                    println!(
                        "Início definido em {:?}. Clique no Destino.",
                        (grid_x, grid_y)
                    );
                }
            }
            InputMode::SetEnd => {
                // Define o ponto final se não for um obstáculo
                if is_mouse_button_pressed(MouseButton::Left) && !grid.is_obstacle(grid_x, grid_y) {
                    if let Some(start_pos) = pending_start {
                        let end_pos = (grid_x, grid_y);
                        println!("Buscando caminho de {:?} para {:?}", start_pos, end_pos);

                        // Calcula o caminho usando A*
                        if let Some(path_nodes) = a_star_search(&grid, start_pos, end_pos) {
                            println!("Caminho encontrado! ({} nós)", path_nodes.len());

                            // Converte o caminho de nós do grid (usize) para posições de pixel (Vec2)
                            let pixel_path =
                                path_nodes.into_iter().map(grid_to_screen_center).collect();

                            // Cria o agente
                            let start_pixel_pos = grid_to_screen_center(start_pos);
                            agents.push(Agent::new(start_pixel_pos, pixel_path, AGENT_SPEED));
                        } else {
                            println!("Nenhum caminho encontrado.");
                        }

                        // Reseta para o próximo agente
                        mode = InputMode::SetStart;
                        pending_start = None;
                    }
                }
            }
        }

        // --- 3. Lógica de Update ---
        // Atualiza a posição de todos os agentes
        for agent in &mut agents {
            agent.update(dt);
        }

        // --- 4. Renderização ---
        clear_background(Color::from_hex(0x111111)); // Fundo cinza escuro

        // Desenha os elementos da simulação
        renderer::draw_grid(GRID_WIDTH, GRID_HEIGHT, CELL_SIZE);
        renderer::draw_cells(&grid, CELL_SIZE);
        renderer::draw_agents(&agents);

        // Desenha feedback visual para o modo de entrada (cursor do mouse)
        renderer::draw_input_feedback(
            &mode,
            pending_start,
            (grid_x, grid_y),
            CELL_SIZE,
            grid.is_obstacle(grid_x, grid_y),
        );

        // Desenha a Interface (HUD)
        renderer::draw_hud(&mode, agents.len(), &benchmark_message);

        // Avança para o próximo frame
        next_frame().await;
    }
}
