[English](../../getting-started.md) · [Русский](../ru/getting-started.md) · [简体中文](../zh-CN/getting-started.md) · **Español**

> Traducción de cortesía; son vinculantes los originales en inglés. Si difieren, vale el inglés.

# Primeros pasos

Cinco minutos, para una persona o un agente.

1. **Lee las reglas:** [CONTRIBUTING.md](CONTRIBUTING.md), y [AGENTS.md](AGENTS.md) si eres un agente o
   operas uno.
2. **Encuentra una tarea:**
   ```
   gh issue list -R scarlett-commons/scarlett-commons -l task -l ready --search "no:assignee"
   ```
   Añade `-l area:shaders` (u otra área de [ROADMAP.md](ROADMAP.md)) para quedarte en una sola dirección.
3. **Reclámala:** comenta `/claim` en el issue. Puedes tener dos reclamos a la vez; cada uno caduca a las 72 horas
   sin un pull request. ¿Demasiado grande? Reclámala y hazle `/split`.
4. **Trabaja:**
   ```
   gh repo fork scarlett-commons/scarlett-commons --clone    # origin = your fork, upstream = this repository
   cd scarlett-commons
   git fetch upstream
   python3 -I -m unittest discover -s tests     # must stay green (Python 3.10+; on Windows use `python`)
   ```
   Si es código, escribe la prueba primero. Tras hacer commit, comprueba que falla sin tu cambio:
   `python3 -I tools/fails_on_base.py upstream/main`. Comprobación de tamaño: `git diff --shortstat upstream/main...HEAD`
   (como máximo 1500 líneas cambiadas).
5. **Abre el pull request** con `Closes #<tarea>`, una lista `Provenance:` y commits firmados
   (`git commit -s`). El bot responde en menos de un minuto.
6. **¿Trabajas en el motor?** Lee [engine-dev.md](engine-dev.md) para la cadena de herramientas, los comandos locales y lo que
   ejecuta el CI, y [engine-plan.md](../../engine-plan.md) para ver dónde encaja tu tarea. Áreas del motor: `cpu`, `hooking`, `recomp`,
   `difftest`, `diagnostics`, `memory`, `gpu`, `graphics`, `system`, `audio`, `storage`, `input`.
7. **Revisa a otros:** comenta `/reviewer on` en el issue del grupo de revisores. Te sortearán al azar.

Las preguntas van en un issue; no hay un chat que se pueda perder.
