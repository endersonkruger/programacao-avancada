use crate::convex_hull::convex_hull;
use crate::geometry::{Point, circle_points, random_points, rectangle_points};

use macroquad::prelude::{screen_height, screen_width};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::time::Instant; // Necessário para a geração de pontos

/// Executa o benchmark e salva os resultados em um CSV.
/// Retorna uma String com a mensagem de resultado (sucesso ou erro).
pub fn run_benchmark() -> String {
    let path = "benchmark_results.csv";

    // Tenta criar o arquivo
    let file = match File::create(path) {
        Ok(f) => f,
        Err(e) => return format!("Erro ao criar CSV: {}", e),
    };
    let mut writer = BufWriter::new(file);

    // Escreve o cabeçalho do CSV
    if let Err(e) = writeln!(writer, "distribuicao,n_pontos,n_envoltoria,tempo_us") {
        return format!("Erro ao escrever cabeçalho: {}", e);
    }

    // Configuração do Benchmark
    let n_values = [100, 500, 1000, 5000, 10000, 25000, 50000, 100000];
    let repetitions = 5; // Repetições para tirar a média

    // Pega as dimensões da tela para a geração de pontos
    let center = Point {
        x: screen_width() / 2.0,
        y: screen_height() / 2.0,
    };
    let radius = screen_height().min(screen_width()) / 2.0 - 50.0;
    let rect_width = radius * 1.6;
    let rect_height = radius;

    // Execução do Benchmark
    for &n in &n_values {
        for dist_type in ["random", "circle", "rectangle"] {
            let mut total_time_us = 0.0;
            let mut last_hull_len = 0;

            for _ in 0..repetitions {
                // 1. Gerar pontos
                let test_points = match dist_type {
                    "random" => random_points(n),
                    "circle" => circle_points(n, center, radius),
                    "rectangle" => rectangle_points(n, center, rect_width, rect_height),
                    _ => vec![], // Nunca deve acontecer
                };

                // 2. Medir o tempo
                let start = Instant::now();
                let test_hull = convex_hull(test_points.clone());
                let time_us = start.elapsed().as_secs_f32() * 1_000_000.0;

                total_time_us += time_us;
                last_hull_len = test_hull.len(); // Pega o N da envoltória
            }

            let avg_time_us = total_time_us / repetitions as f32;

            // 3. Salvar no CSV
            if let Err(e) = writeln!(
                writer,
                "{},{},{},{:.2}",
                dist_type, n, last_hull_len, avg_time_us
            ) {
                return format!("Erro ao escrever linha no CSV: {}", e);
            }
        }
    }

    // Retorna a mensagem de sucesso
    "Benchmark concluído! Salvo em benchmark_results.csv".to_string()
}
