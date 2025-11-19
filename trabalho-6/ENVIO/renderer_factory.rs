// Sugestão do Gemini

// Contrato Abstrato
pub trait RendererFactory {
    fn create_grid_drawer() -> Box<dyn GridDrawer>;
    fn create_agent_drawer() -> Box<dyn AgentDrawer>;
    fn create_hud_drawer() -> Box<dyn HudDrawer>;
}

// Implementação Concreta para Macroquad
pub struct MacroquadRendererFactory;

impl RendererFactory for MacroquadRendererFactory {
    // ... Implementações que usam macroquad::prelude::*
}
