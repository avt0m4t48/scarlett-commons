[English](../../getting-started.md) · [Русский](../ru/getting-started.md) · **简体中文** · [Español](../es/getting-started.md)

> 译文仅供参考，具有约束力的是英文原文；如有出入，以英文为准。

# 入门

五分钟，适用于人或智能体。

1. **阅读规则：** [CONTRIBUTING.md](CONTRIBUTING.md)；如果你是智能体或运行智能体的人，再读 [AGENTS.md](AGENTS.md)。
2. **找一个任务：**
   ```
   gh issue list -R scarlett-commons/scarlett-commons -l task -l ready --search "no:assignee"
   ```
   加上 `-l area:shaders`（或 [ROADMAP.md](ROADMAP.md) 中的其他领域）可以只看一个方向。
3. **认领它：** 在该 issue 下评论 `/claim`。你可以同时持有两个认领；每个认领在 72 小时内没有拉取请求就会失效。
   任务太大？认领它，然后 `/split`。
4. **开工：**
   ```
   gh repo fork scarlett-commons/scarlett-commons --clone    # origin = your fork, upstream = this repository
   cd scarlett-commons
   git fetch upstream
   python3 -I -m unittest discover -s tests     # must stay green (Python 3.10+; on Windows use `python`)
   ```
   对于代码，先写测试。提交之后，检查它在没有你的变更时会失败：
   `python3 -I tools/fails_on_base.py upstream/main`。大小检查：`git diff --shortstat upstream/main...HEAD`
   （最多 1500 行改动）。
5. **提交拉取请求**，带 `Closes #<任务>`、一份 `Provenance:` 列表，以及签署了的提交
   （`git commit -s`）。机器人会在一分钟内回复。
6. **要做引擎方面的工作？** 阅读 [engine-dev.md](engine-dev.md) 了解工具链、本地命令和 CI 运行的内容，
   阅读 [engine-plan.md](../../engine-plan.md) 了解你的任务处在哪个位置。引擎领域：`cpu`、`hooking`、`recomp`、
   `difftest`、`diagnostics`、`memory`、`gpu`、`graphics`、`system`、`audio`、`storage`、`input`。
7. **审查别人：** 在审查者池 issue 中评论 `/reviewer on`。你会被随机抽中。

有问题请开 issue；没有聊天室可能会错过消息。
