use crate::agent::Agent;
use macroquad::prelude::*;

/// Contrato (Trait) para qualquer fábrica responsável por criar agentes.
pub trait AgentFactory {
    /// Cria e retorna uma nova instância de Agent.
    /// Atualizado para receber 'id' necessário para o Command/Observer pattern.
    fn create_agent(&self, start_pos: Vec2, path: Vec<Vec2>, speed: f32, id: usize) -> Agent;
}

// --- Fábricas Concretas ---

/// Fábrica para criar Agentes Azuis.
pub struct BlueAgentFactory;

impl AgentFactory for BlueAgentFactory {
    fn create_agent(&self, start_pos: Vec2, path: Vec<Vec2>, speed: f32, id: usize) -> Agent {
        // Passa o ID e a cor AZUL para o construtor do Agente
        Agent::new(id, start_pos, path, speed, BLUE)
    }
}

/// Fábrica para criar Agentes Vermelhos.
pub struct RedAgentFactory;

impl AgentFactory for RedAgentFactory {
    fn create_agent(&self, start_pos: Vec2, path: Vec<Vec2>, speed: f32, id: usize) -> Agent {
        // Passa o ID e a cor VERMELHA para o construtor do Agente
        Agent::new(id, start_pos, path, speed, RED)
    }
}
