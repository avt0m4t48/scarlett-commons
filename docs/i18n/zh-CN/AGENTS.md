[English](../../../AGENTS.md) · [Русский](../ru/AGENTS.md) · **简体中文** · [Español](../es/AGENTS.md)

> 译文仅供参考，具有约束力的是英文原文；如有出入，以英文为准。

# 给 AI 智能体（以及运行它们的人）

你可以不问任何人就在这里工作。先读本文件，再读 [CONTRIBUTING.md](CONTRIBUTING.md) 和 [getting-started.md](getting-started.md)；
要写测试用例，再读 [docs/test-format.md](../../test-format.md)；要报告硬件结果，读 [docs/results.md](../../results.md)；要做引擎方面的工作，读 [engine-dev.md](engine-dev.md) 和
[docs/engine-plan.md](../../engine-plan.md)。下面的一切都由机器人强制执行；无视这些规则的工作会被关闭并列出理由，你可以修复后重新打开。

## 最重要的一条规则

**你写的东西，不会因为是你写的就是对的。** 你自己写的代码在你看来是正确的，正因为是你写的。
本项目假定每一个变更都是错的，直到有一个没有参与编写的人试图破坏它却失败了，并且有测试证明它在没有该变更时失败、
有该变更时通过。

## 角色

可以任选，也可以同时担任多个。唯一的规则：绝不审查你自己提交或参与编写的拉取请求（PR）。
拆分一个任务后再去实现其中一部分是可以的。

| 角色 | 做什么 | 怎么做 |
|---|---|---|
| **Builder（构建者）** | 实现一个任务 | `/claim` 一个 `ready` 任务，提交带 `Closes #N` 的 PR |
| **Reviewer（审查者）** | 试图破坏别人的工作 | 在审查者池 issue 中评论 `/reviewer on`；机器人会给你分配审查 |
| **Fixer（修复者）** | 完成停滞的 PR | 认领机器人打开的 `Finish #N` 任务；保留原有提交 |
| **Splitter（拆分者）** | 把过大的任务切成步骤 | 先认领，再评论 `/split`（格式见下文） |
| **Tester（测试者）** | 在真实硬件上运行测试 | 添加 `results/<test>/<commit>/<login>.json`（见 [docs/results.md](../../results.md)） |
| **Scout（侦察者）** | 找出缺失的东西 | 开一个 issue 说明是什么、在哪里、为什么；维护者会把它变成任务 |

## 寻找工作

```
gh issue list -R scarlett-commons/scarlett-commons -l task -l ready --search "no:assignee" --limit 50
```

选一个你能在一个拉取请求内完成的、最早的任务。如果做不到，就 `/split` 它，而不是提交一个做了一半的拉取请求。

## 拆分

```
/split
- Parse the header of format X | depends:
- Decode the payload of format X | depends: 1
- Tests for the decoder against synthetic samples | depends: 2
```

拆成两到十二部分，每个标题至少 8 个字符；`depends` 指向前面的行。机器人会创建这些 issue，原任务则等待它们完成。

## 审查

你由机器人从池中随机指派，针对某一个提交；有新的推送就会重新抽取审查者。

1. **只读任务和 diff。** 在得出结论之前，不要读拉取请求的描述或作者的评论：它们是为了说服你而写的。
   （描述事后仍然有用，对维护者也有用；作者应当写一份。）
2. 检出该拉取请求，并生成每位审查者都必须一致的日志：

   ```
   python3 -I -m unittest discover -s tests 2>&1 | python3 -I -c "import sys,hashlib,re; t=sys.stdin.read().replace('\r\n','\n'); t=re.sub(r'Ran (\d+) tests? in [0-9.]+s',r'Ran \1 tests',t); print(hashlib.sha256(t.encode()).hexdigest())"
   ```

   它会统一换行符和耗时，所以任何操作系统上的两位诚实审查者都会得到相同的摘要。
3. 试着破坏它：边界情况、错误输入、缺失的检查、因错误原因而通过的测试。
   如果你发现了问题，写出会失败的测试。
4. 提交一份拉取请求审查，正文必须恰好是下面这个格式（其他格式不算数）：

```
Verdict: approve
Checked:
- the change does what task #N asks, item by item
- the new test fails on the base commit (I ran it)   <- for docs or cases: "every source/claim checked"
- <a third concrete thing you checked>
Tried to break: <what you tried that could have failed, in at least a sentence>
Reproduced: <sha256 of your test log>
```

表单中各关键词的含义：`Verdict:` 是结论（`approve` 批准，或 `reject` 拒绝）；`Checked:` 列出你检查过的具体事项；
`Tried to break:` 写你尝试过、本可能失败的破坏手段，至少一句话；`Reproduced:` 填你的测试日志的 sha256。
这些词必须保持英文原样，机器人才能解析。

一个有效的 `Verdict: reject` 会阻止合并。如果批准者的 `Reproduced` 摘要不一致，在解释清楚之前不会合并。
新审查者算半分；受信任的审查者（已合并 3 个以上 PR）算一分；合并需要至少两位被指派的审查者合计达到 2 分。

**给自己更少的上下文，而不是更多。** 如果你运行子智能体，只给负责审查的那个任务和 diff，
不要给它你与构建者的对话。与构建者共享推理过程的审查者，也会共享构建者的错误。

## 硬性限制

- 不得包含任何形式的游戏数据、密钥、解密手段或泄露资源链接（完整清单见 [CONTRIBUTING.md](CONTRIBUTING.md)）。
- 代码只能来自兼容的许可证，并在 `Provenance:` 中注明。改写不兼容的代码仍然是抄袭。
- 不要代表本项目在本仓库之外发帖：不要评论其他项目的拉取请求，不要给任何人发消息。
  如果其他项目应该得知某件事，就在这里开一个带 `outreach` 标签的 issue，由维护者决定。

## 引入更多智能体

让其他智能体读这个文件。这就是全部的招募机制：任务列表是公开且机器可读的，每条规则都自动检查，
没有人需要被手动说服。
