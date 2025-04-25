// src/scheduler.rs

// TODO: Implementar algoritmos de scheduling
// [ ] FCFS: Procesar productos en orden de llegada
// [ ] Round Robin: Implementar quantum con thread::sleep
// [ ] Manejar reencolado para RR cuando el quantum expira
// [ ] Integración con canales IPC (usar ipc_manager helpers)
// [ ] Garantizar sincronización entre estaciones

use crate::config::{ENABLE_VERBOSE_LOGGING};
use crate::product::Product;
use crate::station;
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use std::{
    collections::VecDeque,
    thread,
    time::{Duration},
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

        if ENABLE_VERBOSE_LOGGING {
            println!(
                "[{}] Iniciada (RR) - Tiempo total procesamiento: {}s - Quantum: {}s",
                name,
                total_time,
                quantum.as_secs()
            );
        }

        loop {
            // 1) Vaciar canal a la cola interna
            match input_rx.try_recv() {
                Ok(prod) => {
                    if ENABLE_VERBOSE_LOGGING {
                        println!("[{}] Recibido producto {}", name, prod.id);
                    }
                    queue.push_back((prod, total_time))
                }
                Err(TryRecvError::Empty) => { /* nada nuevo */ }
                Err(TryRecvError::Disconnected) => source_closed = true,
            }

            // 2) Tomar siguiente producto de la cola
            if let Some((mut product, mut remaining)) = queue.pop_front() {
                if !product.station_logs.contains_key(&name) {
                    product.log_entry(&name);
                }

                let slice = std::cmp::min(quantum.as_secs(), remaining);
                if ENABLE_VERBOSE_LOGGING {
                    println!(
                        "[{}] Procesando producto {} por {}s (restante: {}s)",
                        name, product.id, slice, remaining
                    );
                }

                thread::sleep(Duration::from_secs(slice));
                remaining -= slice;

                if remaining == 0 {
                    product.log_exit(&name);
                    if ENABLE_VERBOSE_LOGGING {
                        println!("[{}] Producto {} completado", name, product.id);
                    }
                    if is_final_station {
                        product.kill();
                        output_tx
                            .send(product)
                            .expect("Falló envío de producto finalizado a métricas");
                    } else {
                        output_tx.send(product).expect("Falló envío RR → next");
                    }
                } else {
                    if ENABLE_VERBOSE_LOGGING {
                        println!(
                            "[{}] Preemisión producto {} - Reencolado con {}s restantes",
                            name, product.id, remaining
                        );
                    }
                    queue.push_back((product, remaining));
                }
            } else if source_closed {
                break;
            } else {
                thread::sleep(Duration::from_millis(50));
            }
        }

        if ENABLE_VERBOSE_LOGGING {
            println!("[{}] Terminada ejecución Round Robin", name);
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
