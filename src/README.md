# **Simulador de Línea de Ensamblaje**

Este proyecto implementa una simulación de una línea de ensamblaje utilizando múltiples estaciones de procesamiento. La simulación maneja productos que pasan por varias estaciones (Corte, Ensamblaje y Empaque), y se pueden procesar utilizando dos algoritmos de planificación: **First-Come-First-Served (FCFS)** y **Round Robin (RR)**. Además, se recogen métricas de rendimiento, como el tiempo de espera y el tiempo de turnaround para cada producto.

## **Características**

- **Generación de productos aleatorios** con tiempos de llegada simulados.
- **Dos algoritmos de planificación**:
  - **FCFS (First-Come-First-Served)**: Procesa productos en el orden en que llegan.
  - **Round Robin**: Asigna un tiempo de procesamiento fijo (quantum) a cada producto antes de pasar al siguiente.
- **Manejo de comunicación entre estaciones** utilizando canales IPC (Inter-Process Communication).
- **Cálculo de métricas**:
  - Tiempo de espera total.
  - Tiempo de turnaround (tiempo de procesamiento total desde la llegada hasta la salida).
  - Reportes en consola y en formato JSON con estadísticas detalladas.

## **Requisitos**

- **Rust**: La simulación está escrita en **Rust** y se debe compilar con la versión estable de Rust.

## **Instrucciones de Uso**

### **Configuración del Proyecto**

1. Clona el repositorio:
   ```bash
   git clone https://github.com/tuusuario/assembly-line-so.git
   cd assembly-line-so
2. Compila el proyecto:
   ```bash
   cargo update
   cargo build
3. Ejecuta la simulación:

   ```bash
   cargo run -- --algorithm fcfs
O para Round Robin (con un quantum de 2 segundos):

   ```bash
  cargo run -- --algorithm rr --quantum 2
