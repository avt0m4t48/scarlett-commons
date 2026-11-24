[English](../../../ROADMAP.md) · [Русский](../ru/ROADMAP.md) · [简体中文](../zh-CN/ROADMAP.md) · **Español**

> Traducción de cortesía; son vinculantes los originales en inglés. Si difieren, vale el inglés.

# Hoja de ruta

Este es el plan desde el que todos trabajan. Cada punto se convierte en issues de tareas etiquetados con su área;
el bot las reparte en orden de dependencias. El avance se sigue automáticamente en [la página de progreso](https://github.com/scarlett-commons/scarlett-commons/blob/status/PROGRESS.md).

**Meta:** hacer que las partes difíciles y compartidas de ejecutar software de Xbox Series X|S en PC queden
correctas y verificadas una sola vez, para todos los proyectos de compatibilidad, en lugar de quince veces.
Construimos conocimiento, pruebas y componentes comprobados; los demás proyectos los usan.

**Nunca:** datos de juegos, claves, descifrado, enlaces a filtraciones, código de licencias incompatibles (ver
[CONTRIBUTING.md](CONTRIBUTING.md)).

## Prioridad

Primero una compilación estable y jugable, lo más rápido posible: las tareas con la etiqueta `priority:playable` ([docs/engine-plan.md](../../engine-plan.md), "Priority"). Todo lo demás la apoya o viene después.

## La línea del motor (junto a cada fase)

El recompilador en `engine/`, importado de la continuación de Leonida con su historial y mantenido al día con ella
mediante una sincronización diaria ([docs/engine-sync.md](../../engine-sync.md)). El trabajo en él sigue las mismas reglas que todo
lo demás, y su avance en títulos reales se informa por hash de compilación ([title-runs.md](title-runs.md)): carga, importaciones
resueltas, dispositivo creado, primer fotograma, menú, en el mundo, jugable. Las fases de abajo lo hacen correcto;
la línea del motor lo hace funcionar. Sus tareas, con las herramientas ya listas para cada una: [docs/engine-plan.md](../../engine-plan.md).

## Fase 0: Cimientos (terminada)

La maquinaria que permite que personas y agentes se unan sin que nadie los incorpore.

- Bot: puerta de reglas, reclamos, plazos, tareas de arreglo, `/split`, revisores ciegos, automerge, modo de prueba en seco.
- Documentación: esta hoja de ruta, arquitectura, primeros pasos, AGENTS.md, traducciones.
- Página de progreso generada a partir del tablero de tareas.
- Espejo en un segundo host; procedimiento de restauración.
- Propuestas automáticas de mantenedores.

**Terminada cuando:** un agente nuevo puede leer AGENTS.md, tomar una tarea y lograr que se acepte un pull request
sin que intervenga ninguna persona salvo sus revisores.

## Fase 1: Conocimiento compartido

Una biblioteca de pruebas que define el comportamiento correcto, comprobada en hardware real.

| Área | Qué | Etiqueta |
|---|---|---|
| Shaders | Casos RDNA2 neutros respecto al formato: VALU f16/f32/f64 (redondeo, denormales, FMA), SALU, LDS, wave64 y entre carriles, imagen y sampler, direccionamiento de buffers, interpolación y exportaciones | `area:shaders` |
| Oráculo de hardware | Salidas esperadas confirmadas en AMD RDNA2 real y en las GPU de PC comunes, por consenso ([docs/results.md](../../results.md)) | `area:results` |
| API de sistema | Especificaciones de comportamiento, a partir de documentación pública y pruebas con homebrew o ejemplos de Microsoft: memoria, hilos, archivos, tiempo | `area:system` |
| API gráfica | Extensiones de D3D12 de la consola frente a D3D12/Vulkan de escritorio: qué difiere y cómo lo mapea cada proyecto | `area:graphics` |
| Audio, almacenamiento, entrada | Especificaciones de comportamiento y pruebas de XAudio2, DirectStorage, GameInput | `area:audio`, `area:storage`, `area:input` |

**Terminada cuando:** cada caso tiene un resultado esperado confirmado por dos informes independientes.

## Fase 2: Adaptadores y la tabla de compatibilidad

- Un ejecutor por traductor: SPIR-V sobre Vulkan, DXIL sobre D3D12, HLSL. El traductor de cada proyecto se conecta mediante un comando.
- Una tabla pública: qué proyecto pasa qué casos, regenerada en cada ejecución.
- Ejecuciones en Linux y macOS (Vulkan, MoltenVK). Etiqueta `area:platforms`.

**Terminada cuando:** al menos tres traductores externos corren en la tabla.

## Fase 3: Arreglos upstream

Cada fallo que la tabla muestra en otro proyecto se convierte en un arreglo pequeño y probado que se ofrece a ese
proyecto. El contacto pasa primero por un issue `outreach`; un mantenedor decide qué se envía, una sola vez, en el
propio tracker de ese proyecto.

## Fase 4: Componentes compartidos

Donde varios proyectos necesitan la misma pieza y ninguno tiene una buena con licencia compatible, se construye aquí
como biblioteca con sus propias pruebas (por ejemplo un intérprete RDNA2 de referencia usado como oráculo cuando no
hay hardware disponible).

## Direcciones en paralelo

Todas las fases posteriores a la 0 corren a la vez; las tareas solo esperan sus propias dependencias. Etiquetas:
`area:shaders`, `area:results`, `area:system`, `area:graphics`, `area:audio`, `area:storage`, `area:input`,
`area:platforms`, `area:infra`, `area:docs`, `area:i18n`; la línea del motor añade `area:cpu`, `area:hooking`,
`area:recomp`, `area:difftest`, `area:diagnostics`, `area:memory`, `area:gpu` ([docs/engine-plan.md](../../engine-plan.md)).
