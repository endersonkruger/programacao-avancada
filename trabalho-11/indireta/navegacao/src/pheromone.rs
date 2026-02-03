use macroquad::prelude::*;
use std::sync::{Mutex, OnceLock};

/// Decaimento do rastro
const DECAY_RATE: f32 = 5.0; 
/// Emissão para marcar a célula como ocupada
const AGENT_EMISSION: f32 = 100.0; 
/// Limiar de perigo
const DANGER_THRESHOLD: f32 = 0.5; 
/// Teto máximo
const MAX_INTENSITY: f32 = 10.0;

/// Gerenciador Singleton de Feromônios
pub struct PheromoneManager {
    grid: Mutex<Vec<Vec<f32>>>,
}

impl PheromoneManager {
    pub fn instance() -> &'static PheromoneManager {
        static INSTANCE: OnceLock<PheromoneManager> = OnceLock::new();
        INSTANCE.get_or_init(|| PheromoneManager {
            grid: Mutex::new(Vec::new()),
        })
    }

    /// Inicializa o grid de feromônios
    pub fn init(&self, width: usize, height: usize) {
        let mut grid = self.grid.lock().unwrap();
        *grid = vec![vec![0.0; width]; height];
    }

    /// Um agente deposita feromônio em sua posição atual
    pub fn deposit(&self, pos: Vec2, _cell_size: f32, grid_mode: crate::GridMode) {
        let (gx, gy) = crate::screen_to_grid(pos.x, pos.y, grid_mode);
        let mut grid = self.grid.lock().unwrap();
        
        if gy < grid.len() && gx < grid[0].len() {
            // Soma valor com um teto
            let new_val = grid[gy][gx] + AGENT_EMISSION * get_frame_time();
            grid[gy][gx] = new_val.min(MAX_INTENSITY);
        }
    }

    /// Verifica se a célula está "bloqueada" pela comunicação indireta
    pub fn is_blocked(&self, gx: usize, gy: usize) -> bool {
        let grid = self.grid.lock().unwrap();
        if gy < grid.len() && gx < grid[0].len() {
            return grid[gy][gx] > DANGER_THRESHOLD;
        }
        false
    }

    /// Atualiza o sistema (Evaporação dos feromônios)
    pub fn update(&self, dt: f32) {
        let mut grid = self.grid.lock().unwrap();
        for row in grid.iter_mut() {
            for cell in row.iter_mut() {
                if *cell > 0.0 {
                    *cell -= DECAY_RATE * dt;
                    if *cell < 0.0 {
                        *cell = 0.0;
                    }
                }
            }
        }
    }

    /// Retorna uma cópia do grid para renderização
    pub fn get_grid_snapshot(&self) -> Vec<Vec<f32>> {
        let grid = self.grid.lock().unwrap();
        grid.clone()
    }
    
    pub fn clear(&self) {
        let mut grid = self.grid.lock().unwrap();
        for row in grid.iter_mut() {
            for cell in row.iter_mut() {
                *cell = 0.0;
            }
        }
    }
}