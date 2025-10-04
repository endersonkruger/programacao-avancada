//! Este programa realiza uma análise de desempenho completa dos algoritmos
//! para a geração de Diagramas de Voronoi e Triangulações de Delaunay.
//! Ele executa benchmarks para diferentes quantidades de pontos, salva os
//! resultados em um arquivo CSV e, em seguida, gera um gráfico a partir desses dados.

use macroquad::rand;
use plotters::prelude::*;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::time::Instant;

//==============================================================================
// ESTRUTURAS DE DADOS (GEOMETRIA E BENCHMARK)
//==============================================================================

/// Representa um ponto 2D com coordenadas de ponto flutuante.
#[derive(Copy, Clone, Debug, PartialEq)]
struct P { x: f32, y: f32 }
impl P {
    fn new(x: f32, y: f32) -> Self { Self { x, y } }
}

/// Representa um triângulo definido por três vértices do tipo `P`.
#[derive(Clone, Debug)]
struct Triangle { a: P, b: P, c: P }

/// Representa uma única linha de dados no arquivo CSV de benchmark.
/// Utiliza `serde` para serializar (escrever) e desserializar (ler) os dados.
#[derive(Debug, Serialize, Deserialize)]
struct BenchmarkRecord {
    #[serde(rename = "Pontos")]
    pontos: i32,
    #[serde(rename = "TempoVoronoi(ms)")]
    tempo_voronoi: f64,
    #[serde(rename = "TempoDelaunay(ms)")]
    tempo_delaunay: f64,
}

//==============================================================================
// FUNÇÕES DOS ALGORITMOS (VORONOI E DELAUNAY)
//==============================================================================

/// Calcula a interseção de duas retas infinitas. Retorna `None` se forem paralelas.
fn line_intersection(p1: P, p2: P, q1: P, q2: P) -> Option<P> {
    let a1 = p2.y - p1.y; let b1 = p1.x - p2.x; let c1 = a1 * p1.x + b1 * p1.y;
    let a2 = q2.y - q1.y; let b2 = q1.x - q2.x; let c2 = a2 * q1.x + b2 * q1.y;
    let det = a1 * b2 - a2 * b1;
    if det.abs() < 1e-6 { None } else { Some(P::new((b2 * c1 - b1 * c2) / det, (a1 * c2 - a2 * c1) / det)) }
}

/// Recorta um polígono por um semiplano, mantendo os pontos no lado positivo da normal.
fn clip_polygon_halfplane(poly: &Vec<P>, normal: P, mid: P) -> Vec<P> {
    let mut out = Vec::new(); if poly.is_empty() { return out; }
    let mut prev = *poly.last().unwrap();
    let mut prev_inside = ((prev.x - mid.x) * normal.x + (prev.y - mid.y) * normal.y) >= 0.0;
    for &cur in poly.iter() {
        let cur_inside = ((cur.x - mid.x) * normal.x + (cur.y - mid.y) * normal.y) >= 0.0;
        if prev_inside != cur_inside {
            if let Some(ix) = line_intersection(prev, cur, P::new(mid.x - normal.y, mid.y + normal.x), P::new(mid.x + normal.y, mid.y - normal.x)) {
                out.push(ix);
            }
        }
        if cur_inside { out.push(cur); }
        prev = cur; prev_inside = cur_inside;
    }
    out
}

/// Constrói a célula de Voronoi para um ponto (site) através do recorte sucessivo
/// de um polígono de limites pelas mediatrizes formadas com os outros pontos.
fn voronoi_cell(site_idx: usize, sites: &Vec<P>, bounds: &Vec<P>) -> Vec<P> {
    let site = sites[site_idx]; let mut poly = bounds.clone();
    for (j, other) in sites.iter().enumerate() { if j == site_idx { continue; }
        let mid = P::new((site.x + other.x) / 2.0, (site.y + other.y) / 2.0);
        let normal = P::new(other.x - site.x, other.y - site.y);
        poly = clip_polygon_halfplane(&poly, P::new(-normal.x, -normal.y), mid);
        if poly.is_empty() { break; }
    }
    poly
}

