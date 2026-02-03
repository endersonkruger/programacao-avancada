use crate::InputMode;
use crate::agent_decorator::AgentComponent;
use crate::grid::{CellType, Grid};
use crate::pheromone::PheromoneManager;
use macroquad::prelude::*;

/// Desenha o mapa de feromônios
pub fn draw_pheromones(width: usize, height: usize, cell_size: f32) {
    let grid_snap = PheromoneManager::instance().get_grid_snapshot();
    
    for y in 0..height {
        if y >= grid_snap.len() { break; }
        for x in 0..width {
            if x >= grid_snap[y].len() { break; }
            
            let intensity = grid_snap[y][x];
            if intensity > 0.1 {
                let alpha = (intensity / 5.0).min(0.6); 
                let color = Color::new(1.0, 0.0, 1.0, alpha);
                
                draw_rectangle(
                    x as f32 * cell_size,
                    y as f32 * cell_size,
                    cell_size,
                    cell_size,
                    color
                );
            }
        }
    }
}

/// Desenha as linhas de grade (cinza claro)
pub fn draw_grid(width: usize, height: usize, cell_size: f32) {
    let screen_w = width as f32 * cell_size;
    let screen_h = height as f32 * cell_size;

    for i in 0..=width {
        let x = i as f32 * cell_size;
        draw_line(x, 0.0, x, screen_h, 1.0, GRAY);
    }
    for i in 0..=height {
        let y = i as f32 * cell_size;
        draw_line(0.0, y, screen_w, y, 1.0, GRAY);
    }
}

/// Desenha as células de obstáculo (quadrados pretos)
pub fn draw_cells(grid: &Grid, cell_size: f32) {
    for y in 0..grid.height {
        for x in 0..grid.width {
            if grid.cells[y][x] == CellType::Obstacle {
                draw_rectangle(
                    x as f32 * cell_size,
                    y as f32 * cell_size,
                    cell_size,
                    cell_size,
                    BLACK,
                );
            }
        }
    }
}

pub fn draw_agents(agents: &[Box<dyn AgentComponent>]) {
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

pub fn draw_input_feedback(
    mode: &InputMode,
    pending_start: Option<(usize, usize)>,
    mouse_grid_pos: (usize, usize),
    cell_size: f32,
    mouse_over_obstacle: bool,
) {
    let x = mouse_grid_pos.0 as f32 * cell_size;
    let y = mouse_grid_pos.1 as f32 * cell_size;
    let color: Color;

    match mode {
        InputMode::DrawObstacle => {
            color = if mouse_over_obstacle { RED } else { Color::new(0.3, 0.3, 0.3, 0.8) };
        }
        InputMode::SetStart => {
            color = if mouse_over_obstacle { RED } else { Color::new(0.0, 1.0, 0.0, 0.5) };
        }
        InputMode::SetEnd => {
            if let Some(start) = pending_start {
                draw_rectangle(
                    start.0 as f32 * cell_size,
                    start.1 as f32 * cell_size,
                    cell_size,
                    cell_size,
                    Color::new(0.0, 1.0, 0.0, 0.8),
                );
            }
            color = if mouse_over_obstacle { RED } else { Color::new(1.0, 0.0, 0.0, 0.5) };
        }
    }
    draw_rectangle(x, y, cell_size, cell_size, color);
}