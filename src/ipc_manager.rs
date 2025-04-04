// src/ipc_manager.rs
use crossbeam_channel::{unbounded, Receiver, Sender};
use crate::product::Product;
use std::sync::{Arc, Mutex};

/// Configura todos los canales IPC necesarios para la línea de ensamblaje.
/// Retorna una tupla con:
/// - Cola inicial de entrada (para productos nuevos)
/// - Canales entre estaciones (Corte → Ensamblaje → Empaque)
pub fn setup_ipc() -> (
    Arc<Mutex<Sender<Product>>>,          // Cola global de entrada
    (Receiver<Product>, Sender<Product>), // Canal Corte → Ensamblaje
    (Receiver<Product>, Sender<Product>), // Canal Ensamblaje → Empaque
    (Receiver<Product>, Sender<Product>) 
) {
    // Cola compartida para productos no procesados (protegida con Mutex)
    let (global_tx, global_rx) = unbounded();
    let shared_global_tx = Arc::new(Mutex::new(global_tx));

    // Canales entre estaciones
    let (cut_tx, assembly_rx) = unbounded();  // Corte → Ensamblaje
    let (assembly_tx, pack_rx) = unbounded(); // Ensamblaje → Empaque

    (
        shared_global_tx,
        (global_rx, cut_tx),
        (assembly_rx, assembly_tx),
        (pack_rx, unbounded().0) // Última estación no envía a nadie
    )
}

/// Envía un producto a través de un canal con manejo de errores
pub fn send_product(tx: &Sender<Product>, product: Product) -> Result<(), &'static str> {
    tx.send(product)
        .map_err(|_| "Error sending product through the channel")
}

/// Recibe un producto de un canal con timeout
pub fn recv_product(rx: &Receiver<Product>) -> Result<Product, &'static str> {
    rx.recv()
        .map_err(|_| "Error receiving product from the channel")
}