use crate::grid::{CellType, Grid};
use crate::pathfinding_factory::PathfindingAlgorithm;
use macroquad::prelude::rand;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::time::Instant;

/// Preenche o grid com obstáculos aleatórios baseados na densidade fornecida.
fn populate_obstacles(grid: &mut Grid, density: f32) {
    for y in 0..grid.height {
        for x in 0..grid.width {
            if rand::gen_range(0.0, 1.0) < density {
                grid.set_cell(x, y, CellType::Obstacle);
            }
        }
    }
}

/// Gera N tarefas (start, end) para os agentes.
fn generate_agent_tasks(grid: &Grid, n: usize) -> Vec<((usize, usize), (usize, usize))> {
    let mut tasks = Vec::with_capacity(n);
    for _ in 0..n {
        // Tenta encontrar um início e fim válidos (células não-obstáculo)
        if let (Some(start_pos), Some(end_pos)) =
            (grid.get_random_empty_cell(), grid.get_random_empty_cell())
        {
            tasks.push((start_pos, end_pos));
        }
    }
    tasks
}

/// Executa o benchmark de desempenho do algoritmo de pathfinding e salva os resultados em um CSV.
/// O algoritmo é recebido como parâmetro.
pub fn run_benchmark(pathfinder: &dyn PathfindingAlgorithm) -> String {
    // <<< RECEBE PATHFINDER
    let path = "pathfinding_benchmark.csv";

    let file = match File::create(path) {
        Ok(f) => f,
        Err(e) => return format!("Erro ao criar CSV: {}", e),
    };
    let mut writer = BufWriter::new(file);

    // Escreve o cabeçalho do CSV
    if let Err(e) = writeln!(
        writer,
        "grid_width,grid_height,obstacle_density,num_agents,total_time_us,avg_time_per_agent_us"
    ) {
        return format!("Erro ao escrever cabeçalho: {}", e);
    }

    // --- Configuração do Benchmark ---
    let resolutions = [(30, 18), (60, 36), (120, 72)];
    let densities = [0.1, 0.3, 0.5]; // 10% (fácil), 30% (médio), 50% (difícil)
    let agent_counts = [10, 50, 100, 200, 500];
    let repetitions = 3; // Média de 3 execuções para estabilizar

    // --- Execução do Benchmark ---
    for &(width, height) in &resolutions {
        for &density in &densities {
            for &n_agents in &agent_counts {
                let mut total_time_rep_us = 0.0;

                for _ in 0..repetitions {
                    // 1. Criar grid e obstáculos
                    let mut grid = Grid::new(width, height);
                    populate_obstacles(&mut grid, density);

                    // 2. Gerar tarefas
                    let tasks = generate_agent_tasks(&grid, n_agents);
                    if tasks.is_empty() {
                        continue;
                    }

                    // 3. Medir o tempo
                    let start = Instant::now();
                    for (start_pos, end_pos) in &tasks {
                        // Executa o Pathfinding usando o trait object
                        let _ = pathfinder.find_path(&grid, *start_pos, *end_pos);
                    }
                    let time_us = start.elapsed().as_secs_f32() * 1_000_000.0;
                    total_time_rep_us += time_us;
                }

                let avg_total_time_us = total_time_rep_us / repetitions as f32;
                let avg_agent_time_us = avg_total_time_us / n_agents as f32;

                // 4. Salvar no CSV
                if let Err(e) = writeln!(
                    writer,
                    "{},{},{:.2},{},{:.2},{:.2}",
                    width, height, density, n_agents, avg_total_time_us, avg_agent_time_us
                ) {
                    return format!("Erro ao escrever linha no CSV: {}", e);
                }
            }
        }
    }

    "Benchmark concluído! Salvo em pathfinding_benchmark.csv".to_string()
}
