use macroquad::prelude::*;

/// Configurações do algoritmo RVO
const NEIGHBOR_DIST: f32 = 60.0; // Distância de visão
const TIME_HORIZON: f32 = 2.5;   // Tempo de antecipação
const RADIUS_MARGIN: f32 = 2.0;  // Margem pessoal padrão

pub struct AgentRvoState {
    pub id: usize,
    pub pos: Vec2,
    pub velocity: Vec2,
    pub radius: f32,
    pub max_speed: f32,
    pub pref_velocity: Vec2,
}

pub struct RvoManager;

impl RvoManager {
    pub fn compute_safe_velocity(agent: &AgentRvoState, neighbors: &[AgentRvoState]) -> Vec2 {
        // Se a intenção é ficar parado, retorna zero
        if agent.pref_velocity.length_squared() < 0.01 {
            return Vec2::ZERO;
        }

        let mut best_velocity = Vec2::ZERO;
        let mut min_penalty = f32::MAX;

        // --- GERAÇÃO DE CANDIDATOS ---
        let mut candidates = Vec::with_capacity(32);

        // 1. Otimista (Velocidade desejada)
        candidates.push(agent.pref_velocity);

        let speed = agent.pref_velocity.length();
        let base_angle = agent.pref_velocity.y.atan2(agent.pref_velocity.x);

        // Gera ângulos de desvio
        let angles = [
            10.0f32, -10.0,
            25.0, -25.0,
            45.0, -45.0, 
            70.0, -70.0, 
            90.0, -90.0,
            110.0, -110.0 // Tenta voltar um pouco se estiver muito bloqueado
        ];

        for &deg in &angles {
            let rad = deg.to_radians();
            let new_angle = base_angle + rad;
            
            // Velocidade total
            candidates.push(vec2(new_angle.cos(), new_angle.sin()) * speed);
            // Meia velocidade (frear para manobrar)
            candidates.push(vec2(new_angle.cos(), new_angle.sin()) * (speed * 0.5));
             // Velocidade muito baixa (quase parando para esperar)
            candidates.push(vec2(new_angle.cos(), new_angle.sin()) * (speed * 0.1));
        }

        // Ficar parado é a última opção
        candidates.push(Vec2::ZERO);

        // --- AVALIAÇÃO ---
        for cand_vel in candidates {
            let penalty = Self::evaluate_velocity(agent, cand_vel, neighbors);
            if penalty < min_penalty {
                min_penalty = penalty;
                best_velocity = cand_vel;
            }
        }

        best_velocity
    }

    fn evaluate_velocity(me: &AgentRvoState, cand_vel: Vec2, neighbors: &[AgentRvoState]) -> f32 {
        // 1. Custo base: Desvio da intenção original
        let dist_to_pref = me.pref_velocity.distance(cand_vel);
        let mut penalty = dist_to_pref; 

        // Penaliza ficar parado se o objetivo é andar (evita inércia excessiva)
        if cand_vel.length_squared() < 0.1 {
            penalty += 50.0;
        }

        // 2. Custo de Colisão
        for other in neighbors {
            if other.id == me.id { continue; }

            let dist_sq = me.pos.distance_squared(other.pos);
            if dist_sq > (NEIGHBOR_DIST * NEIGHBOR_DIST) { continue; }

            // --- TRATAMENTO DE AGENTES PARADOS ---
            // Se o outro está parado (speed < 0.1), ele age como uma parede.
            // Aumenta a margem de segurança para forçar o desvio mais cedo.
            let other_is_static = other.velocity.length_squared() < 0.1;
            
            let effective_margin = if other_is_static {
                RADIUS_MARGIN * 2.5 // Margem muito maior se o outro estiver parado
            } else {
                RADIUS_MARGIN
            };

            let combined_radius = me.radius + other.radius + effective_margin;
            
            let rel_pos = other.pos - me.pos;
            let rel_vel = cand_vel - other.velocity; 
            
            let dist_curr = rel_pos.length();

            // Colisão Imediata (Sobreposição)
            if dist_curr < combined_radius {
                penalty += 100000.0; // Inaceitável
                continue;
            }

            // Previsão de Colisão (Ray Casting no Cone de Velocidade)
            let rel_vel_sq = rel_vel.length_squared();

            if rel_vel_sq > 0.0001 {
                // Tempo até o ponto mais próximo
                let t = rel_pos.dot(rel_vel) / rel_vel_sq;

                // Só nos importamos se a colisão for no futuro próximo
                if t > 0.0 && t < TIME_HORIZON {
                    let closest_point = rel_vel * t;
                    let dist_line = (rel_pos - closest_point).length();

                    if dist_line < combined_radius {
                        // Penalidade baseada no tempo.
                        // Se for bater AGORA (t pequeno), penalidade gigante.
                        // Se for bater daqui a pouco, penalidade alta.
                        
                        let impact_penalty = if other_is_static {
                            // Se o outro está parado, penaliza dobrado qualquer rota de colisão
                            10000.0 / (t + 0.05)
                        } else {
                            5000.0 / (t + 0.1)
                        };

                        penalty += impact_penalty;
                    }
                }
            }
        }

        penalty
    }
}