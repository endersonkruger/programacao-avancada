use crate::observer::{AgentEvent, Observer};
use crate::pheromone::PheromoneManager;
use macroquad::prelude::*;
use std::cell::RefCell;

/// Trait base para Agentes e Decorators.
pub trait AgentComponent {
    fn update(&mut self, dt: f32);
    fn get_color(&self) -> Color;
    fn get_pos(&self) -> Vec2;
    fn is_finished(&self) -> bool;
    fn set_pos(&mut self, pos: Vec2);
    fn get_id(&self) -> usize;
    fn get_next_step_target(&self) -> Option<Vec2>;
    fn consume_fuel(&mut self, amount: f32);
    fn restore_fuel(&mut self, amount: f32);
    fn add_observer(&mut self, observer: Box<dyn Observer>);
    fn get_physical_radius(&self) -> f32;
    fn get_detection_radius(&self) -> f32;
    fn notify(&self, event: AgentEvent);
    fn get_detection_color(&self) -> Color {
        Color::new(1.0, 1.0, 0.0, 0.3)
    }
}

// --- DECORATOR 1: SpeedBoostDecorator ---
pub struct SpeedBoostDecorator {
    pub component: Box<dyn AgentComponent>,
    base_multiplier: f32,
    state: RefCell<(f32, f32)>,
}
impl SpeedBoostDecorator {
    pub fn new(component: Box<dyn AgentComponent>, base_multiplier: f32) -> Self {
        Self { component, base_multiplier, state: RefCell::new((0.0, base_multiplier)) }
    }
}
impl AgentComponent for SpeedBoostDecorator {
    fn update(&mut self, dt: f32) {
        let mut state = self.state.borrow_mut();
        if state.0 > 0.0 {
            state.0 -= dt;
            if state.0 <= 0.0 { state.1 = self.base_multiplier; }
        }
        self.component.update(dt * state.1);
    }
    fn notify(&self, event: AgentEvent) {
        if let AgentEvent::ProximityAlert(_) = event {
            let mut state = self.state.borrow_mut();
            if state.0 <= 0.0 {
                *state = (rand::gen_range(0.2, 0.5), rand::gen_range(0.5, 1.4));
            }
        }
        self.component.notify(event);
    }
    fn get_color(&self) -> Color { self.component.get_color() }
    fn get_pos(&self) -> Vec2 { self.component.get_pos() }
    fn is_finished(&self) -> bool { self.component.is_finished() }
    fn set_pos(&mut self, pos: Vec2) { self.component.set_pos(pos); }
    fn get_id(&self) -> usize { self.component.get_id() }
    fn get_next_step_target(&self) -> Option<Vec2> { self.component.get_next_step_target() }
    fn consume_fuel(&mut self, a: f32) { self.component.consume_fuel(a); }
    fn restore_fuel(&mut self, a: f32) { self.component.restore_fuel(a); }
    fn add_observer(&mut self, obs: Box<dyn Observer>) { self.component.add_observer(obs); }
    fn get_physical_radius(&self) -> f32 { self.component.get_physical_radius() }
    fn get_detection_radius(&self) -> f32 { self.component.get_detection_radius() }
    fn get_detection_color(&self) -> Color { self.component.get_detection_color() }
}

// --- DECORATOR 2: DirectionDeviateDecorator ---
pub struct DirectionDeviateDecorator {
    component: Box<dyn AgentComponent>,
    state: RefCell<(f32, Vec2)>,
}
impl DirectionDeviateDecorator {
    pub fn new(component: Box<dyn AgentComponent>) -> Self {
        Self { component, state: RefCell::new((0.0, vec2(0.0, 0.0))) }
    }
}
impl AgentComponent for DirectionDeviateDecorator {
    fn update(&mut self, dt: f32) {
        let mut state = self.state.borrow_mut();
        if state.0 > 0.0 { state.0 -= dt; }
        self.component.update(dt);
    }
    fn notify(&self, event: AgentEvent) {
        if let AgentEvent::ProximityAlert(_) = event {
            let mut state = self.state.borrow_mut();
            if state.0 <= 0.0 {
                *state = (rand::gen_range(0.1, 0.3), vec2(rand::gen_range(-2.0, 2.0), rand::gen_range(-2.0, 2.0)));
            }
        }
        self.component.notify(event);
    }
    fn get_next_step_target(&self) -> Option<Vec2> {
        let original = self.component.get_next_step_target();
        if let Some(target) = original {
            let state = self.state.borrow();
            if state.0 > 0.0 { return Some(target + state.1); }
            return Some(target);
        }
        None
    }
    fn get_color(&self) -> Color { self.component.get_color() }
    fn get_pos(&self) -> Vec2 { self.component.get_pos() }
    fn is_finished(&self) -> bool { self.component.is_finished() }
    fn set_pos(&mut self, pos: Vec2) { self.component.set_pos(pos); }
    fn get_id(&self) -> usize { self.component.get_id() }
    fn consume_fuel(&mut self, a: f32) { self.component.consume_fuel(a); }
    fn restore_fuel(&mut self, a: f32) { self.component.restore_fuel(a); }
    fn add_observer(&mut self, obs: Box<dyn Observer>) { self.component.add_observer(obs); }
    fn get_physical_radius(&self) -> f32 { self.component.get_physical_radius() }
    fn get_detection_radius(&self) -> f32 { self.component.get_detection_radius() }
    fn get_detection_color(&self) -> Color { self.component.get_detection_color() }
}

