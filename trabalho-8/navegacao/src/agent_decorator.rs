use crate::observer::Observer;
use macroquad::prelude::*;

/// Trait base para Agentes e Decorators.
/// Atualizado para suportar Command Pattern (movimentação controlada) e Observer Pattern (eventos).
pub trait AgentComponent {
    // --- Métodos Básicos (Existentes) ---
    fn update(&mut self, dt: f32);
    fn get_color(&self) -> Color;
    fn get_pos(&self) -> Vec2;
    fn is_finished(&self) -> bool;

    // --- Métodos para o Command Pattern ---
    /// Define a posição diretamente (usado pelo MoveCommand e Undo)
    fn set_pos(&mut self, pos: Vec2);

    /// Retorna o ID único do agente (para saber quem o comando afeta)
    fn get_id(&self) -> usize;

    /// Calcula onde o agente DESEJA ir no próximo passo (sem se mover ainda)
    fn get_next_step_target(&self) -> Option<Vec2>;

    // --- Métodos para o Observer Pattern e Estado ---
    /// Consome combustível/energia
    fn consume_fuel(&mut self, amount: f32);

    /// Restaura combustível (usado no Undo ou recarga)
    fn restore_fuel(&mut self, amount: f32);

    /// Registra um observador para escutar eventos deste agente
    fn add_observer(&mut self, observer: Box<dyn Observer>);
}

/// Decorator Concreto: Aumento de Velocidade
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
    /// Cria um novo decorator.
    /// Nota: Agora aceita Box<dyn AgentComponent> para permitir decorar outros decorators se necessário.
    pub fn new(component: Box<dyn AgentComponent>, multiplier: f32) -> Self {
        Self {
            component,
            speed_multiplier: multiplier,
            is_boost_active: true,
        }
    }
}

/// Implementação do Trait AgentComponent para o Decorator
/// O Decorator deve repassar (delegate) todas as chamadas para o componente interno,
/// modificando apenas o comportamento desejado (neste caso, o tempo/velocidade).
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

    // --- Pass-throughs (Delegações Diretas) ---

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
        // Opcional: Se a velocidade aumenta, o consumo poderia aumentar também?
        // Por enquanto, apenas repassa o valor original.
        self.component.consume_fuel(amount);
    }

    fn restore_fuel(&mut self, amount: f32) {
        self.component.restore_fuel(amount);
    }

    fn add_observer(&mut self, observer: Box<dyn Observer>) {
        self.component.add_observer(observer);
    }
}
