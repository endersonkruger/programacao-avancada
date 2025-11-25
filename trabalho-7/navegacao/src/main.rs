use macroquad::prelude::*;

// --- Módulos do Projeto ---
mod agent;
mod benchmark;
mod grid;
mod renderer;

// --- Módulos de Fábrica ---
mod abstract_factory;
mod agent_factory;
mod grid_factory;

// --- Módulos do Decorator ---
mod agent_decorator;

// --- NOVOS MÓDULOS: Singleton e Adapter ---
mod grid_adapter; // ADAPTER
mod path_manager; // SINGLETON
mod pathfinding_adapter; // Pathfinding que usa o Adapter

// --- NOVO: Renderização Hexagonal ---
mod hexagonal_renderer;

use abstract_factory::{CardinalSimulationFactory, SimulationFactory};
use agent_decorator::{AgentComponent, SpeedBoostDecorator};
use grid::{CellType, Grid};

// NOVOS IMPORTS
use grid_adapter::{HexagonalAdapter, RectangularCardinalAdapter, RectangularDiagonalAdapter};
use path_manager::PathManager;
use pathfinding_adapter::a_star_with_adapter;

// --- Constantes da Simulação ---
const CELL_SIZE: f32 = 20.0;
const GRID_WIDTH: usize = 60;
const GRID_HEIGHT: usize = 36;
const AGENT_SPEED: f32 = 150.0;

#[derive(PartialEq, Debug)]
enum InputMode {
    DrawObstacle,
    SetStart,
    SetEnd,
}

// Novo: Tipo de grid selecionado
#[derive(PartialEq, Debug, Clone, Copy)]
enum GridMode {
    Cardinal4, // 4 direções
    Diagonal8, // 8 direções
    Hexagonal, // 6 direções (hexagonal)
}

fn screen_to_grid(x: f32, y: f32, grid_mode: GridMode) -> (usize, usize) {
    match grid_mode {
        // Usa a função corrigida
        GridMode::Hexagonal => hexagonal_renderer::hex_screen_to_grid(x, y),
        _ => (
            (x / CELL_SIZE).floor() as usize,
            (y / CELL_SIZE).floor() as usize,
        ),
    }
}

fn grid_to_screen_center(pos: (usize, usize), grid_mode: GridMode) -> Vec2 {
    match grid_mode {
        // Usa a função corrigida
        GridMode::Hexagonal => hexagonal_renderer::hex_grid_to_screen(pos),
        _ => vec2(
            pos.0 as f32 * CELL_SIZE + CELL_SIZE / 2.0,
            pos.1 as f32 * CELL_SIZE + CELL_SIZE / 2.0,
        ),
    }
}

/// Helper: Calcula caminho usando Adapter e Singleton
fn calculate_path(
    grid: &Grid,
    start: (usize, usize),
    end: (usize, usize),
    grid_mode: GridMode,
) -> Option<Vec<(usize, usize)>> {
    // Obtém instância do PathManager (SINGLETON)
    let path_manager = PathManager::instance();

    // Busca no cache ou calcula
    path_manager.get_or_calculate(start, end, || {
        // Cria o Adapter apropriado
        match grid_mode {
            GridMode::Cardinal4 => {
                let adapter = RectangularCardinalAdapter::new(grid);
                a_star_with_adapter(&adapter, start, end)
            }
            GridMode::Diagonal8 => {
                let adapter = RectangularDiagonalAdapter::new(grid);
                a_star_with_adapter(&adapter, start, end)
            }
            // Usa o HexagonalAdapter com flat_top=true, coerente com o renderer
            GridMode::Hexagonal => {
                let adapter = HexagonalAdapter::new(grid, true);
                a_star_with_adapter(&adapter, start, end)
            }
        }
    })
}

/// Gera agentes aleatórios usando o novo sistema
fn spawn_random_agents(
    n: usize,
    grid: &Grid,
    agents: &mut Vec<Box<dyn AgentComponent>>,
    agent_creator: &dyn agent_factory::AgentFactory,
    grid_mode: GridMode,
) {
    let mut count = 0;
    for _ in 0..n {
        if let (Some(start_pos), Some(end_pos)) =
            (grid.get_random_empty_cell(), grid.get_random_empty_cell())
        {
            // USA O NOVO SISTEMA: Adapter + Singleton
            if let Some(path_nodes) = calculate_path(grid, start_pos, end_pos, grid_mode) {
                let pixel_path = path_nodes
                    .into_iter()
                    .map(|pos| grid_to_screen_center(pos, grid_mode))
                    .collect();
                let start_pixel_pos = grid_to_screen_center(start_pos, grid_mode);

                let base_agent =
                    agent_creator.create_agent(start_pixel_pos, pixel_path, AGENT_SPEED);

                let decorated_agent = SpeedBoostDecorator::new(base_agent, 2.0);
                agents.push(Box::new(decorated_agent));

                count += 1;
            }
        }
    }
    println!("Gerado {} agentes aleatórios com Speed Boost (2x).", count);
}

