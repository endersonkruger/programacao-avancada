use crate::agent_decorator::AgentComponent;
use crate::observer::{AgentEvent, Observer};
use macroquad::prelude::*;

const PHYSICAL_RADIUS: f32 = 8.0;
const DETECTION_RADIUS: f32 = 18.0;

pub struct Agent {
    pub id: usize,
    pub pos: Vec2,
    path: Vec<Vec2>,
    current_waypoint: usize,
    speed: f32, // Max speed
    velocity: Vec2, // Velocidade atual
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
            velocity: Vec2::ZERO,
            is_finished: false,
            color,
            fuel: 2000.0,
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
            // Zera a velocidade se acabar combustível
            self.velocity = Vec2::ZERO; 
            return;
        }

        // Verifica se chegou ao waypoint ATUAL (para avançar o index)
        if self.current_waypoint < self.path.len() {
            let target = self.path[self.current_waypoint];
            if self.pos.distance(target) < 10.0 {
                self.current_waypoint += 1;
                if self.current_waypoint >= self.path.len() {
                    self.is_finished = true;
                    self.velocity = Vec2::ZERO;
                    self.notify_observers(AgentEvent::Finished);
                }
            }
        }
    }

    // Retorna o alvo desejado (A* puro), ignorando colisões locais
    fn get_next_step_target(&self) -> Option<Vec2> {
        if self.is_finished || self.fuel <= 0.0 {
            return None;
        }
        if self.current_waypoint >= self.path.len() {
            return None;
        }
        Some(self.path[self.current_waypoint])
    }

    fn set_pos(&mut self, pos: Vec2) {
        self.pos = pos;
    }

    // --- Implementação RVO ---
    fn get_velocity(&self) -> Vec2 {
        self.velocity
    }

    fn set_velocity(&mut self, vel: Vec2) {
        self.velocity = vel;
    }

    fn get_max_speed(&self) -> f32 {
        self.speed
    }
    // -------------------------

    fn get_color(&self) -> Color {
        if self.fuel <= 0.0 { GRAY } else { self.color }
    }

    fn get_pos(&self) -> Vec2 { self.pos }
    fn is_finished(&self) -> bool { self.is_finished }
    fn get_id(&self) -> usize { self.id }
    fn consume_fuel(&mut self, amount: f32) { self.fuel -= amount; }
    fn restore_fuel(&mut self, amount: f32) { self.fuel += amount; }
    fn add_observer(&mut self, observer: Box<dyn Observer>) { self.observers.push(observer); }
    fn get_physical_radius(&self) -> f32 { PHYSICAL_RADIUS }
    fn get_detection_radius(&self) -> f32 { DETECTION_RADIUS }
    fn notify(&self, event: AgentEvent) { self.notify_observers(event); }
}