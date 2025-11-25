use crate::grid::Grid;
use crate::pathfinding::a_star_search;

/// Contrato (Trait) para qualquer algoritmo de busca de caminho.
/// Isso permite que o código cliente (main.rs) chame find_path() sem
/// saber se está usando A*, Dijkstra ou outro algoritmo.
pub trait PathfindingAlgorithm {
    fn find_path(
        &self,
        grid: &Grid,
        start: (usize, usize),
        end: (usize, usize),
    ) -> Option<Vec<(usize, usize)>>;
}

/// Implementação concreta que usa o A* de 4 direções (cardeais) existente.
/// O PathfindingAlgorithm é injetado no main.rs e no benchmark.rs.
pub struct AStarCardinal;

impl PathfindingAlgorithm for AStarCardinal {
    fn find_path(
        &self,
        grid: &Grid,
        start: (usize, usize),
        end: (usize, usize),
    ) -> Option<Vec<(usize, usize)>> {
        // Delega a chamada para a função A* concreta em pathfinding.rs
        a_star_search(grid, start, end)
    }
}

// (FUTURO) Poderia ser adicionado AStarDiagonal aqui
// pub struct AStarDiagonal;
// impl PathfindingAlgorithm for AStarDiagonal { ... }
