mod config;
mod product;
mod station;
mod ipc_manager;

/// cargo run main

fn main() {
    println!("Hello, world!");
}

// TODO: Inicializar simulación
// [ ] Configurar canales IPC con ipc_manager::setup_ipc()
// [ ] Iniciar hilos para las 3 estaciones (station::start_station)
// [ ] Generar productos usando controller::generate_products()
// [ ] Implementar CLI con clap (--algorithm, --quantum)
// [ ] Ejecutar scheduler según algoritmo seleccionado
// [ ] Recolectar y mostrar métricas finales