/// Verifica se um ponto `p` está dentro da circunferência circunscrita de um triângulo.
fn circumcircle_contains(tri: &Triangle, p: &P) -> bool {
    let (ax, ay, bx, by, cx, cy) = (tri.a.x, tri.a.y, tri.b.x, tri.b.y, tri.c.x, tri.c.y);
    let d = 2.0 * (ax * (by - cy) + bx * (cy - ay) + cx * (ay - by)); if d.abs() < 1e-6 { return false; }
    let ux = ((ax * ax + ay * ay) * (by - cy) + (bx * bx + by * by) * (cy - ay) + (cx * cx + cy * cy) * (ay - by)) / d;
    let uy = ((ax * ax + ay * ay) * (cx - bx) + (bx * bx + by * by) * (ax - cx) + (cx * cx + cy * cy) * (bx - ax)) / d;
    let r2 = (ux - ax) * (ux - ax) + (uy - ay) * (uy - ay);
    let (dx, dy) = (ux - p.x, uy - p.y);
    dx * dx + dy * dy <= r2 + 1e-6
}

/// Compara dois pontos com uma pequena tolerância para lidar com imprecisões de ponto flutuante.
fn approx_eq(a: P, b: P) -> bool { ((a.x - b.x).abs() < 1e-3) && ((a.y - b.y).abs() < 1e-3) }

/// Verifica se um triângulo utiliza algum dos vértices do super-triângulo inicial.
fn uses_supervertex(p: &P, a: P, b: P, c: P) -> bool { approx_eq(*p, a) || approx_eq(*p, b) || approx_eq(*p, c) }

/// Gera a triangulação de Delaunay utilizando o algoritmo de Bowyer-Watson.
fn bowyer_watson(sites: &Vec<P>) -> Vec<Triangle> {
    let mut triangles = Vec::new(); if sites.len() < 2 { return triangles; }

    // 1. Cria um "super-triângulo" que engloba todos os pontos do conjunto.
    let (mut minx, mut maxx, mut miny, mut maxy) = (sites[0].x, sites[0].x, sites[0].y, sites[0].y);
    for s in sites.iter() { minx = minx.min(s.x); maxx = maxx.max(s.x); miny = miny.min(s.y); maxy = maxy.max(s.y); }
    let delta_max = f32::max((maxx - minx).abs(), (maxy - miny).abs()).max(1.0) * 10.0;
    let (midx, midy) = ((minx + maxx) / 2.0, (miny + maxy) / 2.0);
    let (st_a, st_b, st_c) = (P::new(midx - 2.0 * delta_max, midy - delta_max), P::new(midx, midy + 2.0 * delta_max), P::new(midx + 2.0 * delta_max, midy - delta_max));
    triangles.push(Triangle { a: st_a, b: st_b, c: st_c });

    // 2. Itera sobre cada ponto, adicionando-o à triangulação.
    for p in sites.iter() {
        let mut bad_triangles = Vec::new();
        let mut edges: Vec<(P, P)> = Vec::new();

        // 3. Encontra todos os triângulos "ruins", cuja circunferência circunscrita contém o novo ponto.
        for (i, tri) in triangles.iter().enumerate() { if circumcircle_contains(tri, p) { bad_triangles.push(i); } }

        // 4. Encontra a fronteira da cavidade poligonal formada pelos triângulos ruins.
        // Uma aresta está na fronteira se pertencer a apenas um triângulo ruim.
        for &idx in bad_triangles.iter().rev() {
            let tri = &triangles[idx]; let candidates = vec![(tri.a, tri.b), (tri.b, tri.c), (tri.c, tri.a)];
            for e in candidates {
                // A anotação de tipo `ex: &(P, P)` é necessária para ajudar o compilador
                // a inferir o tipo em algumas versões/edições mais recentes do Rust.
                if let Some(pos) = edges.iter().position(|ex: &(P, P)| (approx_eq(ex.0, e.0) && approx_eq(ex.1, e.1)) || (approx_eq(ex.0, e.1) && approx_eq(ex.1, e.0))) { 
                    edges.remove(pos); 
                } else { 
                    edges.push(e); 
                }
            }
        }

        // 5. Remove os triângulos ruins e re-triangula a cavidade com o novo ponto.
        for &idx in bad_triangles.iter().rev() { triangles.remove(idx); }
        for (ea, eb) in edges { triangles.push(Triangle { a: ea, b: eb, c: *p }); }
    }

    // 6. Limpeza final: remove da triangulação os triângulos que compartilham vértices com o super-triângulo.
    triangles.into_iter().filter(|t| !uses_supervertex(&t.a, st_a, st_b, st_c) && !uses_supervertex(&t.b, st_a, st_b, st_c) && !uses_supervertex(&t.c, st_a, st_b, st_c)).collect()
}

