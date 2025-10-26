use macroquad::prelude::*;

/// Estrutura básica para um ponto 2D.
#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

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
