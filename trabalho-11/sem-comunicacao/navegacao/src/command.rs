use crate::agent_decorator::AgentComponent;
use macroquad::prelude::*;
use std::collections::VecDeque;

/// A interface Command
pub trait Command {
    /// Executa a ação (altera o estado do jogo)
    fn execute(&mut self, agents: &mut Vec<Box<dyn AgentComponent>>);
    /// Desfaz a ação (restaura o estado anterior)
    fn undo(&mut self, agents: &mut Vec<Box<dyn AgentComponent>>);
}

/// Comando Concreto: Mover Agente
/// Guarda o estado necessário para ir e voltar.
pub struct MoveCommand {
    agent_id: usize,
    old_pos: Vec2,
    new_pos: Vec2,
    timestamp: f64,
}

impl MoveCommand {
    pub fn new(agent_id: usize, old_pos: Vec2, new_pos: Vec2) -> Self {
        Self {
            agent_id,
            old_pos,
            new_pos,
            timestamp: get_time(),
        }
    }
}

impl Command for MoveCommand {
    fn execute(&mut self, agents: &mut Vec<Box<dyn AgentComponent>>) {
        if let Some(agent) = agents.get_mut(self.agent_id) {
            agent.set_pos(self.new_pos);
            // Simula gasto de combustível ao mover
            agent.consume_fuel(1.0);
        }
    }

    fn undo(&mut self, agents: &mut Vec<Box<dyn AgentComponent>>) {
        if let Some(agent) = agents.get_mut(self.agent_id) {
            agent.set_pos(self.old_pos);
            // Restaura combustível ao desfazer
            agent.restore_fuel(1.0);
        }
    }
}

/// Gerenciador de Comandos (Invoker)
pub struct CommandManager {
    history: Vec<Box<dyn Command>>,    // Pilha de undo
    queue: VecDeque<Box<dyn Command>>, // Fila de execução
}

impl CommandManager {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            queue: VecDeque::new(),
        }
    }

    /// Adiciona um comando à fila para ser executado
    pub fn add_command(&mut self, cmd: Box<dyn Command>) {
        self.queue.push_back(cmd);
    }

    /// Processa a fila de comandos (Executa tudo que está pendente)
    pub fn process_commands(&mut self, agents: &mut Vec<Box<dyn AgentComponent>>) {
        while let Some(mut cmd) = self.queue.pop_front() {
            cmd.execute(agents);
            self.history.push(cmd);
        }
    }

    /// Desfaz o último comando executado
    pub fn undo_last(&mut self, agents: &mut Vec<Box<dyn AgentComponent>>) {
        if let Some(mut cmd) = self.history.pop() {
            cmd.undo(agents);
            println!("Ação desfeita!");
        }
    }
}
