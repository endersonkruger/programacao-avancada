use crate::agent_decorator::AgentComponent;
use crate::grid::{CellType, Grid};
use crate::InputMode;
use macroquad::prelude::*;

/// Constantes para cálculos hexagonais (Flat-Top)
const HEX_SIZE: f32 = 15.0; // Raio do hexágono (de centro a vértice)
const HEX_WIDTH: f32 = HEX_SIZE * 1.73205; // Largura: √3 * size
const VERTICAL_SPACING: f32 = HEX_SIZE * 1.5; // Espaçamento vertical: 1.5 * size

/// Converte coordenadas de grid para posição central do hexágono em pixels
pub fn hex_grid_to_screen(pos: (usize, usize)) -> Vec2 {
    let x = pos.0 as f32;
    let y = pos.1 as f32;

    // A linha ímpar (y % 2 != 0) é deslocada horizontalmente
    let offset_x = if pos.1 % 2 == 1 { HEX_WIDTH / 2.0 } else { 0.0 };

    // X: Posição da coluna * Largura + Offset da linha + Metade da Largura
    let center_x = x * HEX_WIDTH + offset_x + HEX_WIDTH / 2.0;

    // Y: Posição da linha * Espaçamento Vertical + Raio
    let center_y = y * VERTICAL_SPACING + HEX_SIZE;

    vec2(center_x, center_y)
}

/// Converte coordenadas de tela para coordenadas de grid hexagonal
pub fn hex_screen_to_grid(screen_x: f32, screen_y: f32) -> (usize, usize) {
    let q_approx = (screen_x - HEX_WIDTH / 2.0) / HEX_WIDTH;
    let r_approx = screen_y / VERTICAL_SPACING;

    // Estimativa inicial do grid (arredondamento)
    let y_est = r_approx.round() as i32;
    let x_est_raw = q_approx - (y_est as f32 % 2.0) * 0.5;
    let x_est = x_est_raw.round() as i32;

    // Verifica os 7 hexágonos ao redor da estimativa para encontrar o mais próximo
    let mouse_pos = vec2(screen_x, screen_y);
    let mut closest_pos = (x_est.max(0) as usize, y_est.max(0) as usize);
    let mut min_dist_sq = f32::MAX;

    // Busca nas 9 posições ao redor do ponto estimado
    for dy in -1..=1 {
        for dx in -1..=1 {
            let gx = (x_est + dx).max(0) as usize;
            let gy = (y_est + dy).max(0) as usize;

            let center = hex_grid_to_screen((gx, gy));
            let distance = mouse_pos.distance(center);
            let dist_sq = distance * distance;
            if dist_sq < min_dist_sq {
                min_dist_sq = dist_sq;
                closest_pos = (gx, gy);
            }
        }
    }

    closest_pos
}

/// Desenha um hexágono
pub fn draw_hexagon(cx: f32, cy: f32, size: f32, color: Color, filled: bool) {
    // Offset de -30 graus (ou 330) para flat-top
    let angles: [f32; 6] = [30.0, 90.0, 150.0, 210.0, 270.0, 330.0];
    let points: Vec<Vec2> = angles
        .iter()
        .map(|&angle| {
            let rad = angle.to_radians();
            vec2(cx + size * rad.cos(), cy + size * rad.sin())
        })
        .collect();

    if filled {
        // Desenha hexágono preenchido usando triângulos
        for i in 0..6 {
            let next = (i + 1) % 6;
            draw_triangle(vec2(cx, cy), points[i], points[next], color);
        }
    } else {
        // Desenha apenas as bordas
        for i in 0..6 {
            let next = (i + 1) % 6;
            draw_line(
                points[i].x,
                points[i].y,
                points[next].x,
                points[next].y,
                1.0,
                color,
            );
        }
    }
}

/// Desenha o grid hexagonal completo
pub fn draw_hexagonal_grid(width: usize, height: usize) {
    for y in 0..height {
        for x in 0..width {
            let center = hex_grid_to_screen((x, y));
            draw_hexagon(center.x, center.y, HEX_SIZE, GRAY, false);
        }
    }
}

/// Desenha as células de obstáculo no grid hexagonal
pub fn draw_hexagonal_cells(grid: &Grid) {
    for y in 0..grid.height {
        for x in 0..grid.width {
            if grid.cells[y][x] == CellType::Obstacle {
                let center = hex_grid_to_screen((x, y));
                draw_hexagon(center.x, center.y, HEX_SIZE, BLACK, true);
            }
        }
    }
}

/// Desenha os agentes no grid hexagonal
pub fn draw_hexagonal_agents(agents: &Vec<Box<dyn AgentComponent>>) {
    for agent in agents {
        let pos = agent.get_pos();

        let detection_color = agent.get_detection_color();

        draw_circle_lines(
            pos.x,
            pos.y,
            agent.get_detection_radius(),
            2.0,
            detection_color,
        );

        draw_circle(pos.x, pos.y, agent.get_physical_radius(), agent.get_color());
    }
}

/// Desenha feedback visual para input no grid hexagonal
pub fn draw_hexagonal_input_feedback(
    mode: &InputMode,
    pending_start: Option<(usize, usize)>,
    mouse_grid_pos: (usize, usize),
    mouse_over_obstacle: bool,
) {
    let center = hex_grid_to_screen(mouse_grid_pos);

    let color = match mode {
        InputMode::DrawObstacle => {
            if mouse_over_obstacle {
                RED
            } else {
                Color::new(0.3, 0.3, 0.3, 0.6)
            }
        }
        InputMode::SetStart => {
            if mouse_over_obstacle {
                RED
            } else {
                Color::new(0.0, 1.0, 0.0, 0.4)
            }
        }
        InputMode::SetEnd => {
            // Desenha o ponto inicial pendente
            if let Some(start) = pending_start {
                let start_center = hex_grid_to_screen(start);
                draw_hexagon(
                    start_center.x,
                    start_center.y,
                    HEX_SIZE,
                    Color::new(0.0, 1.0, 0.0, 0.6),
                    true,
                );
            }

            if mouse_over_obstacle {
                RED
            } else {
                Color::new(1.0, 0.0, 0.0, 0.4)
            }
        }
    };

    draw_hexagon(center.x, center.y, HEX_SIZE, color, true);
}
