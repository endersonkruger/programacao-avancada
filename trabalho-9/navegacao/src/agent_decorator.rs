use crate::observer::{AgentEvent, Observer};
use macroquad::prelude::*;
use std::cell::RefCell;

/// Trait base para Agentes e Decorators.
pub trait AgentComponent {
    // --- Métodos Básicos ---
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

    // --- Métodos para Detecção de Colisão ---
    fn get_physical_radius(&self) -> f32;
    fn get_detection_radius(&self) -> f32;

    // --- Método para receber notificações ---
    fn notify(&self, event: AgentEvent);

    // --- Método para pegar a cor do raio visual ---
    fn get_detection_color(&self) -> Color {
        Color::new(1.0, 1.0, 0.0, 0.3)
    }
}

/// --- DECORATOR 1: SpeedBoostDecorator ---
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

    // Pass-throughs (Repassa para o componente interno)
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
    fn get_physical_radius(&self) -> f32 {
        self.component.get_physical_radius()
    }
    fn get_detection_radius(&self) -> f32 {
        self.component.get_detection_radius()
    }
    fn notify(&self, event: AgentEvent) {
        self.component.notify(event);
    }
    fn get_detection_color(&self) -> Color {
        self.component.get_detection_color()
    }
}

/// --- DECORATOR 2: VisualAlertDecorator ---
pub struct VisualAlertDecorator {
    component: Box<dyn AgentComponent>,
    state: RefCell<(f32, Color)>,
}

impl VisualAlertDecorator {
    pub fn new(component: Box<dyn AgentComponent>) -> Self {
        Self {
            component,
            state: RefCell::new((0.0, GREEN)), // Inicia sem alerta
        }
    }
}

impl AgentComponent for VisualAlertDecorator {
    fn update(&mut self, dt: f32) {
        // Atualiza o timer visual
        let mut state = self.state.borrow_mut();
        if state.0 > 0.0 {
            state.0 -= dt;
        }
        // Atualiza o componente interno
        self.component.update(dt);
    }

    fn notify(&self, event: AgentEvent) {
        // Intercepta eventos de colisão para mudar a cor
        match event {
            AgentEvent::CollisionHit(_) => {
                // Colisão real: Vermelho por 0.5s
                *self.state.borrow_mut() = (0.5, RED);
            }
            AgentEvent::ProximityAlert(_) => {
                // Alerta: Laranja por 0.1s (apenas se não estiver vermelho)
                let mut state = self.state.borrow_mut();
                if state.1 != RED || state.0 <= 0.0 {
                    *state = (0.1, ORANGE);
                }
            }
            _ => {}
        }
        // Repassa o evento (para os logs funcionarem)
        self.component.notify(event);
    }

    fn get_detection_color(&self) -> Color {
        let state = self.state.borrow();
        // Se o timer > 0, retorna cor de alerta. Senão, verde transparente.
        if state.0 > 0.0 {
            state.1
        } else {
            Color::new(0.0, 1.0, 0.0, 0.2)
        }
    }

    // Pass-throughs obrigatórios
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
    fn consume_fuel(&mut self, a: f32) {
        self.component.consume_fuel(a);
    }
    fn restore_fuel(&mut self, a: f32) {
        self.component.restore_fuel(a);
    }
    fn add_observer(&mut self, obs: Box<dyn Observer>) {
        self.component.add_observer(obs);
    }
    fn get_physical_radius(&self) -> f32 {
        self.component.get_physical_radius()
    }
    fn get_detection_radius(&self) -> f32 {
        self.component.get_detection_radius()
    }
}
