use crate::agent::Agent;
use macroquad::prelude::*;

/// Contrato (Trait) para qualquer fábrica responsável por criar agentes.
pub trait AgentFactory {
    /// Cria e retorna uma nova instância de Agent, delegando a responsabilidade
    /// de definir as características (como a cor) à implementação concreta.
    fn create_agent(&self, start_pos: Vec2, path: Vec<Vec2>, speed: f32) -> Agent;
}

// --- Fábricas Concretas ---

/// Fábrica para criar Agentes Azuis.
pub struct BlueAgentFactory;

impl AgentFactory for BlueAgentFactory {
    fn create_agent(&self, start_pos: Vec2, path: Vec<Vec2>, speed: f32) -> Agent {
        // Usa a cor AZUL sólida na criação
        Agent::new(start_pos, path, speed, BLUE)
    }
}

/// Fábrica para criar Agentes Vermelhos.
pub struct RedAgentFactory;

impl AgentFactory for RedAgentFactory {
    fn create_agent(&self, start_pos: Vec2, path: Vec<Vec2>, speed: f32) -> Agent {
        // Usa a cor VERMELHA sólida na criação
        Agent::new(start_pos, path, speed, RED)
    }
}
