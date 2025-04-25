// TODO: Control central
// [ ] Generar productos con tiempos de llegada aleatorios
// [ ] Distribuir productos a la cola inicial
// [ ] Coordinar inicio/detención de la simulación
// [ ] Recibir métricas de las estaciones

use crossbeam_channel::Sender;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::{
    config::{
        MAX_ARRIVAL_TIME, MIN_ARRIVAL_TIME, PRODUCTS_PER_BATCH, RANDOM_SEED,
        SIMULATION_SPEED_MULTIPLIER,
    },
    ipc_manager::send_product,
    product::Product,
};

/// Inicia la generación de productos en un hilo aparte.
/// Cada producto es enviado al canal global de entrada.
pub fn generate_products(tx: Arc<Mutex<Sender<Product>>>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        // Inicializar RNG para tiempos de llegada
        let mut rng = match RANDOM_SEED {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_entropy(),
        };

        for i in 1..=PRODUCTS_PER_BATCH as u64 {
            // Simular tiempo de llegada
            let sleep_secs = rng.gen_range(MIN_ARRIVAL_TIME..=MAX_ARRIVAL_TIME);
            let adjusted_sleep = (sleep_secs as f64 / SIMULATION_SPEED_MULTIPLIER) as u64;
            thread::sleep(Duration::from_secs(adjusted_sleep));

            // Crear producto con timestamp
            let arrival_time = current_timestamp();
            let product = Product::new(i, arrival_time);

            // Enviar al canal global
            let guard = tx.lock().expect("Mutex poisoned");
            if let Err(e) = send_product(&*guard, product) {
                eprintln!("[Controller] Error enviando producto {}: {}", i, e);
            }
        }
        // Tras generar todos los productos, caen los clones de Arc, cerrando el canal si no hay más senders.
    })
}

/// Retorna timestamp actual en segundos (UNIX epoch)
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
