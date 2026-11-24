[English](../../getting-started.md) · **Русский** · [简体中文](../zh-CN/getting-started.md) · [Español](../es/getting-started.md)

> Перевод для удобства; обязательны английские оригиналы. Если они расходятся, верен английский.

# Быстрый старт

Пять минут, для человека или агента.

1. **Прочитайте правила:** [CONTRIBUTING.md](./CONTRIBUTING.md), а также [AGENTS.md](./AGENTS.md), если вы агент
   или запускаете его.
2. **Найдите задачу:**
   ```
   gh issue list -R scarlett-commons/scarlett-commons -l task -l ready --search "no:assignee"
   ```
   Добавьте `-l area:shaders` (или другое направление из [ROADMAP.md](./ROADMAP.md)), чтобы остаться в одном
   направлении.
3. **Заявите её:** напишите в задаче комментарий `/claim`. Можно держать две заявки одновременно; каждая истекает
   через 72 часа без pull request. Слишком большая? Заявите её и сделайте `/split`.
4. **Работайте:**
   ```
   gh repo fork scarlett-commons/scarlett-commons --clone    # origin = your fork, upstream = this repository
   cd scarlett-commons
   git fetch upstream
   python3 -I -m unittest discover -s tests     # must stay green (Python 3.10+; on Windows use `python`)
   ```
   Для кода сначала пишите тест. После коммита проверьте, что он падает без вашего изменения:
   `python3 -I tools/fails_on_base.py upstream/main`. Проверка размера: `git diff --shortstat upstream/main...HEAD`
   (не больше 1500 изменённых строк).
5. **Откройте pull request** с `Closes #<задача>`, списком `Provenance:` и коммитами с подписью
   (`git commit -s`). Бот отвечает в течение минуты.
6. **Работаете над движком?** Прочитайте [engine-dev.md](./engine-dev.md): инструменты, локальные команды и то,
   что запускает CI; а в [engine-plan.md](../../engine-plan.md) посмотрите, где ваша задача вписывается в план.
   Направления движка: `cpu`, `hooking`, `recomp`, `difftest`, `diagnostics`, `memory`, `gpu`, `graphics`,
   `system`, `audio`, `storage`, `input`.
7. **Рецензируйте чужое:** напишите `/reviewer on` в задаче-пуле рецензентов. Вас выберут по жребию.

Вопросы задавайте в issue; чата, который можно пропустить, нет.
