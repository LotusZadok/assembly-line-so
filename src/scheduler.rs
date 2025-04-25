// src/scheduler.rs

// TODO: Implementar algoritmos de scheduling
// [ ] FCFS: Procesar productos en orden de llegada
// [ ] Round Robin: Implementar quantum con thread::sleep
// [ ] Manejar reencolado para RR cuando el quantum expira
// [ ] Integración con canales IPC (usar ipc_manager helpers)
// [ ] Garantizar sincronización entre estaciones

use crate::config::DEFAULT_QUANTUM;
use crate::product::Product;
use crate::station;
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use std::{
    collections::VecDeque,
    thread,
    time::{Duration, Instant},
};

/// Algoritmos soportados
pub enum SchedulingAlgorithm {
    FCFS,
    RoundRobin { quantum: Duration },
}

/// Arranca FCFS delegando en station::start_station
pub fn start_fcfs_station(
    name: String,
    input_rx: Receiver<Product>,
    output_tx: Sender<Product>,
    is_final_station: bool,
) {
    // FCFS no necesita más que procesar cada producto de una pasada
    station::start_station(name, input_rx, output_tx, is_final_station);
}

/// Arranca Round Robin con preemisión cada `quantum`
pub fn start_rr_station(
    name: String,
    input_rx: Receiver<Product>,
    output_tx: Sender<Product>,
    quantum: Duration,
    is_final_station: bool,
) {
    thread::spawn(move || {
        let total_time = get_station_time(&name);
        let mut queue: VecDeque<(Product, u64)> = VecDeque::new();
        let mut source_closed = false;

        loop {
            // 1) Vaciar canal a la cola interna
            match input_rx.try_recv() {
                Ok(prod) => queue.push_back((prod, total_time)),
                Err(TryRecvError::Empty) => { /* nada nuevo */ }
                Err(TryRecvError::Disconnected) => source_closed = true,
            }

            // 2) Tomar siguiente producto de la cola
            if let Some((mut product, mut remaining)) = queue.pop_front() {
                // Si es la primera vez, registramos la entrada
                if !product.station_logs.contains_key(&name) {
                    product.log_entry(&name);
                }

                // Procesamos un slice de máximo `quantum` o lo que reste
                let slice = std::cmp::min(quantum.as_secs(), remaining);
                thread::sleep(Duration::from_secs(slice));
                remaining -= slice;

                if remaining == 0 {
                    // Ya terminamos la estación
                    product.log_exit(&name);
                    if is_final_station {
                        product.kill();
                        //reenviar a metricas
                        output_tx
                            .send(product)
                            .expect("Falló envío de producto finalizado a métricas");
                    } else {
                        output_tx.send(product).expect("Falló envío RR → next");
                    }
                } else {
                    // Preemisión: reencolar con tiempo restante
                    queue.push_back((product, remaining));
                }
            }
            // 3) Si no hay trabajo y el canal se cerró, salimos
            else if source_closed {
                break;
            }
            // 4) Si no hay nada por hacer, esperamos un poco
            else {
                thread::sleep(Duration::from_millis(50));
            }
        }
    });
}

/// Utilitario para sacar el tiempo de configuración de cada estación
fn get_station_time(name: &str) -> u64 {
    match name {
        "Corte" => crate::config::CUTTING_TIME,
        "Ensamblaje" => crate::config::ASSEMBLY_TIME,
        "Empaque" => crate::config::PACKAGING_TIME,
        _ => panic!("Estación desconocida en scheduler: {}", name),
    }
}
