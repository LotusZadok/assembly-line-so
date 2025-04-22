// TODO: Sistema de métricas
// [ ] Calcular avg_wait_time (promedio por producto)
// [ ] Calcular avg_turnaround_time (fin - llegada)
// [ ] Generar reporte en consola (tabla formateada)
// [ ] Serializar resultados a JSON (serde_json)
// [ ] Mostrar tiempos por estación individual

use crossbeam_channel::Receiver;
use crate::product::Product;
use crate::config::STATION_ORDER;
use std::thread;

/// Inicia un recolector de métricas que lee productos terminados.
/// `rx`: canal de productos completos.
/// `total_products`: número total esperado para saber cuándo parar.
pub fn start_metrics_collector(
    rx: Receiver<Product>,
    total_products: usize,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut results = Vec::with_capacity(total_products);

        // Recolectar hasta completarse
        for product in rx.iter() {
            results.push(product);
            if results.len() >= total_products {
                break;
            }
        }

        // Cálculo de métricas
        let count = results.len() as f64;
        let total_wait: u64 = results.iter().map(|p| p.metrics.total_wait_time).sum();
        let total_turnaround: u64 = results
            .iter()
            .filter_map(|p| p.metrics.turnaround_time)
            .sum();

        let avg_wait = total_wait as f64 / count;
        let avg_turnaround = total_turnaround as f64 / count;

        // Impresión de resultados
        println!("\n===== MÉTRICAS FINALES =====");
        println!("Productos procesados: {}", results.len());
        println!("Promedio tiempo de espera   : {:.2} s", avg_wait);
        println!("Promedio turnaround         : {:.2} s", avg_turnaround);

        // Tabla por producto
        println!("\nID   Espera(s)   Turnaround(s)");
        for p in &results {
            let turnaround = p.metrics.turnaround_time.unwrap_or_default();
            println!("{:<4} {:<11} {}", p.id, p.metrics.total_wait_time, turnaround);
        }

        // Tiempos por estación
        println!("\nDetalle por estación (duración de procesado):");
        for p in &results {
            println!("Producto {}:", p.id);
            for &station in STATION_ORDER.iter() {
                if let Some(&(entry, exit)) = p.station_logs.get(station) {
                    println!("  {:<12}: {} s", station, exit - entry);
                }
            }
        }

        // Serializar a JSON
        if let Ok(json) = serde_json::to_string_pretty(&results) {
            println!("\nJSON de resultados:\n{}", json);
        }
    })
}
