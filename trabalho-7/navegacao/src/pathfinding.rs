use crate::grid::Grid;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

/// Estrutura que representa um Nó usado pelo A* na fila de prioridade.
#[derive(Copy, Clone, Eq, PartialEq)]
struct Node {
    pos: (usize, usize), // Posição (coordenadas x, y) no grid
    f_cost: usize,       // Custo total (g + h)
    g_cost: usize,       // Custo do início até este nó
}

// Implementação de Ordenação para a BinaryHeap (fila de prioridade)
// A BinaryHeap do Rust é uma max-heap, então invertemos a lógica (cmp)
// para que ela funcione como uma min-heap baseada no f_cost (menor f_cost tem maior prioridade).
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compara f_cost de forma invertida
        other
            .f_cost
            .cmp(&self.f_cost)
            // Desempata usando a posição, se os f_costs forem iguais
            .then_with(|| self.pos.cmp(&other.pos))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Heurística (Distância de Manhattan).
/// Estima o custo de mover do ponto 'a' até o ponto 'b'.
fn heuristic(a: (usize, usize), b: (usize, usize)) -> usize {
    (a.0.abs_diff(b.0)) + (a.1.abs_diff(b.1))
}

/// Reconstrói o caminho final a partir do mapa `came_from`.
fn reconstruct_path(
    came_from: &HashMap<(usize, usize), (usize, usize)>,
    mut current: (usize, usize),
) -> Vec<(usize, usize)> {
    let mut path = vec![current];
    while let Some(&prev) = came_from.get(&current) {
        path.push(prev);
        current = prev;
    }
    path.reverse(); // O caminho é construído do fim para o começo
    path
}

/// Encontra o caminho mais curto entre dois pontos no grid usando o algoritmo A*.
/// Assume movimento em 4 direções (cardeais).
pub fn a_star_search(
    grid: &Grid,
    start: (usize, usize),
    end: (usize, usize),
) -> Option<Vec<(usize, usize)>> {
    let mut open_set = BinaryHeap::new();
    let mut came_from = HashMap::new();

    // g_costs: Custo do início até o nó
    let mut g_costs = HashMap::new();
    g_costs.insert(start, 0);

    // Adiciona o nó inicial
    open_set.push(Node {
        pos: start,
        f_cost: heuristic(start, end),
        g_cost: 0,
    });

    while let Some(current) = open_set.pop() {
        // Chegou ao destino
        if current.pos == end {
            return Some(reconstruct_path(&came_from, end));
        }

        // Define os vizinhos em 4 direções (Norte, Sul, Leste, Oeste)
        let neighbors = [
            (current.pos.0, current.pos.1.saturating_sub(1)), // Cima
            (current.pos.0, current.pos.1 + 1),               // Baixo
            (current.pos.0.saturating_sub(1), current.pos.1), // Esquerda
            (current.pos.0 + 1, current.pos.1),               // Direita
        ];

        for &neighbor_pos in &neighbors {
            // Pula vizinhos inválidos (obstáculos ou fora do grid)
            if grid.is_obstacle(neighbor_pos.0, neighbor_pos.1) {
                continue;
            }

            // O custo de mover para um vizinho é sempre 1
            let new_g_cost = current.g_cost + 1;
            let neighbor_g_cost = *g_costs.get(&neighbor_pos).unwrap_or(&usize::MAX);

            // Se este é um caminho melhor do que o já registrado
            if new_g_cost < neighbor_g_cost {
                g_costs.insert(neighbor_pos, new_g_cost);
                let f_cost = new_g_cost + heuristic(neighbor_pos, end);

                open_set.push(Node {
                    pos: neighbor_pos,
                    f_cost,
                    g_cost: new_g_cost,
                });
                came_from.insert(neighbor_pos, current.pos);
            }
        }
    }

    // Não encontrou caminho
    None
}
