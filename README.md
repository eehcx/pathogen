# Pathogen Firewall ☣️

> *Purga el tráfico impuro. Redefine las reglas de supervivencia.*

Pathogen es una interfaz de terminal (TUI) extremadamente rápida, escrita en **Rust** y potenciada por la arquitectura moderna de **nftables** en el kernel de Linux. Está diseñada para administradores de sistemas que buscan asegurar sus servidores con la eficiencia de la línea de comandos pero con la elegancia visual de una aplicación interactiva.

Inspirado en la estética biomecánica y oscura de H.R. Giger y el universo de *Prometheus / Alien: Covenant*, Pathogen no solo es una herramienta de administración; es un guardián implacable.

---

## 🔥 Características Principales

1. **Gestión de Puertos (Bloqueo Quirúrgico):** Bloquea y desbloquea puertos TCP/UDP al instante.
2. **Cuarentena de IPs (Blacklists de Alto Rendimiento):** Usa `Sets` de nftables para aislar IPs maliciosas sin penalizar el uso del CPU.
3. **Escudo Anti-DDoS y Fuerza Bruta (Rate Limiting):** Protege servicios críticos (como SSH o Bases de Datos) limitando dinámicamente la cantidad de conexiones por segundo/minuto por IP usando `Meters`.
4. **Registros de Purga (Monitor de Logs):** Lee los registros de bloqueos del kernel (`journalctl`) directamente desde la interfaz para observar a tus atacantes en tiempo real.
5. **Arquitectura Limpia y Segura:** Escrito siguiendo *Clean Architecture*. El binario en Rust no corre como root; delega las acciones específicas a scripts aislados mediante directivas de `sudoers`, manteniendo el principio de mínimos privilegios sin interrumpir la UI.

---

## 🛠️ Requisitos del Sistema

- **OS:** Sistema basado en Linux.
- **Backend:** `nftables` instalado y habilitado en el sistema.
- **Compilador:** `cargo` (Rust) para construir desde el código fuente.

---

## 🚀 Instalación y Ejecución

Pathogen incluye un instalador automatizado que compila el binario en modo Release y configura los permisos necesarios para que la TUI fluya sin pedir contraseñas interactivas de `sudo`.

```bash
# Clonar el repositorio
git clone https://github.com/tu-usuario/pathogen-firewall.git
cd pathogen-firewall

# Ejecutar el instalador automático
./install.sh

# Iniciar el guardián
pathogen
```

### ¿Qué hace `install.sh`?
1. Compila el proyecto Rust con `cargo build --release`.
2. Mueve los scripts de infraestructura a `/usr/local/share/pathogen/scripts`.
3. Mueve el binario a `/usr/local/bin/pathogen`.
4. Añade una regla a `/etc/sudoers.d/pathogen` permitiendo a tu usuario ejecutar **exclusivamente** los 5 scripts del firewall, protegiendo así el resto de tu sistema operativo.

---

## ⌨️ Controles de la TUI

* **Menú Principal:** `Flecha Arriba/Abajo` para navegar, `Enter` para acceder.
* **Pantallas de Listas:** `m` o `Esc` para volver al menú.
* **Bloquear (Puertos / IPs):** Presiona `b` (en reglas) o `q` (en cuarentena) para abrir el diálogo.
* **Desbloquear:** Presiona `d` sobre la regla o IP seleccionada.
* **Cambiar Formularios:** Usa `Tab` (para protocolos) o `Espacio` (para unidades de tiempo).
* **Salir de la herramienta:** Presiona `q` en el menú principal.

---

## 🏛️ Arquitectura del Software (Clean Architecture)
El código en Rust está estrictamente separado:
* **Domain:** Entidades puras (`Rule`, `PortRequest`, `RateLimitRequest`, etc.).
* **Use Cases:** Interfaces de aplicación.
* **Infrastructure:** Implementación de persistencia interactuando con JSON y Shell Scripts.
* **Presentation:** Vistas de la terminal usando `ratatui` y `crossterm`.

---

## ⚖️ Licencia
Distribuido bajo licencia MIT. Úsalo bajo tu propia responsabilidad.
