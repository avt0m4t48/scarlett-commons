[English](../../title-runs.md) · [Русский](../ru/title-runs.md) · **简体中文** · [Español](../es/title-runs.md)

> 译文仅供参考，具有约束力的是英文原文；如有出入，以英文为准。

# 游戏运行（Title runs）

引擎在真实游戏上能走多远，在不点名、不分享其中任何内容的前提下报告。

**目前状态：** `builds/` 中还没有登记任何构建，所以报告会作为 `unregistered`（未登记）提交，直到维护者根据两个一致的哈希登记第一个构建。`tools/local_run.py --profile` 所需的构建配置（针对引擎开发时所用的游戏）是 Leonida 后续项目（github.com/ubiquitarianism/leonida）：克隆它并传入它的文件夹。

1. **构建就是哈希。** `builds/<id>.json` 通过可执行文件的 SHA-256 和大小，以中性的 id 登记一个构建。
   在两个人用各自的副本确认了哈希之后，由维护者添加构建。
2. **检查你的副本：** `python3 -I tools/check_build.py <你的安装目录> <列表>`，列表是一行
   `<exe_sha256>  <exe name>`。只对匹配的构建提交报告。
3. **用一条命令运行它**，在你自己的机器上：

   ```
   python3 -I tools/local_run.py --profile <checkout with the title's build profile> --game <your copy> --work <folder>
   ```

   它用本仓库的引擎构建你的副本，在开启跟踪和崩溃报告的情况下运行，并按白名单写出
   `<folder>/report/<login>.json`：到达的阶段（到 first-frame 为止是自动检测的；menu、in-world 或 playable 需传入
   `--stage-reached`）、fps、停止位置（module+offset），以及按失败原因统计的着色器数量。发送之前请先读一遍。
4. **报告：** 在拉取请求中添加 `title-runs/<build id>/<engine commit>/<your login>.json`（格式见
   `tools/title_runs.py`）：到达的最远**阶段**（stage）、fps、**停止位置**（`module+0xOFFSET reason`），以及简短的
   备注。机器人会拒绝不以作者命名的报告。
5. **确认：** 当两个不同的人报告达到某个阶段或更远时，该阶段对（构建，提交）才算数。
   进度页显示每个构建已确认的最远阶段。

阶段按顺序依次为：`loads`、`imports-resolved`、`device-created`、`first-frame`、`menu`、`in-world`、`playable`。

**报告中绝不能有：** 截图、视频、音频、引用游戏代码或数据的日志、转储、着色器清单、文件内容，或者哪里能获得游戏。
一个偏移量和一行理由，就足以在你自己的副本中找到问题。

把你自己的副本变成引擎可以运行的构建的**配置（profile）**（文件列表、构建阶段）在本仓库之外维护；
见 engine/README.md。
