# 后端与 rustustc 模块说明

本文件描述 `src-tauri` 目录下 Rust 侧的主要模块、公开的 Tauri Commands 以及安全与测试注意事项。

## 架构速览

```
前端 invoke
   │
   ▼
lib.rs Tauri commands
   │
   ├─ auth：CAS 登录、凭据加密存储/读取
   ├─ rustustc::cas：CASClient（会话 & Cookie）
   ├─ rustustc::young：YouthService（二课网 API 封装）+ model（实体/过滤器）
   ├─ recommend：基于历史活动的简单文本/标签推荐
   ├─ security：基于 machine_uid 的 AES-GCM 加解密
   └─ state：全局 AppState，持有 CASClient / YouthService（Arc + Mutex）
```

### 数据流
1. 前端调用 `login`，auth::perform_login 使用 `CASClient::login_by_pwd` 获取 CAS Cookie，随后用该 Cookie 初始化 `YouthService` 并保存到全局状态。
2. 其他 command 通过 `get_service` 从 `AppState` 中取出 `YouthService`，并调用二课接口。
3. `auth::save_credentials` 将用户名明文 + 密码密文存入 `tauri-plugin-store`；`security::encrypt_data` 使用本机 machine_uid 派生密钥，保证密文在其他设备不可解。
4. `auth::try_auto_login` 在内存未命中时尝试解密凭据并重新登录。

## Tauri Commands（lib.rs）
| 函数 | 参数 | 返回 | 说明 / 错误码 | 备注 |
| --- | --- | --- | --- | --- |
| `login` | `username: String`, `password: String`, `save: bool` | `Result<serde_json::Value, String>` | 失败时返回 `{code,message}` JSON 字符串；成功时返回用户信息 JSON | `save=true` 会落盘加密凭据；依赖网络 |
| `get_login_status` | - | `Result<serde_json::Value, String>` | `logged_in: bool`，若未登录包含 `has_stored_creds` 与 `username` | 会尝试自动登录（若有密文） |
| `logout` | - | `Result<(), String>` | 清空 state 中的会话并删除存储的密码 | 仅删除密码，用户名保留 |
| `refresh_session` | - | `Result<serde_json::Value, String>` | 重新基于当前 CAS Cookie 刷新 YouthService，失败返回 `{code:"INTERNAL_ERROR",...}` | Cookie 过期将报错 |
| `get_unended_activities` | - | `Result<serde_json::Value, String>` | 未结束活动列表 | 调用 `SecondClass::find`（不展开系列） |
| `get_registered_activities` | - | `Result<serde_json::Value, String>` | 已报名/报名结束列表 | 过滤 `Status::Applying|ApplyEnded` |
| `get_participated_activities` | - | `Result<serde_json::Value, String>` | 已参加/已结项列表 | 过滤掉正在报名的 |
| `register_for_activity` | `activity_id: String` | `Result<bool, String>` | `true` 代表报名成功 | 先 `update` 再 `apply`，若时间冲突会尝试自动取消冲突活动后重试 |
| `get_recommended_activities` | - | `Result<serde_json::Value, String>` | 推荐活动列表（最多 10 条） | 基于历史活动的 TF/标签/部门得分 |
| `get_activity_children` | `activity_id: String` | `Result<serde_json::Value, String>` | 系列课子项目列表 | 非系列课返回 `NOT_A_SERIES` 错误 JSON |
| `get_activity_detail` | `activity_id: String` | `Result<serde_json::Value, String>` | 获取项目详细内容 | 如报名人数需要通过这个才能获得 |
| `get_class_schedule` | - | `Result<serde_json::Value, String>` | 暂未实现，返回空数组 | 预留 |
| `get_pending_appeals` | - | `Result<serde_json::Value, String>` | 暂未实现，返回空数组 | 预留 |

> 错误返回均为 JSON 字符串 `{ code, message }`，前端应先尝试 `JSON.parse` 再做兜底展示。

