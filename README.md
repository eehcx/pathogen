# Pathogen Firewall

> *Purga el tráfico impuro. Redefine las reglas de supervivencia.*

Pathogen es una interfaz de terminal (TUI) de alto rendimiento, escrita en **Rust** y respaldada por la arquitectura moderna de **nftables** en el kernel de Linux. Está diseñada para proveer a los administradores de sistemas una herramienta capaz de asegurar infraestructuras de forma implacable, combinando la eficiencia técnica de la línea de comandos con la precisión visual de una aplicación interactiva.

Inspirado en una estética cruda, biomecánica y de alta precisión (siguiendo la filosofía visual de *Prometheus*), Pathogen no es un simple panel de control, es un guardián absoluto del flujo de red.

---

## Características Principales

1. **Gestión de Puertos (Bloqueo Quirúrgico):** Bloqueo y desbloqueo de puertos TCP/UDP de forma instantánea.
2. **Cuarentena de IPs (Blacklists de Alto Rendimiento):** Utiliza estructuras `Sets` de nftables para aislar direcciones IP maliciosas en memoria sin comprometer el rendimiento del CPU.
3. **Escudo Anti-DDoS y Control de Flujo:** Protege servicios críticos limitando dinámicamente la cantidad de conexiones por segundo/minuto por IP mediante el uso avanzado de `Meters`.
4. **Monitor de Purga (Logs del Kernel):** Visualización en tiempo real de los registros de bloqueos en el kernel (`journalctl`), permitiendo observar la actividad anómala directamente desde la interfaz.
5. **Arquitectura Limpia y Confinamiento de Privilegios:** Desarrollado bajo los principios de *Clean Architecture*. El binario principal minimiza el riesgo delegando operaciones críticas a scripts aislados.

---

## Requisitos del Sistema

- **OS:** Distribución basada en Linux.
- **Backend:** `nftables` instalado, habilitado y activo en el sistema.
- **Compilador:** Herramientas de `cargo` y `rustc` para la construcción desde el código fuente.

---

## Instalación y Ejecución

Pathogen incluye un instalador automatizado que compila el binario en modo Release y orquesta los permisos necesarios para interactuar con el firewall.

```bash
# Clonar el repositorio
git clone https://github.com/eehcx/pathogen.git
cd pathogen

# Ejecutar el instalador automatizado
./install.sh

# Iniciar la interfaz
sudo pathogen
```

### Arquitectura de Instalación (`install.sh`)
1. Compila el proyecto en Rust usando perfil de máxima optimización (`cargo build --release`).
2. Despliega los scripts de infraestructura en `/usr/local/share/pathogen/scripts`.
3. Enlaza el binario compilado en `/usr/local/bin/pathogen`.
4. (Opcional según entorno) Configura reglas en `/etc/sudoers.d/pathogen` para el aislamiento de comandos.

---

## Uso y Controles

La navegación de Pathogen está optimizada para la interacción rápida sin ratón:

* **Menú Principal:** `Flecha Arriba/Abajo` para navegar, `Enter` para acceder.
* **Pantallas de Listas:** `m` o `Esc` para volver al menú de origen.
* **Acciones de Bloqueo:** Presionar `b` (en reglas de puertos) o `q` (en cuarentena de IPs) para invocar cuadros de diálogo de inserción.
* **Eliminación/Desbloqueo:** Presionar `d` sobre la regla o IP actualmente seleccionada.
* **Navegación en Formularios:** Usar `Tab` (alternar protocolos) o `Espacio` (alternar unidades de tiempo).
* **Salir de la Consola:** Presionar `q` desde el menú principal.

---

## Control de Versiones e Integración

Este proyecto sigue una metodología de integración y entrega estructurada:
* **Gitflow:** Todo el desarrollo de nuevas características se realiza en ramas `feature/*` que se integran en la rama `develop`.
* **Hotfixes:** Los parches críticos se resuelven mediante ramas `hotfix/*` que se integran directamente a `master` y a `develop`.
* **Releases:** La rama `master` está reservada exclusivamente para versiones estables. (Actualmente en desarrollo pre-release).

---

## Arquitectura de Software

El código fuente está estrictamente parcelado para garantizar su mantenibilidad y prueba:
* **Domain:** Entidades de negocio puras (`Rule`, `PortRequest`, `RateLimitRequest`).
* **Use Cases:** Interfaces de aplicación y orquestación.
* **Infrastructure:** Implementación concreta de la persistencia interconectando con JSON y llamadas al sistema operativo.
* **Presentation:** Renderizado gráfico y manejo de eventos mediante `ratatui` y `crossterm`.

---

## Licencia

Distribuido bajo licencia GNU GPLv3. Software de infraestructura crítica: úsalo bajo tu propia responsabilidad y audita el código antes de desplegar en producción.