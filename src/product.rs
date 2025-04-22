// src/product.rs
use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

// TODO: (Verificar completado)
// ✓ Struct Product con métricas implementado
// ✓ Sistema de estados atómicos funcionando
// [ ] Opcional: Añadir validación de tiempos no negativos

// Status del producto (Enum para máquina de estados)
#[derive(Debug, Clone, Serialize)]
pub enum ProductStatus {
    Waiting,
    InProgress { station: String },
    InTransit { from: String, to: String },
    Completed,
    Dead,
}

// Estructura principal del producto
#[derive(Debug, Serialize)]
pub struct Product {
    pub id: u64,
    pub arrival_time: u64,
    pub station_logs: HashMap<String, (u64, u64)>,
    #[serde(skip_serializing)] 
    pub status: Arc<AtomicStatus>,
    pub metrics: Metrics,
}

// Métricas internas
#[derive(Debug, Serialize)]
pub struct Metrics {
    pub total_wait_time: u64,
    pub turnaround_time: Option<u64>,
}

// Wrapper atómico para el estado
#[derive(Debug)]
struct AtomicStatus(AtomicU64);

impl Product {
    /// Crea un nuevo producto con ID único
    pub fn new(id: u64, arrival_time: u64) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        
        Self {
            id,
            arrival_time,
            station_logs: HashMap::with_capacity(3),
            status: Arc::new(AtomicStatus::new(ProductStatus::Waiting)),
            metrics: Metrics {
                total_wait_time: 0,
                turnaround_time: None,
            },
        }
    }

    /// Registra entrada a una estación
    pub fn log_entry(&mut self, station: &str) {
        let now = current_timestamp();
        self.station_logs.entry(station.to_string())
            .and_modify(|e| e.0 = now)
            .or_insert((now, 0));
            
        self.set_status(ProductStatus::InProgress {
            station: station.to_string()
        });
    }

    /// Registra salida de una estación
    pub fn log_exit(&mut self, station: &str) {
        let now = current_timestamp();
        if let Some(entry) = self.station_logs.get_mut(station) {
            entry.1 = now;
        }
        
        self.finalize_metrics();
    }

    /// Mata/finaliza el producto
    pub fn kill(&mut self) {
        self.set_status(ProductStatus::Dead);
        self.finalize_metrics();
    }

    /// Calcula tiempo total de espera
    fn finalize_metrics(&mut self) {
        use crate::config::STATION_ORDER;
        let mut last_time = self.arrival_time;
        let mut wait = 0;
   
        // Recorre siempre en el orden configurado
        for &station in STATION_ORDER.iter() {
            if let Some(&(entry, exit)) = self.station_logs.get(station) {
                // espera = entry - last_time, si entry ≥ last_time
                if entry >= last_time {
                    wait += entry - last_time;
                }
                last_time = exit;
            }
        }
   
        self.metrics.total_wait_time = wait;
        self.metrics.turnaround_time = Some(last_time - self.arrival_time);
    }

    pub fn set_status(&self, status: ProductStatus) {
        self.status.store(status);
    }

    pub fn get_status(&self) -> ProductStatus {
        self.status.load()
    }
}

// Implementación atómica para el estado
impl AtomicStatus {
    fn new(status: ProductStatus) -> Self {
        Self(AtomicU64::new(status.to_u64()))
    }

    fn store(&self, status: ProductStatus) {
        self.0.store(status.to_u64(), Ordering::SeqCst); 
    }

    fn load(&self) -> ProductStatus {
        ProductStatus::from_u64(self.0.load(Ordering::SeqCst))
    }
}
// Obtiene timestamp actual en segundos
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

impl ProductStatus {
    // Creado para manejar el proceso de convertir el enum a un AtomicU64 y viceversa, hay que mapear manualmente las stations

    fn to_u64(&self) -> u64 {
        match self {
            ProductStatus::Waiting => 0,
            ProductStatus::InProgress { .. } => 1,
            ProductStatus::InTransit { .. } => 2,
            ProductStatus::Completed => 3,
            ProductStatus::Dead => 4,
        }

    }

    fn from_u64(value: u64) -> Self {
        match value {
            0 => ProductStatus::Waiting,
            1 => ProductStatus::InProgress {
                station: String::new(),
            },
            2 => ProductStatus::InTransit {
                from: String::new(),
                to: String::new(),
            },
            3 => ProductStatus::Completed,
            4 => ProductStatus::Dead,
            _ => panic!("Unvalid value for ProductStatus"),
        }
    }

}

// Tests unitarios (le pedí a deepseek que me los hiciera pero no entiendo muy bien cómo va la vara)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_lifecycle() {
        let mut p = Product::new(1, 1000);
        
        p.log_entry("Cutting");
        assert!(matches!(p.get_status(), ProductStatus::InProgress { .. }));
        
        p.log_exit("Cutting");
        assert!(p.metrics.total_wait_time > 0);
        
        p.kill();
        assert!(matches!(p.get_status(), ProductStatus::Dead));
        assert!(p.metrics.turnaround_time.is_some());
    }
}