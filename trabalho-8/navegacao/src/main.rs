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

// --- Singleton e Adapter ---
mod grid_adapter; // ADAPTER
mod path_manager; // SINGLETON
mod pathfinding_adapter; // Pathfinding que usa o Adapter

// --- Renderização Hexagonal ---
mod hexagonal_renderer;

// --- Command, CoR, Observer ---
mod command;
mod initialization;
mod observer;

use agent_decorator::{AgentComponent, SpeedBoostDecorator};
use grid::{CellType, Grid};

use grid_adapter::{HexagonalAdapter, RectangularCardinalAdapter, RectangularDiagonalAdapter};
use path_manager::PathManager;
use pathfinding_adapter::a_star_with_adapter;

use command::{CommandManager, MoveCommand};
use initialization::init_system;
use observer::RespawnHandler;

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

// Tipo de grid selecionado
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum GridMode {
    Cardinal,  // 4 direções
    Diagonal,  // 8 direções
    Hexagonal, // 6 direções (hexagonal)
}

fn screen_to_grid(x: f32, y: f32, grid_mode: GridMode) -> (usize, usize) {
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

/// Gera agentes aleatórios (Usa Decorator Box e Observer)
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

                // 1. Cria Agente Base (Factory) - Passando ID
                let base_agent =
                    agent_creator.create_agent(start_pixel_pos, pixel_path, AGENT_SPEED, *next_id);

                // 2. Aplica Decorator (SpeedBoost)
                // Note o Box::new() envolvendo o base_agent
                let mut decorated_agent = SpeedBoostDecorator::new(Box::new(base_agent), 2.0);

                // 3. Adiciona Observer (RespawnHandler)
                decorated_agent.add_observer(Box::new(RespawnHandler));

                agents.push(Box::new(decorated_agent));

                *next_id += 1;
                count += 1;
            }
        }
    }
    println!(
        "Gerado {} agentes aleatórios com Speed Boost e Observers.",
        count
    );
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Trabalho 8 - Padrões de Projeto".to_owned(),
        window_width: (GRID_WIDTH as f32 * CELL_SIZE) as i32,
        window_height: (GRID_HEIGHT as f32 * CELL_SIZE + 100.0) as i32,
        fullscreen: false,
        sample_count: 8,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // --- 1. CHAIN OF RESPONSIBILITY: Inicialização ---
    // A inicialização da janela, do grid e da fábrica é feita em cadeia
    let init_ctx = init_system(GRID_WIDTH, GRID_HEIGHT);

    // Recupera os objetos criados pela Chain
    let mut grid = init_ctx
        .grid
        .expect("Grid não foi inicializado pela Chain!");
    let factory = init_ctx
        .factory
        .expect("Factory não foi inicializada pela Chain!");

    // Cria as fábricas de agentes a partir da Abstract Factory obtida
    let blue_agent_creator = factory.create_blue_agent_factory();
    let red_agent_creator = factory.create_red_agent_factory();

    // --- 2. COMMAND MANAGER ---
    let mut command_manager = CommandManager::new();

    let mut agents: Vec<Box<dyn AgentComponent>> = Vec::new();
    let mut mode = InputMode::DrawObstacle;
    let mut grid_mode = GridMode::Cardinal;
    let mut pending_start: Option<(usize, usize)> = None;
    let mut benchmark_message = String::new();

    // Controle de IDs para os agentes
    let mut next_agent_id: usize = 0;

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
            PathManager::instance().clear_cache();
            next_agent_id = 0;
            // Limpa histórico de comandos também seria ideal aqui, mas simplificamos
            println!("Grid e Agentes limpos.");
        }

        if is_key_pressed(KeyCode::R) {
            spawn_random_agents(
                20,
                &grid,
                &mut agents,
                red_agent_creator.as_ref(),
                grid_mode,
                &mut next_agent_id,
            );
            benchmark_message.clear();
        }

        if is_key_pressed(KeyCode::G) {
            grid_mode = match grid_mode {
                GridMode::Cardinal => GridMode::Diagonal,
                GridMode::Diagonal => GridMode::Hexagonal,
                GridMode::Hexagonal => GridMode::Cardinal,
            };
            PathManager::instance().clear_cache();
            println!("Modo do Grid: {:?}", grid_mode);
        }

        // --- NOVO: UNDO (Desfazer Movimento) ---
        if is_key_pressed(KeyCode::Z) {
            command_manager.undo_last(&mut agents);
        }

        // --- Input (Mouse) ---
        match mode {
            InputMode::DrawObstacle => {
                if is_mouse_button_down(MouseButton::Left)
                    && grid_x < GRID_WIDTH
                    && grid_y < GRID_HEIGHT
                {
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

                        if let Some(path_nodes) =
                            calculate_path(&grid, start_pos, end_pos, grid_mode)
                        {
                            let pixel_path = path_nodes
                                .into_iter()
                                .map(|pos| grid_to_screen_center(pos, grid_mode))
                                .collect();

                            // Cria agente manual com Observer
                            let base_agent = blue_agent_creator.create_agent(
                                grid_to_screen_center(start_pos, grid_mode),
                                pixel_path,
                                AGENT_SPEED,
                                next_agent_id,
                            );

                            // Adiciona observer diretamente (sem decorator de speed neste caso, ou com, conforme preferir)
                            // Para consistência, vamos usar Box mas sem decorator extra
                            let mut boxed_agent: Box<dyn AgentComponent> = Box::new(base_agent);
                            boxed_agent.add_observer(Box::new(RespawnHandler));

                            agents.push(boxed_agent);
                            next_agent_id += 1;
                        } else {
                            println!("Nenhum caminho encontrado.");
                        }

                        mode = InputMode::SetStart;
                        pending_start = None;
                    }
                }
            }
        }

        // --- UPDATE COM COMMAND PATTERN ---

        // 1. Atualiza lógica interna dos agentes (sem mover a posição ainda)
        for agent in &mut agents {
            agent.update(dt);
        }

        // 2. Gera Comandos de Movimento
        // O agente decide para onde quer ir (get_next_step_target), e o CommandManager executa.
        for agent in &agents {
            // Se o agente tem uma intenção de movimento
            if let Some(target_pos) = agent.get_next_step_target() {
                let current_pos = agent.get_pos();
                let id = agent.get_id();

                // Cria o comando (contém o timestamp e dados para undo)
                let move_cmd = MoveCommand::new(id, current_pos, target_pos);

                // Enfileira para execução
                command_manager.add_command(Box::new(move_cmd));
            }
        }

        // 3. Executa os Comandos da Fila
        // Isso efetivamente altera a posição dos agentes (set_pos) e salva no histórico
        command_manager.process_commands(&mut agents);

        // --- Renderização ---
        clear_background(Color::from_hex(0x111111));

        match grid_mode {
            GridMode::Hexagonal => {
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

        draw_hud_extended(&mode, &grid_mode, agents.len(), &benchmark_message);

        next_frame().await
    }
}

/// HUD estendido
fn draw_hud_extended(
    mode: &InputMode,
    grid_mode: &GridMode,
    agent_count: usize,
    benchmark_msg: &str,
) {
    let mode_text = format!("Modo: {:?}", mode);
    let grid_mode_text = format!("Grid: {:?}", grid_mode);
    let help_text = "[O] Obst | [A] Agente | [R] Random | [C] Clear | [G] Grid | [Z] Undo";
    let agent_text = format!("Agentes: {}", agent_count);

    draw_text(help_text, 10.0, 25.0, 20.0, WHITE);
    draw_text(&mode_text, 10.0, 50.0, 24.0, YELLOW);
    draw_text(&grid_mode_text, 10.0, 75.0, 24.0, BLUE);
    draw_text(&agent_text, 10.0, 100.0, 24.0, WHITE);

    if !benchmark_msg.is_empty() {
        draw_text(benchmark_msg, 10.0, 125.0, 20.0, GREEN);
    }
}
