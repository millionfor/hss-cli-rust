# hss-cli-rust

## 项目简介

`hss-cli-rust` 是一个基于 Rust 的 Jenkins 命令行工具，支持登录 Jenkins、触发构建、监控构建日志以及中断构建等功能。

## 安装方法

1. 确保已安装 Rust 环境。
2. 克隆本项目到本地：
   ```bash
   git clone <本项目地址>
   cd hss-cli-rust
   ```
3. 编译运行：
   ```bash
   cargo build --release
   ```

## 配置说明

首次使用前需进行 Jenkins 相关信息配置，配置文件位于用户主目录下的 `.hssrc` 文件。

- 配置内容包括：
  - jenkins_url
  - user
  - token

可通过登录命令自动生成配置文件。

## 使用方法

### 1. 登录 Jenkins

```bash
hss-cli login <jenkins_url> <user> <token>
```
- `<jenkins_url>`: Jenkins 服务器地址（如 https://jenkins.example.com/）
- `<user>`: Jenkins 用户名
- `<token>`: Jenkins API Token

示例：
```bash
hss-cli login https://jenkins.example.com/ admin 1234567890abcdef
```

### 2. 触发构建并监控日志

```bash
hss-cli build <project> <branch> <env>
```
- `<project>`: Jenkins Job 名称
- `<branch>`: 代码分支名（如 master、dev 等）
- `<env>`: 环境变量（如 test、prod 等）

示例：
```bash
hss-cli build my-job master test
```

- 构建过程中会自动输出 Jenkins 控制台日志。
- 构建完成后会显示最终结果。
- 构建过程中按下 `Ctrl+C` 可中断 Jenkins 构建。

## 常见问题

- 如果提示“请先运行 'hss-cli login <jenkins_url> <user> <token>' 进行登录”，请先完成登录配置。
- 配置文件 `.hssrc` 位于用户主目录下，可手动编辑。

## 依赖

- [jenkins_sdk](https://crates.io/crates/jenkins_sdk)
- [tokio](https://crates.io/crates/tokio)
- [serde_json](https://crates.io/crates/serde_json)

## 许可证

MIT License [QuanQuan](https://github.com/millionfor) | [HydeeSoft](https://github.com/hydeesoft)