//==============================================================================
// LÓGICA PRINCIPAL (BENCHMARK E PLOTAGEM)
//==============================================================================

/// Roda os benchmarks e salva os resultados em um arquivo CSV.
fn run_and_save_benchmarks(file_path: &str) -> Result<(), Box<dyn Error>> {
    println!("Iniciando benchmark e salvando em '{}'...", file_path);
    let mut writer = csv::Writer::from_path(file_path)?;

    writer.write_record(&["Pontos", "TempoVoronoi(ms)", "TempoDelaunay(ms)"])?;

    let (w, h) = (800.0, 600.0);
    let pad = 200.0;
    let bounds = vec![P::new(-pad, -pad), P::new(w + pad, -pad), P::new(w + pad, h + pad), P::new(-pad, h + pad)];

    for n in (50..=1000).step_by(50) {
        let sites: Vec<P> = (0..n).map(|_| P::new(rand::gen_range(0.0, w), rand::gen_range(0.0, h))).collect();

        let start_voronoi = Instant::now();
        for i in 0..sites.len() {
            let _ = voronoi_cell(i, &sites, &bounds);
        }
        let time_voronoi = start_voronoi.elapsed().as_secs_f64() * 1000.0;

        let start_delaunay = Instant::now();
        let _ = bowyer_watson(&sites);
        let time_delaunay = start_delaunay.elapsed().as_secs_f64() * 1000.0;

        // Escreve o registro manualmente para garantir um formato de número consistente
        // e evitar problemas de formatação automática baseada na localidade do sistema.
        writer.write_record(&[
            n.to_string(),
            format!("{:.4}", time_voronoi),
            format!("{:.4}", time_delaunay),
        ])?;
    }

    writer.flush()?;
    println!("Benchmark finalizado.");
    Ok(())
}

/// Lê um arquivo CSV de benchmark e gera um gráfico de desempenho.
fn create_plot_from_csv(file_path: &str) -> Result<(), Box<dyn Error>> {
    println!("Lendo dados de '{}' e gerando gráfico...", file_path);
    let mut rdr = csv::Reader::from_path(file_path)?;
    let records: Vec<BenchmarkRecord> = rdr.deserialize().collect::<Result<_, _>>()?;

    let output_file = "benchmark_graph.png";
    let root = BitMapBackend::new(output_file, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let pontos: Vec<i32> = records.iter().map(|r| r.pontos).collect();
    let tempos_voronoi: Vec<f64> = records.iter().map(|r| r.tempo_voronoi).collect();
    let tempos_delaunay: Vec<f64> = records.iter().map(|r| r.tempo_delaunay).collect();

    let max_pontos = pontos.iter().max().cloned().unwrap_or(1000);
    let max_tempo = tempos_voronoi.iter().cloned().fold(0.0, f64::max)
        .max(tempos_delaunay.iter().cloned().fold(0.0, f64::max));

    let mut chart = ChartBuilder::on(&root)
        .caption("Desempenho: Voronoi vs. Delaunay", ("sans-serif", 40).into_font())
        .margin(10).x_label_area_size(40).y_label_area_size(60)
        .build_cartesian_2d(0..max_pontos, 0.0..max_tempo * 1.1)?;

    chart.configure_mesh().x_desc("Número de Pontos (N)").y_desc("Tempo de Execução (ms)").draw()?;

    chart.draw_series(LineSeries::new(pontos.iter().zip(tempos_voronoi.iter()).map(|(&x, &y)| (x, y)), &RED))?
        .label("Voronoi (O(N²))").legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart.draw_series(LineSeries::new(pontos.iter().zip(tempos_delaunay.iter()).map(|(&x, &y)| (x, y)), &BLUE))?
        .label("Delaunay (O(N²))").legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    chart.configure_series_labels().background_style(&WHITE.mix(0.8)).border_style(&BLACK).draw()?;

    root.present()?;
    println!("Gráfico salvo com sucesso em '{}'", output_file);
    Ok(())
}

/// Função principal que orquestra o benchmark e a criação do gráfico.
fn main() -> Result<(), Box<dyn Error>> {
    let file_path = "benchmark_data.csv";

    run_and_save_benchmarks(file_path)?;
    create_plot_from_csv(file_path)?;

    Ok(())
}
