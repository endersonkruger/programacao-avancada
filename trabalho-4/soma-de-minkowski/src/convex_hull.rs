use crate::geometry::Point;

/// Calcula o produto vetorial (cross product) de 3 pontos.
/// Usado para determinar a orientação (sentido horário/anti-horário).
fn cross(o: Point, a: Point, b: Point) -> f32 {
    (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x)
}

/// Calcula a Envoltória Convexa (Convex Hull) usando o algoritmo de Graham Scan.
pub fn convex_hull(mut points: Vec<Point>) -> Vec<Point> {
    if points.len() <= 3 {
        return points;
    }

    // Ordena os pontos primeiro por x, depois por y
    points.sort_by(|a, b| {
        a.x.partial_cmp(&b.x)
            .unwrap()
            .then(a.y.partial_cmp(&b.y).unwrap())
    });

    // Constrói a envoltória inferior
    let mut lower = Vec::new();
    for p in &points {
        while lower.len() >= 2 && cross(lower[lower.len() - 2], lower[lower.len() - 1], *p) <= 0.0 {
            lower.pop();
        }
        lower.push(*p);
    }

    // Constrói a envoltória superior
    let mut upper = Vec::new();
    for p in points.iter().rev() {
        while upper.len() >= 2 && cross(upper[upper.len() - 2], upper[upper.len() - 1], *p) <= 0.0 {
            upper.pop();
        }
        upper.push(*p);
    }

    // Remove os últimos pontos duplicados (início e fim)
    lower.pop();
    upper.pop();

    // Combina as duas metades
    lower.extend(upper);
    lower
}