fn window_conf() -> Conf {
    // Aumenta a altura da janela para acomodar o HUD
    Conf {
        window_title: "Trabalho 7 - Singleton + Adapter".to_owned(),
        window_width: (GRID_WIDTH as f32 * CELL_SIZE) as i32,
        window_height: (GRID_HEIGHT as f32 * CELL_SIZE + 100.0) as i32,
        fullscreen: false,
        sample_count: 8,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let factory = CardinalSimulationFactory::new();
    let blue_agent_creator = factory.create_blue_agent_factory();
    let red_agent_creator = factory.create_red_agent_factory();

    let mut grid = factory.create_grid(GRID_WIDTH, GRID_HEIGHT);
    let mut agents: Vec<Box<dyn AgentComponent>> = Vec::new();
    let mut mode = InputMode::DrawObstacle;
    let mut grid_mode = GridMode::Cardinal4; // NOVO: modo do grid
    let mut pending_start: Option<(usize, usize)> = None;
    let mut benchmark_message = String::new();

    loop {
        let dt = get_frame_time();
        let (mouse_x, mouse_y) = mouse_position();
        let (grid_x, grid_y) = screen_to_grid(mouse_x, mouse_y, grid_mode);

        // --- Input (Teclado) ---

        if is_key_pressed(KeyCode::O) {
            mode = InputMode::DrawObstacle;
            pending_start = None;
            println!("Modo: Desenhar Obstáculos");
        }

        if is_key_pressed(KeyCode::A) {
            mode = InputMode::SetStart;
            pending_start = None;
            println!("Modo: Definir Ponto Inicial do Agente");
        }

        if is_key_pressed(KeyCode::C) {
            grid.clear();
            agents.clear();
            pending_start = None;
            benchmark_message.clear();

            // LIMPA O CACHE DO SINGLETON
            PathManager::instance().clear_cache();

            println!("Grid, Agentes e Cache limpos.");
        }

        if is_key_pressed(KeyCode::R) {
            spawn_random_agents(
                20,
                &grid,
                &mut agents,
                red_agent_creator.as_ref(),
                grid_mode, // Passa o modo atual
            );
            benchmark_message.clear();
        }

        // NOVO: [G] - Trocar modo do Grid (4-dir <-> 8-dir <-> Hexagonal)
        if is_key_pressed(KeyCode::G) {
            grid_mode = match grid_mode {
                GridMode::Cardinal4 => GridMode::Diagonal8,
                GridMode::Diagonal8 => GridMode::Hexagonal,
                GridMode::Hexagonal => GridMode::Cardinal4,
            };

            // Limpa cache ao trocar modo
            PathManager::instance().clear_cache();

            println!("Modo do Grid: {:?}", grid_mode);
        }

        // --- Input (Mouse) ---
        match mode {
            InputMode::DrawObstacle => {
                if is_mouse_button_down(MouseButton::Left)
                    && grid_x < GRID_WIDTH
                    && grid_y < GRID_HEIGHT
                {
                    grid.set_cell(grid_x, grid_y, CellType::Obstacle);

                    // Quando o grid muda, limpa o cache
                    PathManager::instance().clear_cache();
                }
            }

            InputMode::SetStart => {
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
                if is_mouse_button_pressed(MouseButton::Left) && !grid.is_obstacle(grid_x, grid_y) {
                    if let Some(start_pos) = pending_start {
                        let end_pos = (grid_x, grid_y);

                        // USA O NOVO SISTEMA
                        if let Some(path_nodes) =
                            calculate_path(&grid, start_pos, end_pos, grid_mode)
                        {
                            let pixel_path = path_nodes
                                .into_iter()
                                .map(|pos| grid_to_screen_center(pos, grid_mode))
                                .collect();

                            let base_agent = blue_agent_creator.create_agent(
                                grid_to_screen_center(start_pos, grid_mode),
                                pixel_path,
                                AGENT_SPEED,
                            );

                            agents.push(Box::new(base_agent));
                        } else {
                            println!("Nenhum caminho encontrado.");
                        }

                        mode = InputMode::SetStart;
                        pending_start = None;
                    }
                }
            }
        }

        // --- Update ---
        for agent in &mut agents {
            agent.update(dt);
        }

        // --- Renderização ---
        clear_background(Color::from_hex(0x111111));

        // Renderiza de acordo com o modo do grid
        match grid_mode {
            GridMode::Hexagonal => {
                // Renderização hexagonal
                hexagonal_renderer::draw_hexagonal_grid(GRID_WIDTH, GRID_HEIGHT);
                hexagonal_renderer::draw_hexagonal_cells(&grid);
                hexagonal_renderer::draw_hexagonal_agents(&agents);
                hexagonal_renderer::draw_hexagonal_input_feedback(
                    &mode,
                    pending_start,
                    (grid_x, grid_y),
                    grid.is_obstacle(grid_x, grid_y),
                );
            }
            _ => {
                // Renderização retangular normal
                renderer::draw_grid(GRID_WIDTH, GRID_HEIGHT, CELL_SIZE);
                renderer::draw_cells(&grid, CELL_SIZE);
                renderer::draw_agents(&agents);
                renderer::draw_input_feedback(
                    &mode,
                    pending_start,
                    (grid_x, grid_y),
                    CELL_SIZE,
                    grid.is_obstacle(grid_x, grid_y),
                );
            }
        }

        // HUD atualizado
        draw_hud_extended(&mode, &grid_mode, agents.len(), &benchmark_message);

        next_frame().await
    }
}

/// HUD estendido com informações do GridMode
fn draw_hud_extended(
    mode: &InputMode,
    grid_mode: &GridMode,
    agent_count: usize,
    benchmark_msg: &str,
) {
    let mode_text = format!("Modo: {:?}", mode);
    let grid_mode_text = format!("Grid: {:?}", grid_mode);
    let help_text = "[O] Obstáculos | [A] Agente | [R] Aleatórios | [C] Limpar | [G] Trocar Grid";
    let agent_text = format!("Agentes: {}", agent_count);

    draw_text(help_text, 10.0, 25.0, 20.0, WHITE);
    draw_text(&mode_text, 10.0, 50.0, 24.0, YELLOW);
    draw_text(&grid_mode_text, 10.0, 75.0, 24.0, BLUE);
    draw_text(&agent_text, 10.0, 100.0, 24.0, WHITE);

    if !benchmark_msg.is_empty() {
        draw_text(benchmark_msg, 10.0, 125.0, 20.0, GREEN);
    }
}
