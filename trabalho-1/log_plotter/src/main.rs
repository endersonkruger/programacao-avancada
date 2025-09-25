use plotters::prelude::*;
use std::env; // 1. Importa o módulo para ler argumentos
use std::fs::read_to_string;
use std::process; // Para encerrar o programa em caso de erro

// O nome do arquivo de saída continua fixo
const OUTPUT_FILENAME: &str = "mouse_path.png";

fn main() {
    println!("Iniciando o programa de plotagem de log...");

    // 2. Coleta os argumentos passados pela linha de comando
    let args: Vec<String> = env::args().collect();

    // 3. Verifica se o número de argumentos está correto
    //    args[0] é o nome do programa, args[1] deve ser o nome do arquivo.
    if args.len() != 2 {
        eprintln!("\nErro: Nenhum arquivo de entrada fornecido.");
        eprintln!("Uso correto: {} <caminho_para_o_arquivo.txt>", args[0]);
        process::exit(1); // Encerra o programa com um código de erro
    }

    // 4. Usa o segundo argumento como o nome do arquivo de entrada
    let input_filename = &args[1];
    println!("Lendo dados de '{}'...", input_filename);

    match parse_log_file(input_filename) {
        Ok(mouse_path) => {
            println!("Arquivo de log lido com sucesso. Encontrados {} registros de percurso.", mouse_path.len());
            if let Err(e) = generate_plot(&mouse_path, OUTPUT_FILENAME) {
                eprintln!("Ocorreu um erro ao gerar o gráfico: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Ocorreu um erro ao ler o arquivo de log '{}': {}", input_filename, e);
        }
    }
}

/// Lê o arquivo de log e extrai apenas os dados do percurso do mouse.
fn parse_log_file(filename: &str) -> Result<Vec<(f64, f32, f32)>, std::io::Error> {
    let mut mouse_path = Vec::new();
    let file_content = read_to_string(filename)?;

    let mut is_mouse_path_section = false;

    for line in file_content.lines() {
        if line.contains("Mouse path (time_seconds, x, y):") {
            is_mouse_path_section = true;
            continue;
        }

        if line.contains("Click events") {
            break;
        }

        if is_mouse_path_section {
            let parts: Vec<&str> = line.split(", ").collect();
            if parts.len() == 3 {
                if let (Ok(time), Ok(x), Ok(y)) = (parts[0].parse::<f64>(), parts[1].parse::<f32>(), parts[2].parse::<f32>()) {
                    mouse_path.push((time, x, y));
                }
            }
        }
    }

    Ok(mouse_path)
}

/// Gera e salva um gráfico a partir dos dados do percurso do mouse.
fn generate_plot(data: &Vec<(f64, f32, f32)>, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(filename, (1280, 720)).into_drawing_area();
    root.fill(&WHITE)?;

    if data.is_empty() {
        println!("Nenhum dado de percurso do mouse para plotar.");
        return Ok(());
    }

    let max_time = data.last().unwrap().0;

    let mut chart = ChartBuilder::on(&root)
        .caption("Percurso do Mouse vs. Tempo", ("sans-serif", 40))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0f64..max_time, 0f32..1280f32)?;

    chart.configure_mesh().draw()?;

    chart
        .draw_series(LineSeries::new(data.iter().map(|(t, x, _)| (*t, *x)), &RED))?
        .label("Coordenada X")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart
        .draw_series(LineSeries::new(data.iter().map(|(t, _, y)| (*t, *y)), &BLUE))?
        .label("Coordenada Y")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;
    println!("Gráfico salvo com sucesso em '{}'", filename);

    Ok(())
}
