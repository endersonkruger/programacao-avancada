use crate::geometry::Point;
use macroquad::prelude::*;

// Desenha os pontos
pub fn render_points(points: &Vec<Point>) {
    for p in points {
        draw_circle(p.x, p.y, 5.0, WHITE);
    }
}

// Função genérica para desenhar qualquer polígono
pub fn render_polygon(poly: &Vec<Point>, color: Color, line_thickness: f32) {
    if poly.len() > 1 {
        for i in 0..poly.len() {
            let a = poly[i];
            let b = poly[(i + 1) % poly.len()];
            draw_line(a.x, a.y, b.x, b.y, line_thickness, color);
        }
    }
}

// Função para desenhar o "robô" em uma posição específica
pub fn render_robot(shape: &Vec<Point>, position: Point, color: Color) {
    let mut world_poly = Vec::new();
    for p in shape {
        world_poly.push(Point {
            x: p.x + position.x,
            y: p.y + position.y,
        });
    }
    render_polygon(&world_poly, color, 2.0);
}

// render_hull usa a função genérica render_polygon
pub fn render_hull(hull: &Vec<Point>) {
    render_polygon(hull, RED, 2.0); // Continua desenhando a envoltória em vermelho
}

// Mostra o HUD (informações na tela)
pub fn render_hud(
    num_points: usize,
    exec_time_us: f32,
    distances: &Vec<(usize, f32)>, // Recebe a lista de distâncias (ID, valor)
    benchmark_msg: &str,
) {
    draw_text(
        &format!("Pontos (ativos): {}", num_points),
        20.0,
        30.0,
        24.0,
        GREEN,
    );
    draw_text(
        &format!("Tempo (total): {:.2} µs", exec_time_us),
        20.0,
        60.0,
        24.0,
        YELLOW,
    );

    // --- Início da Tabela de Distâncias ---
    let mut y_offset = 90.0;

    // Mostra a mensagem do benchmark se houver
    if !benchmark_msg.is_empty() {
        draw_text(benchmark_msg, 20.0, y_offset, 24.0, WHITE);
        y_offset += 30.0;
    }

    draw_text("Distâncias do Robô:", 20.0, y_offset, 22.0, WHITE);
    y_offset += 25.0;

    // Desenha cada entrada da tabela de distâncias
    for (id, dist) in distances {
        let color = if *dist == 0.0 { RED } else { WHITE }; // Vermelho se colidindo
        draw_text(
            &format!("  Polígono {}: {:.2}", id, dist),
            20.0,
            y_offset,
            20.0,
            color,
        );
        y_offset += 25.0;
    }
    // --- Fim da Tabela de Distâncias ---

    // Texto de ajuda com os controles
    draw_text(
        "Clique: Ponto | Enter: Finalizar Polígono | R/C/T: Gerar | B: BENCHMARK | Espaço: Limpar | Esc/Ctrl+C: Sair",
        20.0,
        screen_height() - 30.0,
        20.0,
        GRAY,
    );
}
