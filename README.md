# Better-USTC-2

![Better-USTC-2](/src-tauri/icons/icon.png) <!-- 请将此处的 logo.png 替换为你们的 Logo 图片路径 -->

Better-USTC-2 是一个面向 Android 的移动应用，目标是优化中国科学技术大学“第二课堂”平台的使用体验（推荐机制 + 活动提醒与报名）。

---

## 主要特性

- 统一身份认证（前端输入账号密码，调用后端登录函数）
- 二课活动聚合（未报名 / 已报名 / 已参加 / 待申诉）
- 智能推荐：基于历史参与数据的个性化推荐
- 课表查询与日程提醒（可选）

---

## 技术栈

- 前端：Vue 3 + Vite + Vant
- 后端：Rust（Tauri）
- 平台：Tauri for Mobile（Android 构建，后续可能考虑 iOS）

---

## 快速开始（Android）

1. 克隆仓库并进入目录：

```bash
git clone git@github.com:Huasushis/better-ustc-2.git
cd better-ustc-2
```

2. 安装依赖：

```bash
pnpm install
```

3. 在 Android 设备或模拟器上运行（开发模式）：

```bash
pnpm tauri android dev
```

4. 构建产物（APK）：

```bash
pnpm tauri android build
```

> 注意：请参考 Tauri 官方移动端文档完成 Android SDK、NDK、以及 Rust 环境配置。

---

## 前后端交互与数据格式（预留）

前后端通过 Tauri 的 `invoke` 调用后端命令。项目主要交互数据为两类 JSON：活动列表（Activity List）与课表（Class Schedule）。开发文档中已预留这两部分的 JSON 格式位置，详见：`docs/development.md`。

---

## 工作流与 CI

- 我们采用 **GitFlow** 工作流：
	- `develop` 用于日常开发，`feature/*` 分支用于功能开发，`release/*` 用于准备发布，`main` 用于稳定的已发布版本。
- 项目已配置 **GitHub Actions** 用于自动化：版本管理（release-please）、构建 Android APK（当产生 release/tag 时）等自动流程。

如果你需要调整自动发布或 CI 行为，请查看 `.github/workflows/build.yml`。

---

## 团队

- 陈鑫
- 范祎博涵
- 周映诚

---

## 文档

更详细的[开发文档](/docs/development.md)（API、数据结构与开发规范）请查看：

```
docs/development.md
```

---

## 贡献与许可

欢迎贡献。请遵循仓库的提交规范（推荐使用 Conventional Commits，以便 release-please 能自动生成版本）。

本项目采用 MIT 许可证（见 `LICENSE`）。

