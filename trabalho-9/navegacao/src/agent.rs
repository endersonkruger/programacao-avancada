use crate::agent_decorator::AgentComponent;
use crate::observer::{AgentEvent, Observer};
use macroquad::prelude::*;

// Constantes de tamanho para colisão
const PHYSICAL_RADIUS: f32 = 8.0; // Raio do corpo (Colisão real)
const DETECTION_RADIUS: f32 = 18.0; // Raio do sensor (Colisão prevista)

/// Representa uma entidade móvel que segue um caminho no grid.
pub struct Agent {
    pub id: usize,
    pub pos: Vec2,
    path: Vec<Vec2>,
    current_waypoint: usize,
    speed: f32,
    pub is_finished: bool,
    pub color: Color,
    pub fuel: f32,
    observers: Vec<Box<dyn Observer>>,
    current_step_size: f32,
}

impl Agent {
    pub fn new(id: usize, start_pos: Vec2, path: Vec<Vec2>, speed: f32, color: Color) -> Self {
        Self {
            id,
            pos: start_pos,
            path,
            current_waypoint: 0,
            speed,
            is_finished: false,
            color,
            fuel: 500.0,
            observers: Vec::new(),
            current_step_size: 0.0,
        }
    }

    fn notify_observers(&self, event: AgentEvent) {
        for obs in &self.observers {
            obs.on_notify(self.id, event.clone());
        }
    }
}

impl AgentComponent for Agent {
    fn update(&mut self, dt: f32) {
        self.current_step_size = self.speed * dt;

        if self.fuel <= 0.0 {
            if self.fuel > -1.0 {
                self.notify_observers(AgentEvent::OutOfFuel);
                self.fuel = -10.0;
            }
            return;
        }
    }

    fn get_next_step_target(&self) -> Option<Vec2> {
        if self.is_finished || self.fuel <= 0.0 {
            return None;
        }
        if self.current_waypoint >= self.path.len() {
            return None;
        }

        let target = self.path[self.current_waypoint];
        let distance = self.pos.distance(target);

        if distance < 5.0 {
            return if self.current_waypoint + 1 < self.path.len() {
                Some(self.path[self.current_waypoint + 1])
            } else {
                None
            };
        }

        let direction = (target - self.pos).normalize_or_zero();
        Some(self.pos + direction * self.current_step_size)
    }

    fn set_pos(&mut self, pos: Vec2) {
        self.pos = pos;

        if self.current_waypoint < self.path.len() {
            if self.pos.distance(self.path[self.current_waypoint]) < 5.0 {
                self.current_waypoint += 1;
                if self.current_waypoint >= self.path.len() {
                    self.is_finished = true;
                    self.notify_observers(AgentEvent::Finished);
                }
            }
        }
    }

    fn get_color(&self) -> Color {
        if self.fuel <= 0.0 {
            GRAY
        } else {
            self.color
        }
    }

    fn get_pos(&self) -> Vec2 {
        self.pos
    }
    fn is_finished(&self) -> bool {
        self.is_finished
    }
    fn get_id(&self) -> usize {
        self.id
    }

    fn consume_fuel(&mut self, amount: f32) {
        self.fuel -= amount;
    }
    fn restore_fuel(&mut self, amount: f32) {
        self.fuel += amount;
    }

    fn add_observer(&mut self, observer: Box<dyn Observer>) {
        self.observers.push(observer);
    }

    // --- Implementação dos novos métodos ---

    fn get_physical_radius(&self) -> f32 {
        PHYSICAL_RADIUS
    }

    fn get_detection_radius(&self) -> f32 {
        DETECTION_RADIUS
    }

    fn notify(&self, event: AgentEvent) {
        self.notify_observers(event);
    }
}
