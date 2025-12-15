use crate::agent_decorator::AgentComponent;
use crate::grid::{CellType, Grid};
use crate::{CELL_SIZE, InputMode};
use macroquad::prelude::*;

/// Desenha as linhas de grade (cinza claro)
pub fn draw_grid(width: usize, height: usize, cell_size: f32) {
    let screen_w = width as f32 * cell_size;
    let screen_h = height as f32 * cell_size;

    // Linhas verticais
    for i in 0..=width {
        let x = i as f32 * cell_size;
        draw_line(x, 0.0, x, screen_h, 1.0, GRAY);
    }
    // Linhas horizontais
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
                    BLACK, // Obstáculos são pretos
                );
            }
        }
    }
}

/// Desenha os agentes (círculos). A cor base é definida pela AgentFactory.
pub fn draw_agents(agents: &Vec<Box<dyn AgentComponent>>) {
    for agent_component in agents {
        // Usa os métodos do trait
        let active_color = agent_component.get_color();

        let color = if agent_component.is_finished() {
            // Agente finalizado fica verde e levemente transparente
            Color::new(0.0, 1.0, 0.0, 0.5)
        } else {
            // Agente ativo usa a cor definida pela fábrica
            active_color
        };
        let pos = agent_component.get_pos();
        draw_circle(pos.x, pos.y, CELL_SIZE * 0.35, color);
    }
}

/// Desenha um feedback visual (um "cursor") que segue o mouse,
/// indicando o modo de entrada atual.
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
            // Vermelho se estiver sobre obstáculo (apagando), cinza se estiver desenhando
            color = if mouse_over_obstacle {
                RED
            } else {
                Color::new(0.3, 0.3, 0.3, 0.8)
            };
        }
        InputMode::SetStart => {
            // Verde (para "início") ou Vermelho se for inválido (sobre obstáculo)
            color = if mouse_over_obstacle {
                RED
            } else {
                Color::new(0.0, 1.0, 0.0, 0.5)
            };
        }
        InputMode::SetEnd => {
            // Desenha o ponto inicial pendente (verde sólido)
            if let Some(start) = pending_start {
                draw_rectangle(
                    start.0 as f32 * cell_size,
                    start.1 as f32 * cell_size,
                    cell_size,
                    cell_size,
                    Color::new(0.0, 1.0, 0.0, 0.8), // Verde
                );
            }
            // Vermelho (para "destino") ou Vermelho sólido se for inválido
            color = if mouse_over_obstacle {
                RED
            } else {
                Color::new(1.0, 0.0, 0.0, 0.5)
            };
        }
    }

    // Desenha o "cursor" do grid
    draw_rectangle(x, y, cell_size, cell_size, color);
}
