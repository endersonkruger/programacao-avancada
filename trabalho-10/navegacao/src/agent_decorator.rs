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
    base_multiplier: f32,
    state: RefCell<(f32, f32)>,
}

impl SpeedBoostDecorator {
    pub fn new(component: Box<dyn AgentComponent>, base_multiplier: f32) -> Self {
        Self {
            component,
            base_multiplier,
            state: RefCell::new((0.0, base_multiplier)),
        }
    }
}

impl AgentComponent for SpeedBoostDecorator {
    fn update(&mut self, dt: f32) {
        let mut state = self.state.borrow_mut();

        if state.0 > 0.0 {
            state.0 -= dt;
            if state.0 <= 0.0 {
                state.1 = self.base_multiplier;
            }
        }

        let current_multiplier = state.1;
        let effective_dt = dt * current_multiplier;

        self.component.update(effective_dt);
    }

    fn notify(&self, event: AgentEvent) {
        match event {
            AgentEvent::ProximityAlert(_) => {
                let mut state = self.state.borrow_mut();

                // Só ativa se já não estiver em pânico (para evitar resetar o timer a cada frame)
                if state.0 <= 0.0 {
                    // Duração curta: 0.2s a 0.5s
                    let duration = rand::gen_range(0.2, 0.5);

                    // Velocidade: 0.5xaté 1.4x
                    let random_speed = rand::gen_range(0.5, 1.4);

                    *state = (duration, random_speed);
                }
            }
            _ => {}
        }
        self.component.notify(event);
    }

    // Pass-throughs
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
    fn get_detection_color(&self) -> Color {
        self.component.get_detection_color()
    }
}

/// --- DECORATOR 2: DirectionDeviateDecorator ---
pub struct DirectionDeviateDecorator {
    component: Box<dyn AgentComponent>,
    state: RefCell<(f32, Vec2)>,
}

impl DirectionDeviateDecorator {
    pub fn new(component: Box<dyn AgentComponent>) -> Self {
        Self {
            component,
            state: RefCell::new((0.0, vec2(0.0, 0.0))),
        }
    }
}

impl AgentComponent for DirectionDeviateDecorator {
    fn update(&mut self, dt: f32) {
        let mut state = self.state.borrow_mut();
        if state.0 > 0.0 {
            state.0 -= dt;
        }
        self.component.update(dt);
    }

    fn notify(&self, event: AgentEvent) {
        match event {
            AgentEvent::ProximityAlert(_) => {
                let mut state = self.state.borrow_mut();
                if state.0 <= 0.0 {
                    // Duração do desvio: 0.1s a 0.3s
                    let duration = rand::gen_range(0.1, 0.3);

                    // Desvio : -2.0 a 2.0 pixels
                    let jx = rand::gen_range(-2.0, 2.0);
                    let jy = rand::gen_range(-2.0, 2.0);

                    *state = (duration, vec2(jx, jy));
                }
            }
            _ => {}
        }
        self.component.notify(event);
    }

    fn get_next_step_target(&self) -> Option<Vec2> {
        let original_target = self.component.get_next_step_target();

        if let Some(target) = original_target {
            let state = self.state.borrow();
            if state.0 > 0.0 {
                // Soma o pequeno vetor de tremor ao alvo
                return Some(target + state.1);
            }
            return Some(target);
        }
        None
    }

    // Pass-throughs
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
    fn get_detection_color(&self) -> Color {
        self.component.get_detection_color()
    }
}

/// --- DECORATOR 3: VisualAlertDecorator ---
pub struct VisualAlertDecorator {
    component: Box<dyn AgentComponent>,
    state: RefCell<(f32, Color)>,
}

impl VisualAlertDecorator {
    pub fn new(component: Box<dyn AgentComponent>) -> Self {
        Self {
            component,
            state: RefCell::new((0.0, GREEN)),
        }
    }
}

impl AgentComponent for VisualAlertDecorator {
    fn update(&mut self, dt: f32) {
        let mut state = self.state.borrow_mut();
        if state.0 > 0.0 {
            state.0 -= dt;
        }
        self.component.update(dt);
    }

    fn notify(&self, event: AgentEvent) {
        match event {
            AgentEvent::CollisionHit(_) => {
                *self.state.borrow_mut() = (0.5, RED);
            }
            AgentEvent::ProximityAlert(_) => {
                let mut state = self.state.borrow_mut();
                if state.1 != RED || state.0 <= 0.0 {
                    *state = (0.1, ORANGE);
                }
            }
            _ => {}
        }
        self.component.notify(event);
    }

    fn get_detection_color(&self) -> Color {
        let state = self.state.borrow();
        if state.0 > 0.0 {
            state.1
        } else {
            Color::new(0.0, 1.0, 0.0, 0.2)
        }
    }

    // Pass-throughs
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
