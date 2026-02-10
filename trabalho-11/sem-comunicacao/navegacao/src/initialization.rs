use crate::abstract_factory::{CardinalSimulationFactory, SimulationFactory};
use crate::grid::Grid;

/// Contexto compartilhado passado pela corrente
pub struct InitContext {
    pub grid: Option<Grid>,
    pub factory: Option<Box<dyn SimulationFactory>>,
    pub width: usize,
    pub height: usize,
}

/// Interface do Handler da Chain
pub trait InitHandler {
    fn set_next(&mut self, next: Box<dyn InitHandler>);
    fn handle(&mut self, context: &mut InitContext);
}

// --- Handler 1: Configuração Básica ---
pub struct ConfigInitHandler {
    next: Option<Box<dyn InitHandler>>,
}
impl ConfigInitHandler {
    pub fn new() -> Self {
        Self { next: None }
    }
}
impl InitHandler for ConfigInitHandler {
    fn set_next(&mut self, next: Box<dyn InitHandler>) {
        self.next = Some(next);
    }
    fn handle(&mut self, context: &mut InitContext) {
        println!("[CHAIN] Configurando simulação...");
        // Define a fábrica padrão
        context.factory = Some(Box::new(CardinalSimulationFactory::new()));

        if let Some(next) = &mut self.next {
            next.handle(context);
        }
    }
}

// --- Handler 2: Inicialização do Grid ---
pub struct GridInitHandler {
    next: Option<Box<dyn InitHandler>>,
}
impl GridInitHandler {
    pub fn new() -> Self {
        Self { next: None }
    }
}
impl InitHandler for GridInitHandler {
    fn set_next(&mut self, next: Box<dyn InitHandler>) {
        self.next = Some(next);
    }
    fn handle(&mut self, context: &mut InitContext) {
        println!("[CHAIN] Inicializando Grid...");
        if let Some(factory) = &context.factory {
            context.grid = Some(factory.create_grid(context.width, context.height));
        }

        if let Some(next) = &mut self.next {
            next.handle(context);
        }
    }
}

// --- Helper para montar a corrente ---
pub fn init_system(width: usize, height: usize) -> InitContext {
    let mut ctx = InitContext {
        grid: None,
        factory: None,
        width,
        height,
    };

    let mut step1 = ConfigInitHandler::new();
    let step2 = GridInitHandler::new();

    step1.set_next(Box::new(step2));

    // Executa a corrente
    step1.handle(&mut ctx);

    ctx
}
