use macroquad::prelude::*;

/// Tipos de célula que podem existir no grid.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CellType {
    Empty,    // Célula vazia, livre para movimento
    Obstacle, // Célula de obstáculo, bloqueada
}

/// Estrutura principal que representa o ambiente de simulação.
pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<CellType>>,
}

impl Grid {
    /// Cria um novo grid preenchido com células vazias.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![vec![CellType::Empty; width]; height],
        }
    }

    /// Define o tipo de uma célula específica nas coordenadas (x, y).
    pub fn set_cell(&mut self, x: usize, y: usize, cell_type: CellType) {
        if x < self.width && y < self.height {
            self.cells[y][x] = cell_type;
        }
    }

    /// Verifica se uma célula é um obstáculo.
    /// Retorna `true` se estiver fora dos limites ou for um obstáculo.
    pub fn is_obstacle(&self, x: usize, y: usize) -> bool {
        if x < self.width && y < self.height {
            self.cells[y][x] == CellType::Obstacle
        } else {
            true // Considera fora dos limites como obstáculo para a busca de caminho
        }
    }

    /// Limpa todos os obstáculos do grid, redefinindo todas as células para Empty.
    pub fn clear(&mut self) {
        self.cells = vec![vec![CellType::Empty; self.width]; self.height];
    }

    /// Encontra e retorna as coordenadas (x, y) de uma célula vazia aleatória.
    /// Retorna `None` se não conseguir encontrar após muitas tentativas.
    pub fn get_random_empty_cell(&self) -> Option<(usize, usize)> {
        let max_attempts = self.width * self.height;
        let mut attempts = 0;

        while attempts < max_attempts {
            let x = rand::gen_range(0, self.width);
            let y = rand::gen_range(0, self.height);

            if !self.is_obstacle(x, y) {
                return Some((x, y));
            }
            attempts += 1;
        }
        None // Não encontrou célula vazia
    }
}
