use crate::grid_adapter::GridAdapter;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

/// Estrutura que representa um Nó usado pelo A* na fila de prioridade.
#[derive(Copy, Clone, Eq, PartialEq)]
struct Node {
    pos: (usize, usize),
    f_cost: usize,
    g_cost: usize,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .f_cost
            .cmp(&self.f_cost)
            .then_with(|| self.pos.cmp(&other.pos))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Heurística (Distância de Manhattan).
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
    path.reverse();
    path
}

/// A* Search que usa o GridAdapter para ser agnóstico ao tipo de grid.
/// Funciona com qualquer implementação de GridAdapter (retangular, hexagonal, etc.)
pub fn a_star_with_adapter(
    adapter: &dyn GridAdapter,
    start: (usize, usize),
    end: (usize, usize),
) -> Option<Vec<(usize, usize)>> {
    // Validações iniciais
    if !adapter.is_valid_position(start) || !adapter.is_valid_position(end) {
        return None;
    }

    let mut open_set = BinaryHeap::new();
    let mut came_from = HashMap::new();
    let mut g_costs = HashMap::new();

    g_costs.insert(start, 0);

    open_set.push(Node {
        pos: start,
        f_cost: heuristic(start, end),
        g_cost: 0,
    });

    while let Some(current) = open_set.pop() {
        if current.pos == end {
            return Some(reconstruct_path(&came_from, end));
        }

        // USA O ADAPTER para obter os vizinhos
        // Esta é a grande vantagem: o algoritmo não precisa saber
        // se está trabalhando com grid retangular, hexagonal, etc.
        let neighbors = adapter.get_neighbors(current.pos);

        for neighbor_pos in neighbors {
            // USA O ADAPTER para calcular o custo de movimento
            let move_cost = adapter.movement_cost(current.pos, neighbor_pos);
            let new_g_cost = current.g_cost + move_cost;
            let neighbor_g_cost = *g_costs.get(&neighbor_pos).unwrap_or(&usize::MAX);

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

    None
}
