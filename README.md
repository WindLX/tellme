# Tellme 🗣️

> "Tell me what just happened!"

`tellme` 是一个命令行工具，旨在解决终端输出过长、回滚缓冲区不足导致关键信息丢失的问题。当你运行一个复杂的编译、测试或部署命令后，只需输入 `tellme`，它就会自动将上一条命令的**全部输出**保存到一个日志文件中，让 Debug 和复盘变得轻而易举。

---

## 🚀 特性

- **一键捕获**: 无需预先设置，命令运行后即可捕获。
- **完整保存**: 不再受终端回滚行数限制，捕获完整的 stdout 和 stderr。
- **自动清理**: 自动去除 ANSI 颜色代码，生成干净、可读的日志文件。
- **时间戳命名**: 默认以时间戳命名日志文件，方便归档和查找。
- **高性能**: 核心逻辑由 Rust 编写，处理大型日志文件速度极快。
- **轻量级集成**: 通过简单的 Zsh 钩子与你的 Shell 无缝集成。

## 🛠️ 安装

### 先决条件

- **Zsh**: `tellme` 目前通过 Zsh Hooks 实现，因此需要 Zsh 环境。
- **curl**: 用于下载安装脚本。

### 一键安装

只需在你的终端中运行以下命令：

```bash
curl -fsSL https://raw.githubusercontent.com/WindLX/tellme/main/install.sh | sh
```

安装脚本会自动完成以下工作：
1.  从 GitHub Releases 下载预编译的二进制文件到 `~/.local/bin`。
2.  如果下载失败，会尝试使用 `cargo` 从源码编译。
3.  下载 Zsh 钩子脚本到 `~/.config/tellme/tellme.zsh`。
4.  在你的 `~/.zshrc` 文件中添加一行 `source` 命令来加载钩子。

安装完成后，请**重启你的终端**或运行 `source ~/.zshrc` 来使配置生效。

## 💡 使用方法

`tellme` 的使用非常简单直观。

**场景一：捕获编译错误**

```zsh
# 1. 运行一个可能会产生大量输出的命令
$ make build-all
... (屏幕疯狂滚动，最后出现了一个你看不到的错误) ...

# 2. 立即运行 tellme
$ tellme
✔ Output saved to tellme_2023-10-28_15-30-00.log

# 3. 现在你可以用任何编辑器查看完整的日志
$ vim tellme_2023-10-28_15-30-00.log
```

**场景二：自定义输出文件名**

使用 `-o` 或 `--output` 参数指定文件名。

```zsh
$ pytest -v tests/
... (测试失败，输出很长) ...

$ tellme -o test_failure.log
✔ Output saved to test_failure.log
```

**场景三：保留原始颜色**

默认情况下，`tellme` 会移除 ANSI 颜色代码。如果你想保留它们，请使用 `-r` 或 `--raw` 参数。

```zsh
$ tellme --raw -o colored_output.txt
✔ Output saved to colored_output.txt
```

## 🗑️ 卸载

我们提供了一个干净的卸载脚本。

```bash
curl -fsSL https://raw.githubusercontent.com/WindLX/tellme/main/uninstall.sh | sh
```
该脚本会移除二进制文件、配置文件，并清理你的 `.zshrc`。

## 🤝 贡献

欢迎提交 Issues 和 Pull Requests！

## 📄 许可证

本项目使用 [MIT](LICENSE) 许可证。