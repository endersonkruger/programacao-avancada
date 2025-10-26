use crate::convex_hull::convex_hull;
use crate::geometry::Point;

/// Calcula a Soma de Minkowski de dois polígonos (representados como listas de vértices).
pub fn minkowski_sum(poly_a: &Vec<Point>, poly_b: &Vec<Point>) -> Vec<Point> {
    if poly_a.is_empty() || poly_b.is_empty() {
        return Vec::new();
    }

    let mut summed_vertices = Vec::new();
    summed_vertices.reserve(poly_a.len() * poly_b.len()); // Otimização

    // 1. Somar cada vértice de A com cada vértice de B
    for pa in poly_a {
        for pb in poly_b {
            summed_vertices.push(Point {
                x: pa.x + pb.x,
                y: pa.y + pb.y,
            });
        }
    }

    // 2. A soma é a envoltória convexa dos pontos somados
    convex_hull(summed_vertices)
}
