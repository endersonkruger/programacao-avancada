use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Gerenciador Singleton que mantém cache de caminhos calculados.
/// Garante que apenas uma instância exista durante toda a execução.
pub struct PathManager {
    /// Cache de caminhos: key = (start, end), value = caminho calculado
    cache: Mutex<HashMap<((usize, usize), (usize, usize)), Vec<(usize, usize)>>>,
}

impl PathManager {
    /// Retorna a instância única do PathManager (Singleton)
    pub fn instance() -> &'static PathManager {
        static INSTANCE: OnceLock<PathManager> = OnceLock::new();
        INSTANCE.get_or_init(|| PathManager {
            cache: Mutex::new(HashMap::new()),
        })
    }

    /// Busca um caminho no cache ou calcula se necessário
    pub fn get_or_calculate<F>(
        &self,
        start: (usize, usize),
        end: (usize, usize),
        calculator: F,
    ) -> Option<Vec<(usize, usize)>>
    where
        F: FnOnce() -> Option<Vec<(usize, usize)>>,
    {
        let key = (start, end);

        // Tenta buscar no cache primeiro
        {
            let cache = self.cache.lock().unwrap();
            if let Some(path) = cache.get(&key) {
                return Some(path.clone());
            }
        }

        // Cache miss - calcula o caminho
        if let Some(path) = calculator() {
            // Armazena no cache
            let mut cache = self.cache.lock().unwrap();
            cache.insert(key, path.clone());
            Some(path)
        } else {
            None
        }
    }

    /// Limpa o cache (útil quando o grid é modificado)
    pub fn clear_cache(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
        println!("Cache de caminhos limpo.");
    }
}
