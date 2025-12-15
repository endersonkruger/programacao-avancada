use crate::observer::Observer;
use macroquad::prelude::*;

/// Trait base para Agentes e Decorators.
/// Atualizado para suportar Command, Observer e Detecção de Colisão
pub trait AgentComponent {
    // --- Métodos Básicos (Existentes) ---
    fn update(&mut self, dt: f32);
    fn get_color(&self) -> Color;
    fn get_pos(&self) -> Vec2;
    fn is_finished(&self) -> bool;

    // --- Métodos para o Command Pattern ---
    fn set_pos(&mut self, pos: Vec2);
    fn get_id(&self) -> usize;
    fn get_next_step_target(&self) -> Option<Vec2>;

    // --- Métodos para o Observer e Estado ---
    fn consume_fuel(&mut self, amount: f32);
    fn restore_fuel(&mut self, amount: f32);
    fn add_observer(&mut self, observer: Box<dyn Observer>);

    // --- [NOVO] Métodos para Detecção de Colisão (Trabalho 9) ---
    /// Retorna o raio físico do agente (para colisão real)
    fn get_physical_radius(&self) -> f32;

    /// Retorna o raio do sensor (para evitar colisão antes que aconteça)
    fn get_detection_radius(&self) -> f32;

    /// Permite disparar eventos externamente (usado pelo Main na detecção)
    fn notify(&self, event: crate::observer::AgentEvent);
}

/// Decorator Concreto: Aumento de Velocidade
pub struct SpeedBoostDecorator {
    pub component: Box<dyn AgentComponent>,
    speed_multiplier: f32,
    is_boost_active: bool,
}

impl SpeedBoostDecorator {
    pub fn new(component: Box<dyn AgentComponent>, multiplier: f32) -> Self {
        Self {
            component,
            speed_multiplier: multiplier,
            is_boost_active: true,
        }
    }
}

impl AgentComponent for SpeedBoostDecorator {
    fn update(&mut self, dt: f32) {
        let effective_dt = if self.is_boost_active {
            dt * self.speed_multiplier
        } else {
            dt
        };
        self.component.update(effective_dt);
    }

    fn get_color(&self) -> Color {
        self.component.get_color()
    }

    fn get_pos(&self) -> Vec2 {
        self.component.get_pos()
    }

    fn is_finished(&self) -> bool {
        self.component.is_finished()
    }

    fn set_pos(&mut self, pos: Vec2) {
        self.component.set_pos(pos);
    }

    fn get_id(&self) -> usize {
        self.component.get_id()
    }

    fn get_next_step_target(&self) -> Option<Vec2> {
        self.component.get_next_step_target()
    }

    fn consume_fuel(&mut self, amount: f32) {
        self.component.consume_fuel(amount);
    }

    fn restore_fuel(&mut self, amount: f32) {
        self.component.restore_fuel(amount);
    }

    fn add_observer(&mut self, observer: Box<dyn Observer>) {
        self.component.add_observer(observer);
    }

    // --- Pass-through dos novos métodos ---

    fn get_physical_radius(&self) -> f32 {
        self.component.get_physical_radius()
    }

    fn get_detection_radius(&self) -> f32 {
        self.component.get_detection_radius()
    }

    fn notify(&self, event: crate::observer::AgentEvent) {
        self.component.notify(event);
    }
}