// --- DECORATOR 3: VisualAlertDecorator ---
pub struct VisualAlertDecorator {
    component: Box<dyn AgentComponent>,
    state: RefCell<(f32, Color)>,
}
impl VisualAlertDecorator {
    pub fn new(component: Box<dyn AgentComponent>) -> Self {
        Self { component, state: RefCell::new((0.0, GREEN)) }
    }
}
impl AgentComponent for VisualAlertDecorator {
    fn update(&mut self, dt: f32) {
        let mut state = self.state.borrow_mut();
        if state.0 > 0.0 { state.0 -= dt; }
        self.component.update(dt);
    }
    fn notify(&self, event: AgentEvent) {
        match event {
            AgentEvent::CollisionHit(_) => { *self.state.borrow_mut() = (0.5, RED); }
            AgentEvent::ProximityAlert(_) => {
                let mut state = self.state.borrow_mut();
                if state.1 != RED || state.0 <= 0.0 { *state = (0.1, ORANGE); }
            }
            _ => {}
        }
        self.component.notify(event);
    }
    fn get_detection_color(&self) -> Color {
        let state = self.state.borrow();
        if state.0 > 0.0 { state.1 } else { Color::new(0.0, 1.0, 0.0, 0.2) }
    }
    fn get_color(&self) -> Color { self.component.get_color() }
    fn get_pos(&self) -> Vec2 { self.component.get_pos() }
    fn is_finished(&self) -> bool { self.component.is_finished() }
    fn set_pos(&mut self, pos: Vec2) { self.component.set_pos(pos); }
    fn get_id(&self) -> usize { self.component.get_id() }
    fn get_next_step_target(&self) -> Option<Vec2> { self.component.get_next_step_target() }
    fn consume_fuel(&mut self, a: f32) { self.component.consume_fuel(a); }
    fn restore_fuel(&mut self, a: f32) { self.component.restore_fuel(a); }
    fn add_observer(&mut self, obs: Box<dyn Observer>) { self.component.add_observer(obs); }
    fn get_physical_radius(&self) -> f32 { self.component.get_physical_radius() }
    fn get_detection_radius(&self) -> f32 { self.component.get_detection_radius() }
}

/// --- DECORATOR 4: IndirectCommunicationDecorator ---
pub struct IndirectCommunicationDecorator {
    component: Box<dyn AgentComponent>,
    grid_mode: crate::GridMode,
}

impl IndirectCommunicationDecorator {
    pub fn new(component: Box<dyn AgentComponent>, grid_mode: crate::GridMode) -> Self {
        Self { component, grid_mode }
    }
}

impl AgentComponent for IndirectCommunicationDecorator {
    fn update(&mut self, dt: f32) {
        // 1. ESCRITA: Deposita feromônio na posição atual
        let pos = self.component.get_pos();
        PheromoneManager::instance().deposit(pos, crate::CELL_SIZE, self.grid_mode);
        self.component.update(dt);
    }

    fn get_next_step_target(&self) -> Option<Vec2> {
        let target_opt = self.component.get_next_step_target();

        if let Some(target) = target_opt {
            let current_pos = self.component.get_pos();
            let (current_gx, current_gy) = crate::screen_to_grid(current_pos.x, current_pos.y, self.grid_mode);
            let (target_gx, target_gy) = crate::screen_to_grid(target.x, target.y, self.grid_mode);

            // Só checa bloqueio se estiver tentando mudar de célula isso evita que o agente se bloqueie com seu próprio rastro
            if (target_gx != current_gx || target_gy != current_gy) {
                if PheromoneManager::instance().is_blocked(target_gx, target_gy) {
                    // Célula ocupada: Aciona o ProximityAlert e não move
                    self.notify(AgentEvent::ProximityAlert(9999));
                    return None;
                }
            }
            
            return Some(target);
        }

        None
    }
    // Pass-throughs
    fn get_color(&self) -> Color { self.component.get_color() }
    fn get_pos(&self) -> Vec2 { self.component.get_pos() }
    fn is_finished(&self) -> bool { self.component.is_finished() }
    fn set_pos(&mut self, pos: Vec2) { self.component.set_pos(pos); }
    fn get_id(&self) -> usize { self.component.get_id() }
    fn consume_fuel(&mut self, a: f32) { self.component.consume_fuel(a); }
    fn restore_fuel(&mut self, a: f32) { self.component.restore_fuel(a); }
    fn add_observer(&mut self, obs: Box<dyn Observer>) { self.component.add_observer(obs); }
    fn get_physical_radius(&self) -> f32 { self.component.get_physical_radius() }
    fn get_detection_radius(&self) -> f32 { self.component.get_detection_radius() }
    fn get_detection_color(&self) -> Color { self.component.get_detection_color() }
    fn notify(&self, event: AgentEvent) { self.component.notify(event); }
}