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

-   [x] **统一身份认证**：安全加密并存储用户账号信息，实现便捷登录。
-   [x] **二课活动浏览**：
    -   [x] 获取所有可报名的二课活动。
    -   [x] 查看已报名但未参加的活动。
    -   [x] 查看已参加并获得学时的活动。
    -   [x] 查看待申诉的学时项目。
-   [ ] **活动一键报名**：在应用内直接报名参加感兴趣的二课项目。
-   [x] **个性化推荐**：基于用户历史活动数据，智能推荐可能感兴趣的二课项目。
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

### 登录实现

参见 [登录](login.md)

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


## ❗ 错误处理规范 (Error Handling)

前端与后端交互目前接口设计为 `Result<T, String>`。为避免单纯返回一串不可解析的文本，推荐以下策略：

1. Rust 端仍可对外暴露 `Result<String, String>` / `Result<Json, String>`，但内部先构造结构化错误再 `serde_json::to_string`。
2. 统一错误 JSON 结构（前端收到后按 `code` 分支处理，`message` 做展示，`details` 可选）。

### 标准错误 JSON 结构

```json
{
  "code": "AUTH_FAILED",
  "message": "用户名或密码错误",
  "details": {
    "attempts": 1,
    "lock": false
  }
}
```

字段说明：
- `code`: 机器可读、稳定的错误码（全大写，下划线分隔）。
- `message`: 面向用户的可本地化文案（默认中文，可根据需要做 i18n）。
- `details`: 可选的附加数据，供前端做更细化的逻辑（不用于直接展示）。

### 推荐错误码列表

| 领域 | 错误码 | 场景说明 | 前端建议处理 |
| --- | --- | --- | --- |
| 登录 | `AUTH_FAILED` | 账号或密码错误 | 清空密码输入，提示重试 |
| 登录 | `AUTH_LOCKED` | 账号短期封锁 / 需要验证码 | 跳转安全验证界面 |
| 登录 | `AUTH_EXPIRED` | 登录状态失效 / Cookie 过期 | 触发重新登录流程 |
| 登录 | `AUTH_NETWORK` | 请求学校统一认证网络失败 | 显示“网络异常”并允许重试 |
| 通用 | `BAD_REQUEST` | 参数缺失或格式错误 | 高亮错误字段 |
| 通用 | `INTERNAL_ERROR` | 未捕获的内部异常 | 友好提示 + 允许反馈 |
| 通用 | `NOT_FOUND` | 目标活动 / 课表数据不存在 | 展示空态 |
| 活动 | `ACTIVITY_FULL` | 活动名额已满 | 禁用报名按钮 |
| 活动 | `ACTIVITY_CLOSED` | 报名截止 | 展示“已截止”标签 |
| 活动 | `ALREADY_REGISTERED` | 重复报名 | 切换为“已报名”状态 |
| 活动 | `REGISTER_FAIL` | 后端报名接口失败（原因不明） | 弹出失败提示，可重试 |
| 活动 | `APPEAL_UNAVAILABLE` | 当前无可申诉项目 | 展示空列表 |
| 推荐 | `NO_RECOMMENDATION` | 暂无足够历史数据 | 引导用户多参与活动 |
| 爬取 | `CRAWL_CHANGED` | 学校页面结构改变导致解析失败 | 通知用户更新版本 |
| 爬取 | `CRAWL_TIMEOUT` | 请求超时 | 提供刷新按钮 |
| 爬取 | `CRAWL_BLOCKED` | 被目标站限流/封锁 | 建议稍后再试 |
| 安全 | `ENCRYPT_FAIL` | 加密/解密用户凭据失败 | 强制登出并要求重新登录 |
| 课表 | `SCHEDULE_EMPTY` | 当前时间段无课表数据 | 展示空态 |

### Rust 实现示例

```rust
use serde::Serialize;

#[derive(Serialize)]
struct ApiError<'a> {
    code: &'a str,
    message: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
}

fn err(code: &'static str, message: &'static str, details: Option<serde_json::Value>) -> String {
    serde_json::to_string(&ApiError { code, message, details }).unwrap_or_else(|_| {
        // 兜底：若序列化失败
        format!("{\"code\":\"INTERNAL_ERROR\",\"message\":\"内部错误\"}")
    })
}

#[tauri::command]
async fn login(username: String, password: String) -> Result<bool, String> {
    if username.is_empty() || password.is_empty() {
        return Err(err("BAD_REQUEST", "账号或密码为空", None));
    }
    match perform_auth(&username, &password).await {
        Ok(true) => Ok(true),
        Ok(false) => Err(err("AUTH_FAILED", "用户名或密码错误", None)),
        Err(e) if e.is_network() => Err(err("AUTH_NETWORK", "网络异常，请稍后重试", None)),
        Err(_) => Err(err("INTERNAL_ERROR", "服务器内部错误", None)),
    }
}
```

### 前端处理建议

```ts
interface ApiError {
  code: string;
  message: string;
  details?: any;
}

function parseError(e: string): ApiError {
  try { return JSON.parse(e); } catch { return { code: 'INTERNAL_ERROR', message: e }; }
}

async function safeLogin(u: string, p: string) {
  const ok = await invoke<boolean>('login', { username: u, password: p })
    .catch(err => { const api = parseError(err); handleError(api); return false; });
  return ok;
}
```

### 错误信息文案规范
- `message` 面向用户，避免泄露内部实现；后台日志可额外记录堆栈。
- 不在 `message` 中包含敏感信息（账号、密码、token）。
- 可根据需要实现多语言：前端根据 `code` 做映射而非直接展示后端文本。

### 迁移计划（可选）
后续可将 `Result<T, String>` 重构为 `Result<T, ApiErrorStruct>` 并利用 `tauri::InvokeError` 自定义错误类型；当前阶段保持简单以加快迭代。

---

## 🚀 未来展望 (Roadmap)

*   [ ] **更完善的推荐算法**：引入协同过滤或内容推荐模型。
*   [ ] **支持 iOS**：扩展项目以支持在 iOS 平台上构建和运行。
*   [ ] **UI/UX 优化**：持续改进 Vant 组件库的用户体验。
*   [ ] **原生功能集成**：探索使用 Tauri 插件实现如系统通知、生物识别等原生移动功能。

---

## 📄 许可证 (License)

本项目采用 [MIT License](./LICENSE) 开源。
