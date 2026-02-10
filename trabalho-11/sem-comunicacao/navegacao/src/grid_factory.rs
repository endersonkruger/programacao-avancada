use crate::grid::Grid;

/// Contrato (Trait) para qualquer fábrica responsável por criar estruturas de Grid.
/// Isso permite a criação de diferentes tipos de grid sem alterar o código que os utiliza.
pub trait GridFactory {
    /// Cria e retorna uma nova instância de Grid.
    fn create(&self, width: usize, height: usize) -> Grid;
}

/// Implementação Concreta: Fábrica para o Grid Retangular Padrão.
pub struct RectangularGridFactory;

impl GridFactory for RectangularGridFactory {
    fn create(&self, width: usize, height: usize) -> Grid {
        // Simplesmente chama o construtor do Grid existente.
        RectangularGridFactory::new(width, height)
    }
}

// Para manter o encapsulamento, definimos o construtor aqui
impl RectangularGridFactory {
    pub fn new(width: usize, height: usize) -> Grid {
        Grid::new(width, height)
    }
}
