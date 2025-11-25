use crate::agent_factory::{AgentFactory, BlueAgentFactory, RedAgentFactory};
use crate::grid::Grid;
use crate::grid_factory::{GridFactory, RectangularGridFactory};
use crate::pathfinding_factory::{AStarCardinal, PathfindingAlgorithm};

/// Contrato principal (Trait) para a criação de um conjunto coerente de simulação.
/// Qualquer simulação (4-dir, 8-dir, etc.) deve implementar este trait.
pub trait SimulationFactory {
    /// Cria uma instância da grade, delegando ao GridFactory.
    fn create_grid(&self, width: usize, height: usize) -> Grid;

    /// Cria o algoritmo de pathfinding.
    fn create_pathfinder(&self) -> Box<dyn PathfindingAlgorithm>;

    /// Cria a fábrica de agentes azuis.
    fn create_blue_agent_factory(&self) -> Box<dyn AgentFactory>;

    /// Cria a fábrica de agentes azuis.
    fn create_red_agent_factory(&self) -> Box<dyn AgentFactory>;
}

/// Implementação Concreta: Simulação Padrão (4-Direções, Agentes Azuis).
/// Esta fábrica cria um conjunto de componentes que trabalham juntos.
pub struct CardinalSimulationFactory {
    // A Fábrica Abstrata mantém referências às Fábricas Concretas que irá usar.
    grid_factory: RectangularGridFactory,
}

// O construtor é necessário para inicializar as fábricas internas.
impl CardinalSimulationFactory {
    pub fn new() -> Self {
        Self {
            grid_factory: RectangularGridFactory,
        }
    }
}

impl SimulationFactory for CardinalSimulationFactory {
    fn create_grid(&self, width: usize, height: usize) -> Grid {
        // Delega a criação para a fábrica de Grid
        self.grid_factory.create(width, height)
    }

    fn create_pathfinder(&self) -> Box<dyn PathfindingAlgorithm> {
        // Retorna o A* de 4 direções
        Box::new(AStarCardinal)
    }

    fn create_blue_agent_factory(&self) -> Box<dyn AgentFactory> {
        // Retorna a fábrica que cria Agentes Azuis
        Box::new(BlueAgentFactory)
    }

    fn create_red_agent_factory(&self) -> Box<dyn AgentFactory> {
        // Retorna a fábrica que cria Agentes Vermelhos
        Box::new(RedAgentFactory)
    }
}
