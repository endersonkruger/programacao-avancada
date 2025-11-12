use macroquad::prelude::*;

pub struct Agent {
    pub pos: Vec2,           // Posição atual em pixels
    path: Vec<Vec2>,         // Caminho a seguir (em pixels)
    current_waypoint: usize, // Índice do próximo ponto no caminho
    speed: f32,              // Velocidade de movimento
    pub is_finished: bool,   // Se o agente chegou ao destino
}

impl Agent {
    pub fn new(start_pos: Vec2, path: Vec<Vec2>, speed: f32) -> Self {
        Self {
            pos: start_pos,
            path,
            current_waypoint: 0,
            speed,
            is_finished: false,
        }
    }

    /// Atualiza a posição do agente
    pub fn update(&mut self, dt: f32) {
        if self.is_finished {
            return;
        }

        // Se não há mais pontos no caminho, marca como finalizado
        if self.current_waypoint >= self.path.len() {
            self.is_finished = true;
            return;
        }

        // Pega o próximo ponto do caminho
        let target = self.path[self.current_waypoint];
        let distance_to_target = self.pos.distance(target);

        // Se está perto o suficiente, avança para o próximo ponto
        // Usamos um limiar pequeno para evitar "tremer"
        if distance_to_target < 5.0 {
            self.current_waypoint += 1;
        } else {
            // Calcula a direção e move o agente
            let direction = (target - self.pos).normalize_or_zero();
            self.pos += direction * self.speed * dt;
        }
    }
}
