use macroquad::prelude::*;

// --- Módulos do Projeto ---
mod agent; // Define a lógica do Agente (movimento)
mod benchmark;
mod grid; // Define a estrutura do Grid (células, obstáculos)
mod pathfinding; // Implementa o algoritmo A* (A-Star)
mod renderer; // Contém todas as funções de desenho

// --- Módulos de Fábrica ---
mod abstract_factory;
mod agent_factory;
mod grid_factory;
mod pathfinding_factory;

use agent::Agent;
use benchmark::run_benchmark;
use grid::{CellType, Grid};

// Importa os novos tipos e contratos de fábrica
use abstract_factory::{CardinalSimulationFactory, SimulationFactory};
use pathfinding_factory::PathfindingAlgorithm;

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
fn screen_to_grid(x: f32, y: f32) -> (usize, usize) {
    (
        (x / CELL_SIZE).floor() as usize,
        (y / CELL_SIZE).floor() as usize,
    )
}

/// Converte coordenadas do grid para a posição central do pixel (para o Agente).
fn grid_to_screen_center(pos: (usize, usize)) -> Vec2 {
    vec2(
        pos.0 as f32 * CELL_SIZE + CELL_SIZE / 2.0,
        pos.1 as f32 * CELL_SIZE + CELL_SIZE / 2.0,
    )
}

/// Gera `n` agentes com posições e destinos aleatórios.
/// Usa PathfindingAlgorithm e AgentFactory para criar os componentes.
fn spawn_random_agents(
    n: usize,
    grid: &Grid,
    agents: &mut Vec<Agent>,
    pathfinder: &dyn PathfindingAlgorithm,
    agent_creator: &dyn agent_factory::AgentFactory,
) {
    let mut count = 0;
    for _ in 0..n {
        if let (Some(start_pos), Some(end_pos)) =
            (grid.get_random_empty_cell(), grid.get_random_empty_cell())
        {
            // Usa o pathfinder da fábrica para calcular o caminho
            if let Some(path_nodes) = pathfinder.find_path(grid, start_pos, end_pos) {
                let pixel_path = path_nodes.into_iter().map(grid_to_screen_center).collect();
                let start_pixel_pos = grid_to_screen_center(start_pos);

                // Usa a AgentFactory para criar o agente
                agents.push(agent_creator.create_agent(start_pixel_pos, pixel_path, AGENT_SPEED));
                count += 1;
            }
        }
    }
    println!("Gerado {} agentes aleatórios.", count);
}

/// Configurações da janela do Macroquad.
fn window_conf() -> Conf {
    Conf {
        window_title: "Trabalho 6 - FÁBRICAS - Pathfinding A*".to_owned(),
        window_width: (GRID_WIDTH as f32 * CELL_SIZE) as i32,
        window_height: (GRID_HEIGHT as f32 * CELL_SIZE + 50.0) as i32,
        fullscreen: false,
        sample_count: 8,
        ..Default::default()
    }
}

/// Ponto de entrada principal da aplicação.
#[macroquad::main(window_conf)]
async fn main() {
    // --- 0. Inicialização da Fábrica Abstrata ---
    // Instancia a Fábrica Abstrata (o conjunto de componentes 4-direções)
    let factory = CardinalSimulationFactory::new();

    // Recupera os componentes de suas respectivas fábricas
    let pathfinder = factory.create_pathfinder(); // Box<dyn PathfindingAlgorithm>
    let blue_agent_creator = factory.create_blue_agent_factory(); // Box<dyn AgentFactory>
    let red_agent_creator = factory.create_red_agent_factory(); // Box<dyn AgentFactory>

    // --- 1. Estado da Aplicação ---
    // Cria o Grid usando a Fábrica Abstrata (que delega ao GridFactory)
    let mut grid = factory.create_grid(GRID_WIDTH, GRID_HEIGHT);
    let mut agents: Vec<Agent> = Vec::new();
    let mut mode = InputMode::DrawObstacle;
    let mut pending_start: Option<(usize, usize)> = None; // Posição de início pendente
    let mut benchmark_message = String::new();

    // --- Loop Principal ---
    loop {
        let dt = get_frame_time(); // Delta Time
        let (mouse_x, mouse_y) = mouse_position();
        let (grid_x, grid_y) = screen_to_grid(mouse_x, mouse_y);

        // --- 1. Lógica de Input (Teclado) ---
        // [O] - Desenhar Obstáculos
        if is_key_pressed(KeyCode::O) {
            mode = InputMode::DrawObstacle;
            pending_start = None;
            println!("Modo: Desenhar Obstáculos");
        }

        // [A] - Definir Agente (Start/End)
        if is_key_pressed(KeyCode::A) {
            mode = InputMode::SetStart;
            pending_start = None;
            println!("Modo: Definir Ponto Inicial do Agente");
        }

        // [C] - Limpar Grid e Agentes
        if is_key_pressed(KeyCode::C) {
            grid.clear();
            agents.clear();
            pending_start = None;
            benchmark_message.clear();
            println!("Grid e Agentes limpos.");
        }

        // [R] - Gerar Agentes Aleatórios (Random)
        if is_key_pressed(KeyCode::R) {
            spawn_random_agents(
                20,
                &grid,
                &mut agents,
                pathfinder.as_ref(),        // Passa o pathfinder
                red_agent_creator.as_ref(), // Passa o agent_creator
            );
            benchmark_message.clear();
        }

        // [B] - Executar Benchmark
        if is_key_pressed(KeyCode::B) {
            // Renderiza a tela antes de rodar o benchmark (que é longo)
            next_frame().await;

            // Executa o benchmark (passando o pathfinder da Fábrica)
            benchmark_message = run_benchmark(pathfinder.as_ref());
        }

        // --- 2. Lógica de Input (Mouse) ---
        match mode {
            InputMode::DrawObstacle => {
                // Desenha obstáculos se o clique estiver dentro do grid
                if is_mouse_button_down(MouseButton::Left)
                    && grid_x < GRID_WIDTH
                    && grid_y < GRID_HEIGHT
                {
                    grid.set_cell(grid_x, grid_y, CellType::Obstacle);
                }
            }

            InputMode::SetStart => {
                // Define o ponto inicial se não for um obstáculo
                if is_mouse_button_pressed(MouseButton::Left) && !grid.is_obstacle(grid_x, grid_y) {
                    pending_start = Some((grid_x, grid_y));
                    mode = InputMode::SetEnd;
                    println!(
                        "Ponto inicial ({}, {}) definido. Clique no destino.",
                        grid_x, grid_y
                    );
                }
            }

            InputMode::SetEnd => {
                // Define o ponto final se não for um obstáculo
                if is_mouse_button_pressed(MouseButton::Left) && !grid.is_obstacle(grid_x, grid_y) {
                    if let Some(start_pos) = pending_start {
                        let end_pos = (grid_x, grid_y);
                        println!(
                            "Calculando caminho de ({}, {}) para ({}, {})",
                            start_pos.0, start_pos.1, end_pos.0, end_pos.1
                        );

                        // Usa o pathfinder da Fábrica
                        if let Some(path_nodes) = pathfinder.find_path(&grid, start_pos, end_pos) {
                            println!("Caminho encontrado com {} passos.", path_nodes.len());

                            let pixel_path =
                                path_nodes.into_iter().map(grid_to_screen_center).collect();

                            // Cria o agente usando a AgentFactory
                            let start_pixel_pos = grid_to_screen_center(start_pos);
                            agents.push(blue_agent_creator.create_agent(
                                start_pixel_pos,
                                pixel_path,
                                AGENT_SPEED,
                            ));
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

        next_frame().await
    }
}
