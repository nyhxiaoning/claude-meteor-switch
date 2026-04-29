# Claude Dynamic Meteor

本地代理网关，让 Claude Code 通过 `/model opus|sonnet|haiku` 动态路由到不同 LLM 厂商，支持 Anthropic/OpenAI 双协议格式双向转换。

## Tech Stack

- **Desktop**: Tauri 2.0 (Rust + React)
- **Proxy**: axum (嵌入 Tauri 进程)
- **HTTP**: reqwest (SSE 流式)
- **Frontend**: React + TypeScript + shadcn/ui + Tailwind + react-router-dom (HashRouter)
- **DB**: SQLite (rusqlite bundled, Arc\<Mutex<Connection>>)
- **IPC**: Tauri Commands + Channel API
- **Theme**: next-themes (深色/浅色/跟随系统)

## Architecture

```
Claude Code → ANTHROPIC_BASE_URL=http://127.0.0.1:9876 → axum proxy
  → model 带边界关键词匹配 → 确定厂商
  → Anthropic 格式上游: 最小化旁路解析透传 (提取 usage)
  → OpenAI 格式上游: 请求/响应双向转换 (含 SSE 流式状态机)
```

路由优先级: 精确匹配 > 带边界关键词匹配 > 第一个启用的默认厂商
关键词匹配: `(^|-)keyword(-|$)` 正则，保存时做冲突检测

## Project Structure

```
claude-dynamic-meteor/
├── src-tauri/src/
│   ├── proxy/          # axum server + handler + router + stream (端口冲突检测+自动递增)
│   ├── adapter/        # LlmAdapter trait: anthropic(旁路usage) / openai(转换)
│   ├── config/         # Provider 结构 + SQLite 持久化 (keyring/aes-gcm)
│   ├── claude/         # settings.json 读写 (一键对接/还原)
│   ├── commands/       # Tauri Commands: server/provider/log/stats/claude
│   ├── db/             # SQLite: logs/migration (Arc<Mutex<Connection>>, 读时聚合)
│   └── monitor/        # Channel 实时推送 + SQLite 写入协调
├── src/
│   ├── components/     # layout/dashboard/providers/logs/claude/settings/shared/ui
│   ├── hooks/          # useProxyChannel/useProviders/useProxyServer/useStats
│   └── lib/            # tauri 封装, types
```

## Key Conventions

- `LlmAdapter` trait 统一返回 Anthropic SSE 流，`AnthropicAdapter` 旁路提取 usage 直通、`OpenAIAdapter` 内聚转换
- SSE 流式转换用 `OpenAIToAnthropicConverter` 状态机 (content\_block\_index, active\_tool\_calls: HashMap, text\_block\_started, message\_started)
- SQLite 写入用 `spawn_blocking` + `Arc<Mutex<Connection>>` 不阻塞代理；Channel 实时推送 + SQLite 持久化双通道
- 日志默认不存储请求体/响应体，"详细日志模式"开关控制（单条限 50KB）
- 统计采用读时聚合（从 request\_logs GROUP BY），不使用 daily\_stats 物化表
- API Key 用 `keyring` crate 安全存储 (三平台)，降级为 `aes-gcm` 加密文件
- 跨平台路径用 `dirs` crate；仅监听 `127.0.0.1`；端口冲突时自动递增
- UI 组件全部 shadcn/ui，图表 recharts，路由 react-router-dom HashRouter
- 各页面有空状态引导组件 (EmptyState)

## Format Conversion (Anthropic ↔ OpenAI)

**请求**: system 顶层→messages\[0] system; `tools[].input_schema`→`tools[].function.parameters`; `tool_use`→`function_call`; `tool_result`→独立 `role:tool` message

**响应 (流式 SSE)**: 首个 chunk→`message_start`+`content_block_start(text)`; delta.content→`content_block_delta`; 首个 tool\_call→关闭 text block→`content_block_start(tool_use)`+`input_json_delta`; 多 tool\_call (不同 index)→逐个关闭+新建; finish→`content_block_stop`+`message_delta`+`message_stop`; `stop`→`end_turn`, `tool_calls`→`tool_use`

**已知限制**: OpenAI 格式上游不支持 thinking/extended\_thinking 内容块

## Claude Code Integration

写入 `~/.claude/settings.local.json`:

```json
{ "env": {
    "ANTHROPIC_BASE_URL": "http://127.0.0.1:9876",
    "ANTHROPIC_API_KEY": "sk-ant-api03-proxy-placeholder-000...",
    "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC": "true"
}}
```

## Error Handling

上游不可达→502; 超时→504; 路由未匹配→400; 格式转换失败→500; 端口冲突→自动递增; OpenAI 错误需转 Anthropic 格式; 客户端断开主动关闭上游

## Data

- 日志保留 90 天，启动自动清理；`user_version` 管理迁移（启动时检测+提示）；支持 JSON/CSV 导出
- Provider 持久化到 SQLite providers 表，API Key 加密存储
- 统计读时聚合，不走物化 daily\_stats 表

## Phases

1. 骨架 + Anthropic 直通(旁路usage) + SQLite + 厂商 CRUD + 主题 + 空状态
2. OpenAI 格式适配 (请求/响应转换 + SSE 多tool\_call状态机)
3. 监控仪表盘 + 日志(读时聚合) + Claude Code 一键对接
4. 工具调用映射 + keyring + 系统托盘 + 开机自启
5. 打包与分发 (macOS .dmg / Windows .msi / Linux .AppImage)

## Other

需求地址在当前目录下的 PRD-PLAN.md 中

