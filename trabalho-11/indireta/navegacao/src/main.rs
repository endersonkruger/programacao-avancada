use macroquad::prelude::*;

// --- Módulos do Projeto ---
mod agent;
mod benchmark;
mod grid;
mod renderer;
mod pheromone;

// --- Módulos de Fábrica ---
mod abstract_factory;
mod agent_factory;
mod grid_factory;

// --- Módulos do Decorator ---
mod agent_decorator;

// --- Singleton e Adapter ---
mod grid_adapter;
mod path_manager;
mod pathfinding_adapter;

// --- Renderização Hexagonal ---
mod hexagonal_renderer;

// --- Command, CoR, Observer ---
mod command;
mod initialization;
mod observer;

use agent_decorator::{
    AgentComponent, DirectionDeviateDecorator, SpeedBoostDecorator, VisualAlertDecorator, IndirectCommunicationDecorator
};
use grid::{CellType, Grid};
use grid_adapter::{HexagonalAdapter, RectangularCardinalAdapter, RectangularDiagonalAdapter};
use path_manager::PathManager;
use pathfinding_adapter::a_star_with_adapter;
use command::{CommandManager, MoveCommand};
use initialization::init_system;
use observer::{RespawnHandler};
use pheromone::PheromoneManager;

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

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum GridMode {
    Cardinal,
    Diagonal,
    Hexagonal,
}

pub fn screen_to_grid(x: f32, y: f32, grid_mode: GridMode) -> (usize, usize) {
    match grid_mode {
        GridMode::Hexagonal => hexagonal_renderer::hex_screen_to_grid(x, y),
        _ => (
            (x / CELL_SIZE).floor() as usize,
            (y / CELL_SIZE).floor() as usize,
        ),
    }
}