## rustustc::cas
- **CASClient**：基于 `tauri-plugin-http::reqwest` + CookieStore。
  - `login_by_pwd(username?, password?)`：解析 CAS 登录页的 crypto/flowkey，用 ECB-AES128 加密口令并提交。支持从环境变量 `USTC_CAS_USR` / `USTC_CAS_PWD` 读取（用于测试）。
  - `is_login()`：GET 登录页，若跳转则视为已登录。
  - `logout()`：访问 `gate/logout`。
  - `get_info()`：调用 `gate/getUser` / `getPersonId` / `userInfo` 组合接口，返回 `UserInfo`。
- **风险点**：
  - 使用 ECB（学校接口要求），仅用于登录密码加密，不存储明文。
  - 依赖正则解析 HTML，若 CAS 页面改版需更新。
  - 环境变量缺失时会报错；移动端生产路径依赖 UI 输入而非 env。

## rustustc::young
- **YouthService**：
  - 基于 CAS SSO ticket 获取 `token`，后续请求需 `X-Access-Token` + AES-CBC 加密参数。
  - `request/get_result/page_search`：统一做加密、分页与重试（默认 3 次）。
- **model**：
  - `TimePeriod`、`Module/Department/Label`、`SCFilter`（筛选器，支持名称/模块/部门/标签/时间段，附加本地 check）。
  - `User` / `SignInfo`：获取当前用户、联系方式等。
  - `SecondClass`：活动实体，支持 `find`、`get_participated`、`apply`、`cancel_apply`、`update`、`get_children`。
- **风险点**：
  - `YouthService::encrypt` 依赖 token 长度 >= 32；若接口变更，需显式校验。
  - `SecondClass::apply` 在时间冲突时自动取消冲突活动后重试，逻辑依赖接口提示字符串包含“时间冲突”。
  - 多数字段解析使用 `unwrap_or_default`，前端显示应兼容缺失值。

## recommend
- 使用 jieba 分词，对活动名称/部门/简介做 TF 权重，结合历史活动的部门/模块加分，过滤已参与活动。
- 网络依赖：需要先获取历史活动、当前候选列表。

## auth
- `perform_login`：CAS 登录 + YouthService 初始化 + 获取当前用户，并写入 `AppState`。
- `save_credentials`：以 machine_uid 派生密钥 AES-GCM 加密密码，存入 `tauri-plugin-store`。
- `try_auto_login`：先检查内存 state，再尝试读取并解密本地密文后自动登录。
- `clear_credentials`：仅删除密码，保留用户名便于自动填充。
- **风险点**：机器变更/密钥不同会导致解密失败，此时返回未登录。

## security
- `encrypt_data` / `decrypt_data`：AES-256-GCM，密钥 = SHA256(machine_uid + salt)。
- `machine_uid` 获取失败时会使用固定字符串作为 fallback，安全性降低但保证可用性；生产可考虑改为“失败则拒绝登录”。

## state
- `AppState`：`Mutex<Option<Arc<CASClient>>>` 与 `Mutex<Option<Arc<YouthService>>>`。所有 command 通过锁获取当前会话。

## 测试与环境变量
- 集成测试需网络与真实账号：
  - `.env` 中提供 `USTC_CAS_USR`、`USTC_CAS_PWD`（对应 CAS 账号与密码）。
  - `cargo test -p better_ustc_2_lib -- --ignored` 可用于标记网络相关用例。
- 建议新增的离线单元测试：
  - `SCFilter::check`、`TimePeriod` 解析/重叠判断。
  - `SecondClass::applyable` 等纯逻辑分支。
  - `security::encrypt_data/decrypt_data` 循环测试（需 mock machine_uid 时可注入固定 key）。

## 前端使用提示
- 所有错误字符串请先尝试 `JSON.parse`；若失败再展示原文本。
- 登录后如收到 `AUTH_REQUIRED` 或 `{code:"INTERNAL_ERROR", message:"Session expired..."}` 应提示重新登录并清理本地状态。
- 推荐列表可能为空：显示“暂无推荐”，并引导多参与活动以丰富画像。

## 可能的改进方向
- 将错误类型从 `String` 收敛为统一 `ApiError` 结构并实现 `Into<tauri::InvokeError>`。
- `YouthService::encrypt` 增加 token 长度检查与更友好错误信息。
- 将自动取消冲突的策略改为前端确认或返回冲突列表，避免误操作。
- 引入缓存（如活动列表、标签列表）减少重复请求。
- 为存储的用户名也做轻度加密或混淆，减少泄露风险。
