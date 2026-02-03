use crate::agent_decorator::{
    AgentComponent, DirectionDeviateDecorator, SpeedBoostDecorator, VisualAlertDecorator,
    IndirectCommunicationDecorator
};
use crate::agent_factory::AgentFactory;
use crate::grid::Grid;
use crate::grid_adapter::{
    HexagonalAdapter, RectangularCardinalAdapter, RectangularDiagonalAdapter,
};
use crate::path_manager::PathManager;
use crate::pathfinding_adapter::a_star_with_adapter;
use crate::{CELL_SIZE, GridMode};
use macroquad::prelude::*;
use std::fs::OpenOptions;
use std::io::Write;

/// Gerenciador de Benchmark
pub struct BenchmarkManager {
    current_test_name: String,
    is_recording: bool,
    frame_data: Vec<FrameRecord>,
    start_time: f64,
}

struct FrameRecord {
    time_elapsed: f64,
    fps: i32,
    agent_count: usize,
}

impl BenchmarkManager {
    pub fn new() -> Self {
        Self {
            current_test_name: "None".to_string(),
            is_recording: false,
            frame_data: Vec::new(),
            start_time: 0.0,
        }
    }

    /// Inicia a gravação de dados para um cenário
    pub fn start_test(&mut self, test_name: &str) {
        self.current_test_name = test_name.to_string();
        self.is_recording = true;
        self.frame_data.clear();
        self.start_time = get_time();
        println!(">>> INICIANDO BENCHMARK: {}", test_name);
    }

    /// Deve ser chamado a cada frame no main loop
    pub fn update(&mut self, agent_count: usize) {
        if self.is_recording {
            let fps = get_fps();
            let time = get_time() - self.start_time;

            self.frame_data.push(FrameRecord {
                time_elapsed: time,
                fps,
                agent_count,
            });

            // Desenha indicador na tela
            draw_text(
                &format!("REC: {} ({:.1}s)", self.current_test_name, time),
                10.0,
                200.0,
                30.0,
                RED,
            );

            // Para automaticamente após 15 segundos
            if time > 15.0 {
                self.save_and_stop();
            }
        }
    }

    /// Salva os dados no CSV e para a gravação
    pub fn save_and_stop(&mut self) {
        if !self.is_recording {
            return;
        }

        self.is_recording = false;
        self.save_to_csv();
        println!(
            "<<< FIM DO BENCHMARK: {}. Resultados salvos em benchmark_results.csv",
            self.current_test_name
        );
    }

    fn save_to_csv(&self) {
        let filename = "benchmark_results.csv";

        // Abre arquivo em modo Append ou Cria novo
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(filename)
            .expect("Falha ao abrir arquivo de benchmark");

        // Escreve os dados: Teste, Tempo, FPS, Agentes
        for record in &self.frame_data {
            if let Err(e) = writeln!(
                file,
                "{}, {:.4}, {}, {}",
                self.current_test_name, record.time_elapsed, record.fps, record.agent_count
            ) {
                eprintln!("Erro ao escrever no CSV: {}", e);
            }
        }
    }

    pub fn is_running(&self) -> bool {
        self.is_recording
    }
}

// --- FUNÇÕES GERADORAS DE CENÁRIOS ---

/// Cenário 1: Uma fileira inteira de cada lado indo para o lado oposto
pub fn spawn_opposing_rows(
    grid: &Grid,
    agents: &mut Vec<Box<dyn AgentComponent>>,
    factory: &dyn AgentFactory,
    grid_mode: GridMode,
    next_id: &mut usize,
) {
    let rows_count = 1; // 1 Fileira de cada lado
    spawn_lanes(
        grid,
        agents,
        factory,
        grid_mode,
        next_id,
        rows_count,
        "1_Row_Vs_1_Row",
    );
}

/// Cenário 2: Duas fileiras inteiras de cada lado
pub fn spawn_double_opposing_rows(
    grid: &Grid,
    agents: &mut Vec<Box<dyn AgentComponent>>,
    factory: &dyn AgentFactory,
    grid_mode: GridMode,
    next_id: &mut usize,
) {
    let rows_count = 2; // 2 Fileiras de cada lado
    spawn_lanes(
        grid,
        agents,
        factory,
        grid_mode,
        next_id,
        rows_count,
        "2_Rows_Vs_2_Rows",
    );
}