fn grid_to_screen_center(pos: (usize, usize), grid_mode: GridMode) -> Vec2 {
    match grid_mode {
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
    let path_manager = PathManager::instance();

    path_manager.get_or_calculate(start, end, || match grid_mode {
        GridMode::Cardinal => {
            let adapter = RectangularCardinalAdapter::new(grid);
            a_star_with_adapter(&adapter, start, end)
        }
        GridMode::Diagonal => {
            let adapter = RectangularDiagonalAdapter::new(grid);
            a_star_with_adapter(&adapter, start, end)
        }
        GridMode::Hexagonal => {
            let adapter = HexagonalAdapter::new(grid, true);
            a_star_with_adapter(&adapter, start, end)
        }
    })
}

/// Helper para criar a stack de Decorators de um agente
fn create_agent_stack(
    factory: &dyn agent_factory::AgentFactory,
    start: Vec2,
    path: Vec<Vec2>,
    id: usize,
    grid_mode: GridMode
) -> Box<dyn AgentComponent> {
    // 1. Cria Agente Base
    let base = factory.create_agent(start, path, AGENT_SPEED, id);
    
    // 2. Decorator de Comunicação Indireta
    let comm_agent = IndirectCommunicationDecorator::new(Box::new(base), grid_mode);

    // 3. Decorator de Desvio de Direção
    let dir_agent = DirectionDeviateDecorator::new(Box::new(comm_agent));

    // 4. Decorator de Velocidade
    let speed_agent = SpeedBoostDecorator::new(Box::new(dir_agent), 2.0);

    // 5. Decorator Visual
    let mut visual_agent = VisualAlertDecorator::new(Box::new(speed_agent));

    // 6. Observer
    visual_agent.add_observer(Box::new(RespawnHandler));

    Box::new(visual_agent)
}

fn spawn_random_agents(
    n: usize,
    grid: &Grid,
    agents: &mut Vec<Box<dyn AgentComponent>>,
    agent_creator: &dyn agent_factory::AgentFactory,
    grid_mode: GridMode,
    next_id: &mut usize,
) {
    let mut count = 0;
    for _ in 0..n {
        if let (Some(start_pos), Some(end_pos)) =
            (grid.get_random_empty_cell(), grid.get_random_empty_cell())
        {
            if let Some(path_nodes) = calculate_path(grid, start_pos, end_pos, grid_mode) {
                let pixel_path = path_nodes
                    .into_iter()
                    .map(|pos| grid_to_screen_center(pos, grid_mode))
                    .collect();
                let start_pixel_pos = grid_to_screen_center(start_pos, grid_mode);

                agents.push(create_agent_stack(agent_creator, start_pixel_pos, pixel_path, *next_id, grid_mode));
                *next_id += 1;
                count += 1;
            }
        }
    }
    println!("Gerado {} agentes aleatórios", count);
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Trabalho 11 - Comunicação Indireta".to_owned(),
        window_width: (GRID_WIDTH as f32 * CELL_SIZE) as i32,
        window_height: (GRID_HEIGHT as f32 * CELL_SIZE + 100.0) as i32,
        fullscreen: false,
        sample_count: 8,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut benchmark_manager = benchmark::BenchmarkManager::new();
    let init_ctx = init_system(GRID_WIDTH, GRID_HEIGHT);

    let mut grid = init_ctx.grid.expect("Grid error");
    let factory = init_ctx.factory.expect("Factory error");

    let blue_agent_creator = factory.create_blue_agent_factory();
    let red_agent_creator = factory.create_red_agent_factory();
    
    // --- INICIALIZA COMUNICAÇÃO INDIRETA ---
    PheromoneManager::instance().init(GRID_WIDTH, GRID_HEIGHT);

    let mut command_manager = CommandManager::new();
    let mut agents: Vec<Box<dyn AgentComponent>> = Vec::new();
    let mut mode = InputMode::DrawObstacle;
    let mut grid_mode = GridMode::Cardinal;
    let mut pending_start: Option<(usize, usize)> = None;
    let mut benchmark_message = String::new();
    let mut next_agent_id: usize = 0;
    
    let mut show_pheromones = true;

    loop {
        let dt = get_frame_time();
        let (mouse_x, mouse_y) = mouse_position();
        let (grid_x, grid_y) = screen_to_grid(mouse_x, mouse_y, grid_mode);

        // --- Atualiza Decaimento dos Feromônios ---
        PheromoneManager::instance().update(dt);

        if is_key_pressed(KeyCode::O) { mode = InputMode::DrawObstacle; pending_start = None; }
        if is_key_pressed(KeyCode::A) { mode = InputMode::SetStart; pending_start = None; }
        if is_key_pressed(KeyCode::C) {
            grid.clear();
            agents.clear();
            PheromoneManager::instance().clear();
            pending_start = None;
            benchmark_message.clear();
            PathManager::instance().clear_cache();
            next_agent_id = 0;
        }
        if is_key_pressed(KeyCode::R) {
            spawn_random_agents(20, &grid, &mut agents, red_agent_creator.as_ref(), grid_mode, &mut next_agent_id);
        }
        if is_key_pressed(KeyCode::P) {
            show_pheromones = !show_pheromones;
        }
        if is_key_pressed(KeyCode::G) {
            grid_mode = match grid_mode {
                GridMode::Cardinal => GridMode::Diagonal,
                GridMode::Diagonal => GridMode::Hexagonal,
                GridMode::Hexagonal => GridMode::Cardinal,
            };
            PathManager::instance().clear_cache();
            PheromoneManager::instance().clear();
        }
        if is_key_pressed(KeyCode::Z) { command_manager.undo_last(&mut agents); }

        /// --- BENCHMARKS (TECLAS 1, 2, 3) ---
        if is_key_pressed(KeyCode::Key1) {
            grid.clear();
            agents.clear();
            PheromoneManager::instance().clear(); // Limpa feromônios antigos
            benchmark::spawn_opposing_rows(
                &grid,
                &mut agents,
                blue_agent_creator.as_ref(),
                grid_mode,
                &mut next_agent_id,
            );
            benchmark_manager.start_test("1_Row_Opposing");
        }
        if is_key_pressed(KeyCode::Key2) {
            grid.clear();
            agents.clear();
            PheromoneManager::instance().clear();
            benchmark::spawn_double_opposing_rows(
                &grid,
                &mut agents,
                blue_agent_creator.as_ref(),
                grid_mode,
                &mut next_agent_id,
            );
            benchmark_manager.start_test("2_Rows_Opposing");
        }
        if is_key_pressed(KeyCode::Key3) {
            grid.clear();
            agents.clear();
            PheromoneManager::instance().clear();
            benchmark::spawn_random_scenario(
                &grid,
                &mut agents,
                blue_agent_creator.as_ref(),
                grid_mode,
                &mut next_agent_id,
                100,
            );
            benchmark_manager.start_test("Random_100");
        }

        // --- Inputs Mouse ---
        match mode {
            InputMode::DrawObstacle => {
                if is_mouse_button_down(MouseButton::Left) && grid_x < GRID_WIDTH && grid_y < GRID_HEIGHT {
                    grid.set_cell(grid_x, grid_y, CellType::Obstacle);
                    PathManager::instance().clear_cache();
                }
            }
            InputMode::SetStart => {
                if is_mouse_button_pressed(MouseButton::Left) && !grid.is_obstacle(grid_x, grid_y) {
                    pending_start = Some((grid_x, grid_y));
                    mode = InputMode::SetEnd;
                }
            }
            InputMode::SetEnd => {
                if is_mouse_button_pressed(MouseButton::Left) && !grid.is_obstacle(grid_x, grid_y) {
                    if let Some(start_pos) = pending_start {
                        let end_pos = (grid_x, grid_y);
                        if let Some(path_nodes) = calculate_path(&grid, start_pos, end_pos, grid_mode) {
                            let pixel_path = path_nodes.iter().map(|p| grid_to_screen_center(*p, grid_mode)).collect();
                            let start_screen = grid_to_screen_center(start_pos, grid_mode);
                            
                            agents.push(create_agent_stack(
                                blue_agent_creator.as_ref(), start_screen, pixel_path, next_agent_id, grid_mode
                            ));
                            next_agent_id += 1;
                        }
                        mode = InputMode::SetStart;
                        pending_start = None;
                    }
                }
            }
        }

        // 1. Atualiza agentes 
        for agent in &mut agents {
            agent.update(dt);
        }

        // 2. Gera Comandos de Movimento 
        for agent in &agents {
            if let Some(target_pos) = agent.get_next_step_target() {
                let current_pos = agent.get_pos();
                let move_cmd = MoveCommand::new(agent.get_id(), current_pos, target_pos);
                command_manager.add_command(Box::new(move_cmd));
            }
        }

        command_manager.process_commands(&mut agents);
        benchmark_manager.update(agents.len());

        // --- Renderização ---
        clear_background(Color::from_hex(0x111111));

        match grid_mode {
            GridMode::Hexagonal => {
                hexagonal_renderer::draw_hexagonal_grid(GRID_WIDTH, GRID_HEIGHT);
                if show_pheromones { renderer::draw_pheromones(GRID_WIDTH, GRID_HEIGHT, CELL_SIZE); } 
                hexagonal_renderer::draw_hexagonal_cells(&grid);
                hexagonal_renderer::draw_hexagonal_agents(&agents);
                hexagonal_renderer::draw_hexagonal_input_feedback(&mode, pending_start, (grid_x, grid_y), grid.is_obstacle(grid_x, grid_y));
            }
            _ => {
                renderer::draw_grid(GRID_WIDTH, GRID_HEIGHT, CELL_SIZE);
                if show_pheromones { renderer::draw_pheromones(GRID_WIDTH, GRID_HEIGHT, CELL_SIZE); }
                renderer::draw_cells(&grid, CELL_SIZE);
                renderer::draw_agents(&agents);
                renderer::draw_input_feedback(&mode, pending_start, (grid_x, grid_y), CELL_SIZE, grid.is_obstacle(grid_x, grid_y));
            }
        }

        draw_hud_extended(&mode, &grid_mode, agents.len(), &benchmark_message, show_pheromones);
        next_frame().await
    }
}

fn draw_hud_extended(
    mode: &InputMode,
    grid_mode: &GridMode,
    agent_count: usize,
    benchmark_msg: &str,
    show_pheromones: bool
) {
    let mode_text = format!("Modo: {:?}", mode);
    let grid_mode_text = format!("Grid: {:?}", grid_mode);
    let help_text = "[O] Obstáculo | [A] Agente | [R] Random | [C] Clear | [P] Feromônios";
    let status_text = format!("Agentes: {} | Feromônios Visíveis: {}", agent_count, show_pheromones);

    draw_text(help_text, 10.0, 25.0, 20.0, WHITE);
    draw_text(&mode_text, 10.0, 50.0, 24.0, YELLOW);
    draw_text(&grid_mode_text, 10.0, 75.0, 24.0, BLUE);
    draw_text(&status_text, 10.0, 100.0, 24.0, WHITE);

    if !benchmark_msg.is_empty() {
        draw_text(benchmark_msg, 10.0, 125.0, 20.0, GREEN);
    }
}