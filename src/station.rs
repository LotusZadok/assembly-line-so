// src/station.rs
use crate::config::{
    ASSEMBLY_TIME, CUTTING_TIME, ENABLE_VERBOSE_LOGGING, PACKAGING_TIME,
    SIMULATION_SPEED_MULTIPLIER,
};
use crate::product::Product;
use crossbeam_channel::{Receiver, Sender};
use std::thread;
use std::time::{Duration, SystemTime};

// TODO: (Verificar completado)
// ✓ Lógica de procesamiento con sleep implementada
// ✓ Manejo de logs de entrada/salida
// [ ] Opcional: Añadir timeout para operaciones bloqueantes

pub fn start_station(
    name: String,
    input_rx: Receiver<Product>,
    output_tx: Sender<Product>,
    is_final_station: bool,
) {
    thread::spawn(move || {
        let processing_time = get_processing_time(&name);

        if ENABLE_VERBOSE_LOGGING {
            println!(
                "[{}] Iniciada - Tiempo procesamiento: {}s",
                name, processing_time
            );
        }

        loop {
            match input_rx.recv() {
                Ok(mut product) => {
                    // Registrar entrada
                    let entry_time = current_timestamp();
                    product.log_entry(&name);

                    if ENABLE_VERBOSE_LOGGING {
                        println!("[{}] Procesando producto {}...", name, product.id);
                    }

                    // Simular procesamiento
                    let adjusted_time =
                        (processing_time as f64 / SIMULATION_SPEED_MULTIPLIER) as u64;
                    thread::sleep(Duration::from_secs(adjusted_time));

                    // Registrar salida
                    product.log_exit(&name);

                    if ENABLE_VERBOSE_LOGGING {
                        println!(
                            "[{}] Producto {} procesado en {}s",
                            name,
                            product.id,
                            current_timestamp() - entry_time
                        );
                    }

                    // Manejar siguiente paso
                    if is_final_station {
                        product.kill();
                        // ¡Muy importante! enviarlo para que lo recoja el módulo de métricas
                        output_tx
                            .send(product)
                            .expect("Error enviando producto finalizado a métricas");
                    } else {
                        output_tx
                            .send(product)
                            .expect("Error enviando producto a la siguiente estación");
                    }
                }
                Err(_) => break, // Canal cerrado
            }
        }
    });
}

/// Obtiene el tiempo de procesamiento configurado para cada estación
fn get_processing_time(station_name: &str) -> u64 {
    match station_name {
        "Corte" => CUTTING_TIME,
        "Ensamblaje" => ASSEMBLY_TIME,
        "Empaque" => PACKAGING_TIME,
        _ => panic!("Estación desconocida: {}", station_name),
    }
}

/// Obtiene el timestamp actual en segundos
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

// Tests unitarios (le pedí a deepseek que me los hiciera pero no entiendo muy bien cómo va la vara)
#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam_channel::unbounded;

    #[test]
    fn test_station_processing() {
        let (tx, rx) = unbounded();
        let (next_tx, next_rx) = unbounded();

        start_station("Corte".to_string(), rx, next_tx, false);

        let mut product = Product::new(1, 0);
        tx.send(product).unwrap();

        let processed = next_rx.recv().unwrap();

        assert!(processed.station_logs.get("Corte").unwrap().1 > 0);
        assert_eq!(processed.get_status(), ProductStatus::EnTransito);
    }

    #[test]
    fn test_final_station() {
        let (tx, rx) = unbounded();
        let (_, next_rx) = unbounded();

        start_station("Empaque".to_string(), rx, next_rx, true);

        let mut product = Product::new(2, 0);
        tx.send(product).unwrap();

        thread::sleep(Duration::from_secs(1));
        assert!(matches!(product.get_status(), ProductStatus::Muerto));
    }
}
