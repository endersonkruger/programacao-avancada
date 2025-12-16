/// Eventos que podem ocorrer com um agente
#[derive(Debug, Clone)]
pub enum AgentEvent {
    OutOfFuel,             // O agente ficou sem energia
    Finished,              // O agente chegou ao destino
    ProximityAlert(usize), // Sensor detectou algo (Antes de bater)
    CollisionHit(usize),   // Colisão física (Durante a batida)
}

/// Interface para quem quer escutar eventos (Observer)
pub trait Observer {
    fn on_notify(&self, agent_id: usize, event: AgentEvent);
}

/// Um Observer Concreto que gerencia o Respawn e Logs de Colisão
pub struct RespawnHandler;

impl Observer for RespawnHandler {
    fn on_notify(&self, agent_id: usize, event: AgentEvent) {
        match event {
            AgentEvent::OutOfFuel => {
                println!(
                    "[OBSERVER] Agente {} ficou sem combustível! Solicitando Respawn.",
                    agent_id
                );
            }
            AgentEvent::Finished => {
                println!("[OBSERVER] Agente {} chegou ao destino.", agent_id);
            }
            AgentEvent::ProximityAlert(other_id) => {
                println!(
                    "[SENSOR] Agente {} detectou risco de colisão com Agente {}",
                    agent_id, other_id
                );
            }
            AgentEvent::CollisionHit(other_id) => {
                println!(
                    "[COLISÃO] Agente {} colidiu fisicamente com Agente {}",
                    agent_id, other_id
                );
            }
        }
    }
}
