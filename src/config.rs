// src/config.rs

// Podemos cambiar el tamaño a 32-bits, dependiendo de como vaya la vara

// -- Parámetros principales --
pub const PRODUCTS_PER_BATCH: usize = 10; // Mínimo 10 productos 
pub const DEFAULT_QUANTUM: u64 = 2; // Quantum por defecto para Round Robin (segundos)

// Tiempos de procesamiento para cada estación (en segundos)
pub const CUTTING_TIME: u64 = 1; // Estación 1: Corte
pub const ASSEMBLY_TIME: u64 = 5; // Estación 2: Ensamblaje
pub const PACKAGING_TIME: u64 = 2; // Estación 3: Empaque

// -- Tiempos de llegada (rango aleatorio) --
pub const MIN_ARRIVAL_TIME: u64 = 1;
pub const MAX_ARRIVAL_TIME: u64 = 5;

// -- Parámetros adicionales --
pub const SIMULATION_SPEED_MULTIPLIER: f64 = 1.0; // Escala el tiempo de ejecución (1.0 = real)
pub const ENABLE_VERBOSE_LOGGING: bool = true; // Habilita logging detallado
pub const RANDOM_SEED: Option<u64> = Some(42);

/// Orden estricto de las estaciones para métricas y reportes
pub const STATION_ORDER: [&str; 3] = ["Corte", "Ensamblaje", "Empaque"];
