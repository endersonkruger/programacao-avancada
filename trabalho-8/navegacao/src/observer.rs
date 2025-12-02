/// Eventos que podem ocorrer com um agente
#[derive(Debug, Clone)]
pub enum AgentEvent {
    OutOfFuel, // O agente ficou sem energia
    Finished,  // O agente chegou ao destino
}

/// Interface para quem quer escutar eventos (Observer)
pub trait Observer {
    fn on_notify(&self, agent_id: usize, event: AgentEvent);
}

/// Um Observer Concreto que gerencia o Respawn
pub struct RespawnHandler;

impl Observer for RespawnHandler {
    fn on_notify(&self, agent_id: usize, event: AgentEvent) {
        match event {
            AgentEvent::OutOfFuel => {
                println!(
                    "[OBSERVER] Agente {} ficou sem combustível! Solicitando Respawn.",
                    agent_id
                );
                // disparo do comando de respawn.
            }
            AgentEvent::Finished => {
                println!("[OBSERVER] Agente {} chegou ao destino.", agent_id);
            }
        }
    }
}
