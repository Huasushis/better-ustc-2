# Better-USTC-2 开发文档

![Better-USTC-2](/logo.png) <!-- 请将此处的 logo.png 替换为你们的 Logo 图片路径 -->

**Better-USTC-2** 是一个旨在优化中国科学技术大学“第二课堂”平台的移动端应用项目。我们致力于通过现代化的技术栈和智能推荐算法，为同学们提供更便捷、更个性化的二课活动体验。

本项目为课程设计的最终项目。

---

## 团队成员

*   **陈鑫**
*   **范祎博涵**
*   **周映诚**

---

## ✨ 功能规划 (Features)

-   [ ] **统一身份认证**：安全加密并存储用户账号信息，实现便捷登录。
-   [ ] **二课活动浏览**：
    -   [ ] 获取所有可报名的二课活动。
    -   [ ] 查看已报名但未参加的活动。
    -   [ ] 查看已参加并获得学时的活动。
    -   [ ] 查看待申诉的学时项目。
-   [ ] **活动一键报名**：在应用内直接报名参加感兴趣的二-   [ ] **个性化推荐**：基于用户历史活动数据，智能推荐可能感兴趣的二课项目。
-   [ ] **课表查询**：集成课表查询功能，方便学生规划时间。
-   [ ] **活动提醒**：(可选) 对已报名的活动进行日程提醒。

---

## 🏛️ 技术架构 (Architecture)

*   **前端 (Frontend)**: `Vue.js` + `Vite` + `Vant`
*   **后端 (Backend)**: `Rust`
*   **框架 (Framework)**: `Tauri v2` - 使用 Web 技术构建跨平台移动端应用。

---

## 🖼️ 应用截图

<!-- 此处预留截图位置，用于展示应用界面 -->
| 登录页 | 主界面 |
| :---: | :---: |
| *待补充* | *待补充* |

---

## 🛠️ 开发环境搭建 (Development Setup)

1.  **克隆仓库**:
    ```bash
    git clone <your-repo-url>
    cd better-ustc-2
    ```

2.  **安装前端依赖**:
    ```bash
    pnpm install
    ```

3.  **配置 Rust & Tauri v2 环境**:
  请参考 [Tauri v2 官方文档](https://v2.tauri.org.cn/start/)（含 Android 章节）完成 Rust、Node.js、Android SDK/NDK 等依赖的配置。

4.  **在 Android 设备或模拟器上运行**:
    ```bash
    pnpm tauri android dev
    ```

5.  **构建 APK**:
    ```bash
    pnpm tauri android build
    ```

---

## ⚙️ API 设计 & 数据结构

应用前后端通过 Tauri 的 `invoke` 机制进行通信。后端 Rust 函数暴露给前端调用。

### 后端核心函数 (Rust -> Tauri Commands)

| 函数名 | 参数 | 返回值 | 描述 |
| :--- | :--- | :--- | :--- |
| `login` | `username: String`, `password: String` | `Result<bool, String>` | 接收用户账号密码进行登录，加密存储。返回成功或失败信息。 |
| `get_login_status` | - | `Result<UserInfo, String>` | 检查当前登录状态，若已登录则返回用户信息。 |
| `get_class_schedule` | - | `Result<Json, String>` | 爬取并返回当前学期的课表数据。 |
| `get_unregistered_activities` | - | `Result<Json, String>` | 获取所有可报名的二课活动列表。 |
| `get_registered_activities` | - | `Result<Json, String>` | 获取已报名但未参加的活动列表。 |
| `get_participated_activities` | - | `Result<Json, String>` | 获取已参加的活动列表。 |
| `get_pending_appeals` | - | `Result<Json, String>` | 获取待学时申诉的项目列表。 |
| `register_for_activity` | `activity_id: String` | `Result<bool, String>` | 报名指定的二课活动。 |
| `get_recommended_activities` | - | `Result<Json, String>` | 根据用户画像返回个性化推荐的活动列表。 |

### 数据格式 (JSON)

#### 1. 活动列表 (Activity List)

```json
// 此处预留活动列表的 JSON 结构
// 建议直接采用从学校网站爬取的数据格式，以简化处理流程
[
  {
    "activity_id": "unique_activity_id",
    "name": "活动名称",
    "time": "活动时间",
    "location": "活动地点",
    "organizer": "主办方",
    "credit": 1.0,
    "status": "unregistered | registered | participated"
  }
]
```

#### 2. 课表数据 (Class Schedule)

```json
// 此处预留课表数据的 JSON 结构
// 同样建议采用爬取到的原始数据格式
{
  "monday": [
    {
      "course_name": "课程名称",
      "time_slot": "1-2",
      "location": "教学楼",
      "teacher": "教师姓名"
    }
  ],
  "tuesday": [],
  "...": "..."
}
```

---

## 🚀 未来展望 (Roadmap)

*   [ ] **更完善的推荐算法**：引入协同过滤或内容推荐模型。
*   [ ] **支持 iOS**：扩展项目以支持在 iOS 平台上构建和运行。
*   [ ] **UI/UX 优化**：持续改进 Vant 组件库的用户体验。
*   [ ] **原生功能集成**：探索使用 Tauri 插件实现如系统通知、生物识别等原生移动功能。

---

## 📄 许可证 (License)

本项目采用 [MIT License](./LICENSE) 开源。