/// Lógica interna para criar fileiras opostas
fn spawn_lanes(
    grid: &Grid,
    agents: &mut Vec<Box<dyn AgentComponent>>,
    factory: &dyn AgentFactory,
    grid_mode: GridMode,
    next_id: &mut usize,
    rows_width: usize,
    _scenario_tag: &str,
) {
    let path_manager = PathManager::instance();
    let mut spawned = 0;

    for y in 0..grid.height {
        // Esquerda -> Direita (Azuis/Normais)
        for x_off in 0..rows_width {
            let start = (x_off, y);
            let end = (grid.width - 1 - x_off, y);
            spawn_single_agent(
                grid,
                agents,
                factory,
                grid_mode,
                next_id,
                start,
                end,
                path_manager,
            );
            spawned += 1;
        }

        for x_off in 0..rows_width {
            let start = (grid.width - 1 - x_off, y);
            let end = (x_off, y);
            spawn_single_agent(
                grid,
                agents,
                factory,
                grid_mode,
                next_id,
                start,
                end,
                path_manager,
            );
            spawned += 1;
        }
    }
    println!("Spawned {} agents in lanes.", spawned);
}

/// Cenário 3: Casos Aleatórios
pub fn spawn_random_scenario(
    grid: &Grid,
    agents: &mut Vec<Box<dyn AgentComponent>>,
    factory: &dyn AgentFactory,
    grid_mode: GridMode,
    next_id: &mut usize,
    count: usize,
) {
    let path_manager = PathManager::instance();
    let mut spawned = 0;
    let max_attempts = count * 10;
    let mut attempts = 0;

    while spawned < count && attempts < max_attempts {
        attempts += 1;
        if let (Some(start), Some(end)) =
            (grid.get_random_empty_cell(), grid.get_random_empty_cell())
        {
            if start != end {
                if spawn_single_agent(
                    grid,
                    agents,
                    factory,
                    grid_mode,
                    next_id,
                    start,
                    end,
                    path_manager,
                ) {
                    spawned += 1;
                }
            }
        }
    }
    println!("Spawned {} random agents.", spawned);
}

/// Helper para criar um único agente com a stack completa de Decorators
fn spawn_single_agent(
    grid: &Grid,
    agents: &mut Vec<Box<dyn AgentComponent>>,
    factory: &dyn AgentFactory,
    grid_mode: GridMode,
    next_id: &mut usize,
    start: (usize, usize),
    end: (usize, usize),
    path_manager: &PathManager,
) -> bool {
    // 1. Calcula Caminho
    let path_opt = path_manager.get_or_calculate(start, end, || match grid_mode {
        GridMode::Hexagonal => {
            let adapter = HexagonalAdapter::new(grid, true);
            a_star_with_adapter(&adapter, start, end)
        }
        GridMode::Diagonal => {
            let adapter = RectangularDiagonalAdapter::new(grid);
            a_star_with_adapter(&adapter, start, end)
        }
        GridMode::Cardinal => {
            let adapter = RectangularCardinalAdapter::new(grid);
            a_star_with_adapter(&adapter, start, end)
        }
    });

    if let Some(grid_path) = path_opt {
        // 2. Converte para Pixels
        let pixel_path: Vec<Vec2> = grid_path
            .into_iter()
            .map(|p| get_screen_pos(p, grid_mode))
            .collect();

        let start_pos = get_screen_pos(start, grid_mode);
        let speed = 150.0; // Velocidade base

        // 3. Cria Agente Base
        let base_agent = factory.create_agent(start_pos, pixel_path, speed, *next_id);

        // 4.1. Comunicação Indireta
        // Essencial para o benchmark testar a colisão nova
        let comm_agent = IndirectCommunicationDecorator::new(Box::new(base_agent), grid_mode);

        // 4.2. Desvio de Direção
        let direction_agent = DirectionDeviateDecorator::new(Box::new(comm_agent));
        
        // 4.3. Velocidade Reativa
        let speed_agent = SpeedBoostDecorator::new(Box::new(direction_agent), 2.0); 
        
        // 4.4. Alerta Visual
        let mut visual_agent = VisualAlertDecorator::new(Box::new(speed_agent));

        // Observer
        visual_agent.add_observer(Box::new(crate::observer::RespawnHandler));

        agents.push(Box::new(visual_agent));
        *next_id += 1;
        return true;
    }
    false
}

// Helper local para evitar dependência circular complexa com main
fn get_screen_pos(pos: (usize, usize), mode: GridMode) -> Vec2 {
    match mode {
        GridMode::Hexagonal => crate::hexagonal_renderer::hex_grid_to_screen(pos),
        _ => vec2(
            pos.0 as f32 * CELL_SIZE + CELL_SIZE / 2.0,
            pos.1 as f32 * CELL_SIZE + CELL_SIZE / 2.0,
        ),
    }
}