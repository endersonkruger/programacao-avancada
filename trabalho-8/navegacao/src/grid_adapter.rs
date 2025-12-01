use crate::grid::Grid;

/// Interface unificada (Target) para trabalhar com diferentes tipos de grid.
/// O Adapter Pattern permite que grids com diferentes sistemas de vizinhança
/// sejam usados através da mesma interface.
pub trait GridAdapter {
    /// Retorna os vizinhos de uma célula, independente do tipo de grid
    fn get_neighbors(&self, pos: (usize, usize)) -> Vec<(usize, usize)>;

    /// Verifica se uma posição é válida e não é obstáculo
    fn is_valid_position(&self, pos: (usize, usize)) -> bool;

    /// Calcula o custo de movimento entre duas células adjacentes
    fn movement_cost(&self, from: (usize, usize), to: (usize, usize)) -> usize;
}

/// Adapter Concreto: Grid Retangular com 4 direções (Cardinal)
pub struct RectangularCardinalAdapter<'a> {
    grid: &'a Grid,
}

impl<'a> RectangularCardinalAdapter<'a> {
    pub fn new(grid: &'a Grid) -> Self {
        Self { grid }
    }
}

impl<'a> GridAdapter for RectangularCardinalAdapter<'a> {
    fn get_neighbors(&self, pos: (usize, usize)) -> Vec<(usize, usize)> {
        let mut neighbors = Vec::with_capacity(4);

        // Norte (cima)
        if pos.1 > 0 {
            neighbors.push((pos.0, pos.1 - 1));
        }

        // Sul (baixo)
        if pos.1 + 1 < self.grid.height {
            neighbors.push((pos.0, pos.1 + 1));
        }

        // Oeste (esquerda)
        if pos.0 > 0 {
            neighbors.push((pos.0 - 1, pos.1));
        }

        // Leste (direita)
        if pos.0 + 1 < self.grid.width {
            neighbors.push((pos.0 + 1, pos.1));
        }

        // Filtra obstáculos
        neighbors
            .into_iter()
            .filter(|&n| !self.grid.is_obstacle(n.0, n.1))
            .collect()
    }

    fn is_valid_position(&self, pos: (usize, usize)) -> bool {
        pos.0 < self.grid.width && pos.1 < self.grid.height && !self.grid.is_obstacle(pos.0, pos.1)
    }

    fn movement_cost(&self, _from: (usize, usize), _to: (usize, usize)) -> usize {
        1 // Custo uniforme para movimento cardinal
    }
}

/// Adapter Concreto: Grid Retangular com 8 direções (Cardinal + Diagonal)
pub struct RectangularDiagonalAdapter<'a> {
    grid: &'a Grid,
}

impl<'a> RectangularDiagonalAdapter<'a> {
    pub fn new(grid: &'a Grid) -> Self {
        Self { grid }
    }
}

impl<'a> GridAdapter for RectangularDiagonalAdapter<'a> {
    fn get_neighbors(&self, pos: (usize, usize)) -> Vec<(usize, usize)> {
        let mut neighbors = Vec::with_capacity(8);
        let x = pos.0 as i32;
        let y = pos.1 as i32;

        // 8 direções: cardinais + diagonais
        let directions = [
            (0, -1),  // Norte
            (0, 1),   // Sul
            (-1, 0),  // Oeste
            (1, 0),   // Leste
            (-1, -1), // Noroeste
            (1, -1),  // Nordeste
            (-1, 1),  // Sudoeste
            (1, 1),   // Sudeste
        ];

        for (dx, dy) in directions.iter() {
            let nx = x + dx;
            let ny = y + dy;

            if nx >= 0 && ny >= 0 {
                let new_pos = (nx as usize, ny as usize);
                if new_pos.0 < self.grid.width
                    && new_pos.1 < self.grid.height
                    && !self.grid.is_obstacle(new_pos.0, new_pos.1)
                {
                    neighbors.push(new_pos);
                }
            }
        }

        neighbors
    }

    fn is_valid_position(&self, pos: (usize, usize)) -> bool {
        pos.0 < self.grid.width && pos.1 < self.grid.height && !self.grid.is_obstacle(pos.0, pos.1)
    }

    fn movement_cost(&self, from: (usize, usize), to: (usize, usize)) -> usize {
        // Movimento diagonal custa mais (aproximadamente √2 ≈ 1.414)
        // Usamos 14 para movimento diagonal e 10 para cardinal
        let dx = from.0.abs_diff(to.0);
        let dy = from.1.abs_diff(to.1);

        if dx > 0 && dy > 0 {
            14 // Diagonal
        } else {
            10 // Cardinal
        }
    }
}

/// Adapter Concreto: Grid Hexagonal
/// Em grids hexagonais, cada célula tem 6 vizinhos
pub struct HexagonalAdapter<'a> {
    grid: &'a Grid,
    /// Define se usamos "flat-top" ou "pointy-top" hexagons
    flat_top: bool,
}

impl<'a> HexagonalAdapter<'a> {
    pub fn new(grid: &'a Grid, flat_top: bool) -> Self {
        Self { grid, flat_top }
    }
}

impl<'a> GridAdapter for HexagonalAdapter<'a> {
    fn get_neighbors(&self, pos: (usize, usize)) -> Vec<(usize, usize)> {
        let mut neighbors = Vec::with_capacity(6);
        let x = pos.0 as i32;
        let y = pos.1 as i32;

        // Vizinhos para hexagonal "flat-top"
        // A configuração muda dependendo se a linha é par ou ímpar
        let directions = if self.flat_top {
            if y % 2 == 0 {
                // Linha par
                vec![
                    (0, -1), // Norte
                    (1, 0),  // Nordeste
                    (1, 1),  // Sudeste
                    (0, 1),  // Sul
                    (-1, 1), // Sudoeste
                    (-1, 0), // Noroeste
                ]
            } else {
                // Linha ímpar (offset)
                vec![
                    (0, -1),  // Norte
                    (1, -1),  // Nordeste
                    (1, 0),   // Sudeste
                    (0, 1),   // Sul
                    (-1, 0),  // Sudoeste
                    (-1, -1), // Noroeste
                ]
            }
        } else {
            // "pointy-top" hexagons (orientação alternativa)
            if x % 2 == 0 {
                vec![
                    (1, 0),   // Leste
                    (0, 1),   // Sudeste
                    (-1, 1),  // Sudoeste
                    (-1, 0),  // Oeste
                    (-1, -1), // Noroeste
                    (0, -1),  // Nordeste
                ]
            } else {
                vec![
                    (1, 0),  // Leste
                    (1, 1),  // Sudeste
                    (0, 1),  // Sudoeste
                    (-1, 0), // Oeste
                    (0, -1), // Noroeste
                    (1, -1), // Nordeste
                ]
            }
        };

        for (dx, dy) in directions {
            let nx = x + dx;
            let ny = y + dy;

            if nx >= 0 && ny >= 0 {
                let new_pos = (nx as usize, ny as usize);
                if new_pos.0 < self.grid.width
                    && new_pos.1 < self.grid.height
                    && !self.grid.is_obstacle(new_pos.0, new_pos.1)
                {
                    neighbors.push(new_pos);
                }
            }
        }

        neighbors
    }

    fn is_valid_position(&self, pos: (usize, usize)) -> bool {
        pos.0 < self.grid.width && pos.1 < self.grid.height && !self.grid.is_obstacle(pos.0, pos.1)
    }

    fn movement_cost(&self, _from: (usize, usize), _to: (usize, usize)) -> usize {
        1 // Custo uniforme para todos os 6 vizinhos hexagonais
    }
}
