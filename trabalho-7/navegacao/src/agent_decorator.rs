use crate::agent::Agent;
use macroquad::prelude::*;

/// 📜 Trait base para Agentes e Decorators.
/// Todos os componentes (Agente base ou decorado) devem implementar esta interface.
pub trait AgentComponent {
    fn update(&mut self, dt: f32);
    fn get_color(&self) -> Color;
    fn get_pos(&self) -> Vec2;
    fn is_finished(&self) -> bool;
}

/// 🚀 Decorator Concreto: Aumento de Velocidade
/// Adiciona a funcionalidade de alterar a velocidade do agente base.
pub struct SpeedBoostDecorator {
    /// O componente que está sendo decorado (Agent ou outro Decorator)
    pub component: Box<dyn AgentComponent>,
    /// Multiplicador de velocidade (2.0 para o dobro da velocidade)
    speed_multiplier: f32,
    /// Flag para rastrear se o boost está ativo
    is_boost_active: bool,
}

impl SpeedBoostDecorator {
    pub fn new(agent: Agent, multiplier: f32) -> Self {
        Self {
            // Empacota o agente base em um Box para o trait object
            component: Box::new(agent),
            speed_multiplier: multiplier,
            is_boost_active: true,
        }
    }

    // Método para ativar/desativar o boost em tempo real.
    // pub fn toggle_boost(&mut self) {
    //     self.is_boost_active = !self.is_boost_active;
    // }
}

/// Implementação do Trait AgentComponent para o Decorator
impl AgentComponent for SpeedBoostDecorator {
    fn update(&mut self, dt: f32) {
        // Se o boost está ativo, altera o Delta Time (dt) efetivo.
        let effective_dt = if self.is_boost_active {
            dt * self.speed_multiplier
        } else {
            dt
        };

        // Delega a chamada para o componente base usando o novo dt.
        self.component.update(effective_dt);
    }

    // Métodos delegados
    fn get_color(&self) -> Color {
        self.component.get_color()
    }

    fn get_pos(&self) -> Vec2 {
        self.component.get_pos()
    }

    fn is_finished(&self) -> bool {
        self.component.is_finished()
    }
}
