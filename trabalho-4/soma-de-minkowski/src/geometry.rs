use macroquad::prelude::*;
use std::f32; // Usaremos f32::MAX

/// Estrutura básica para um ponto 2D.
#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

// --- Funções de Geração de Pontos ---

/// Gera `n` pontos em posições aleatórias dentro da tela.
pub fn random_points(n: usize) -> Vec<Point> {
    (0..n)
        .map(|_| Point {
            x: rand::gen_range(50.0, screen_width() - 50.0),
            y: rand::gen_range(50.0, screen_height() - 50.0),
        })
        .collect()
}

/// Gera `n` pontos formando um círculo.
pub fn circle_points(n: usize, center: Point, radius: f32) -> Vec<Point> {
    (0..n)
        .map(|i| {
            let angle = i as f32 / n as f32 * 2.0 * std::f32::consts::PI;
            Point {
                x: center.x + radius * angle.cos(),
                y: center.y + radius * angle.sin(),
            }
        })
        .collect()
}

/// Gera `n` pontos formando as bordas de um retângulo.
pub fn rectangle_points(n: usize, center: Point, width: f32, height: f32) -> Vec<Point> {
    let mut points = Vec::new();
    for i in 0..n {
        let t = i as f32 / n as f32; // Percentual ao longo do perímetro
        let side = (t * 4.0).floor() as i32; // Lado (0=topo, 1=direita, 2=base, 3=esquerda)
        let local = (t * 4.0) % 1.0; // Percentual ao longo do lado
        let (x, y) = match side {
            0 => (
                center.x - width / 2.0 + local * width,
                center.y - height / 2.0,
            ),
            1 => (
                center.x + width / 2.0,
                center.y - height / 2.0 + local * height,
            ),
            2 => (
                center.x + width / 2.0 - local * width,
                center.y + height / 2.0,
            ),
            _ => (
                center.x - width / 2.0,
                center.y + height / 2.0 - local * height,
            ),
        };
        points.push(Point { x, y });
    }
    points
}

// --- Funções de Cálculo de Distância ---

/// Calcula a distância ao quadrado entre dois pontos.
fn dist_sq(a: Point, b: Point) -> f32 {
    (a.x - b.x) * (a.x - b.x) + (a.y - b.y) * (a.y - b.y)
}

/// Calcula a distância mínima de um ponto `p` a um segmento de linha `a` - `b`.
fn point_to_segment_distance(p: Point, a: Point, b: Point) -> f32 {
    let l2 = dist_sq(a, b);
    if l2 == 0.0 {
        return dist_sq(p, a).sqrt(); // Segmento é um ponto
    }

    // Projeta o ponto p no segmento ab
    let t = ((p.x - a.x) * (b.x - a.x) + (p.y - a.y) * (b.y - a.y)) / l2;
    let t = t.max(0.0).min(1.0); // Prende t ao segmento [0, 1]

    // Ponto mais próximo no segmento
    let closest = Point {
        x: a.x + t * (b.x - a.x),
        y: a.y + t * (b.y - a.y),
    };

    dist_sq(p, closest).sqrt()
}

/// Verifica se um ponto está dentro de um polígono usando o algoritmo Ray-Casting.
fn point_in_polygon(poly: &Vec<Point>, p: Point) -> bool {
    if poly.is_empty() {
        return false;
    }
    let mut inside = false;
    let n = poly.len();
    let mut j = n - 1;
    for i in 0..n {
        let vi = poly[i];
        let vj = poly[j];

        let intersect = ((vi.y > p.y) != (vj.y > p.y))
            && (p.x < (vj.x - vi.x) * (p.y - vi.y) / (vj.y - vi.y) + vi.x);

        if intersect {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// Calcula a distância mínima de um ponto `p` a um polígono `poly`.
/// Retorna 0 se o ponto estiver dentro do polígono.
pub fn min_distance_from_point(poly: &Vec<Point>, p: Point) -> f32 {
    // 1. Se o ponto está dentro, a distância é 0.
    if point_in_polygon(poly, p) {
        return 0.0;
    }

    // 2. Se está fora, a distância é a menor distância a uma de suas arestas.
    let mut min_dist = f32::MAX;
    if poly.len() < 2 {
        return f32::MAX;
    } // Não é um polígono válido

    for i in 0..poly.len() {
        let a = poly[i];
        let b = poly[(i + 1) % poly.len()]; // Aresta de a para b
        let dist = point_to_segment_distance(p, a, b);
        min_dist = min_dist.min(dist);
    }

    min_dist
}
