use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// Gerenciador Singleton que mantém cache de caminhos calculados.
/// Garante que apenas uma instância exista durante toda a execução.
pub struct PathManager {
    /// Cache de caminhos: key = (start, end), value = caminho calculado
    cache: Mutex<HashMap<((usize, usize), (usize, usize)), Vec<(usize, usize)>>>,
    /// Estatísticas de uso
    stats: Mutex<PathStats>,
}

#[derive(Default)]
pub struct PathStats {
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub total_paths: usize,
}

impl PathManager {
    /// Retorna a instância única do PathManager (Singleton)
    pub fn instance() -> &'static PathManager {
        static INSTANCE: OnceLock<PathManager> = OnceLock::new();
        INSTANCE.get_or_init(|| PathManager {
            cache: Mutex::new(HashMap::new()),
            stats: Mutex::new(PathStats::default()),
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
                // Cache hit
                let mut stats = self.stats.lock().unwrap();
                stats.cache_hits += 1;
                return Some(path.clone());
            }
        }

        // Cache miss - calcula o caminho
        let mut stats = self.stats.lock().unwrap();
        stats.cache_misses += 1;
        drop(stats);

        if let Some(path) = calculator() {
            // Armazena no cache
            let mut cache = self.cache.lock().unwrap();
            cache.insert(key, path.clone());

            let mut stats = self.stats.lock().unwrap();
            stats.total_paths += 1;

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

    /// Retorna estatísticas de uso do cache
    pub fn get_stats(&self) -> PathStats {
        let stats = self.stats.lock().unwrap();
        PathStats {
            cache_hits: stats.cache_hits,
            cache_misses: stats.cache_misses,
            total_paths: stats.total_paths,
        }
    }

    /// Reseta as estatísticas
    pub fn reset_stats(&self) {
        let mut stats = self.stats.lock().unwrap();
        *stats = PathStats::default();
    }

    /// Retorna a taxa de acerto do cache (0.0 a 1.0)
    pub fn cache_hit_rate(&self) -> f32 {
        let stats = self.stats.lock().unwrap();
        let total = stats.cache_hits + stats.cache_misses;
        if total == 0 {
            0.0
        } else {
            stats.cache_hits as f32 / total as f32
        }
    }
}

impl PathStats {
    pub fn print(&self) {
        println!("=== Path Manager Stats ===");
        println!("Cache Hits: {}", self.cache_hits);
        println!("Cache Misses: {}", self.cache_misses);
        println!("Total Paths Stored: {}", self.total_paths);
        let total = self.cache_hits + self.cache_misses;
        if total > 0 {
            let hit_rate = (self.cache_hits as f32 / total as f32) * 100.0;
            println!("Cache Hit Rate: {:.1}%", hit_rate);
        }
        println!("========================");
    }
}
