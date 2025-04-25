mod config;
mod controller;
mod ipc_manager;
mod metrics;
mod product;
mod scheduler;
mod station;

use clap::{Arg, Command};
use config::{DEFAULT_QUANTUM, PRODUCTS_PER_BATCH};
use controller::generate_products;
use crossbeam_channel::unbounded;
use metrics::start_metrics_collector;
use scheduler::{SchedulingAlgorithm, start_fcfs_station, start_rr_station};
use std::time::Duration;

// TODO: Inicializar simulación
// [ ] Configurar canales IPC con ipc_manager::setup_ipc()
// [ ] Iniciar hilos para las 3 estaciones (station::start_station)
// [ ] Generar productos usando controller::generate_products()
// [ ] Implementar CLI con clap (--algorithm, --quantum)
// [ ] Ejecutar scheduler según algoritmo seleccionado
// [ ] Recolectar y mostrar métricas finales

// cargo run main

fn main() {
    // 1. Parse CLI
    let matches = Command::new("assembly_simulator")
        .version("0.1")
        .author("Christopher Estudiante de Ingeniería en Computación")
        .about("Simula una línea de ensamblaje con IPC y scheduling")
        .arg(
            Arg::new("algorithm")
                .short('a')
                .long("algorithm")
                .value_name("ALGORITHM")
                .help("Algoritmo de scheduling: 'fcfs' o 'rr'")
                .required(true)
                .num_args(1),
        )
        .arg(
            Arg::new("quantum")
                .short('q')
                .long("quantum")
                .value_name("QUANTUM")
                .help("Quantum en segundos para Round Robin")
                .num_args(1),
        )
        .get_matches();

    let algo_str = matches
        .get_one::<String>("algorithm")
        .expect("Se requiere --algorithm")
        .to_lowercase();

    let scheduling_algo = match algo_str.as_str() {
        "fcfs" => SchedulingAlgorithm::FCFS,
        "rr" => {
            let q_secs = matches
                .get_one::<String>("quantum")
                .map(|v| v.parse::<u64>().expect("Quantum debe ser un número"))
                .unwrap_or(DEFAULT_QUANTUM);
            SchedulingAlgorithm::RoundRobin {
                quantum: Duration::from_secs(q_secs),
            }
        }
        other => panic!("Algoritmo desconocido: {} (usar 'fcfs' o 'rr')", other),
    };

    println!("Iniciando simulación con algoritmo: {}", algo_str);
    if let SchedulingAlgorithm::RoundRobin { quantum } = scheduling_algo {
        println!("Quantum configurado: {}s", quantum.as_secs());
    }

    // 2. Configurar IPC
    let (global_tx, (global_rx, cut_tx), (assembly_rx, assembly_tx), (pack_rx, _)) =
        ipc_manager::setup_ipc();

    // 3. Canal de métricas
    let (metrics_tx, metrics_rx) = unbounded();
    let metrics_handle = start_metrics_collector(metrics_rx, PRODUCTS_PER_BATCH);

    // 4. Generar productos
    let producer_handle = generate_products(global_tx);

    // 5. Iniciar estaciones según algoritmo
    match scheduling_algo {
        SchedulingAlgorithm::FCFS => {
            start_fcfs_station("Corte".into(), global_rx, cut_tx, false);
            start_fcfs_station("Ensamblaje".into(), assembly_rx, assembly_tx, false);
            start_fcfs_station("Empaque".into(), pack_rx, metrics_tx, true);
        }
        SchedulingAlgorithm::RoundRobin { quantum } => {
            start_rr_station("Corte".into(), global_rx, cut_tx, quantum, false);
            start_rr_station(
                "Ensamblaje".into(),
                assembly_rx,
                assembly_tx,
                quantum,
                false,
            );
            start_rr_station("Empaque".into(), pack_rx, metrics_tx, quantum, true);
        }
    }

    // 6. Esperar fin de generación y métricas
    producer_handle.join().expect("Error en hilo generador");
    metrics_handle
        .join()
        .expect("Error en recolector de métricas");

    println!("Simulación completada.");
}
