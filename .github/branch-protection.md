# 分支保护与必需检查

这份文件是 `main` 分支保护设置的**唯一记录**。平台上的开关无法被版本控制，所以在这里写清
「应该是什么」，并在最后给出核对命令——换机器、换维护者或重建仓库时照此复现。

规则依据见宪法的「分支与提交」「拉取请求」「CI 门禁」三节。

## 应该开启的设置

| 设置 | 值 | 理由 |
| --- | --- | --- |
| 目标分支 | `main` | 唯一长期分支 |
| Require a pull request before merging | 开 | 宪法禁止直接提交到主分支 |
| Required approvals | `0` | 单开发者仓库条款：平台不允许自己批准自己的 PR，审批由 PR 描述中的自我证明承担 |
| Dismiss stale approvals | 开 | 新增提交即需重新确认（多维护者后才有实际意义） |
| Require status checks to pass | 开 | 下方十二个检查全部勾选 |
| Require branches to be up to date | 开 | 避免「本地绿、合并后红」 |
| Do not allow bypassing the above settings | 开 | 否则管理员仍可直接推送，等于没开 |
| Allow force pushes / deletions | 关 | 历史只增不改 |

出现第二个活跃维护者时，把 Required approvals 改为 `1`，并恢复宪法要求的独立审查；
本文件的其余部分不变。

## 必需检查（十二个）

名称必须与 `ci.yml` 里的 job `name:` 逐字一致，改 job 名就要同步改这里与平台设置。

| 检查名 | 覆盖内容 | 对应原则 |
| --- | --- | --- |
| `cargo fmt --check` | 格式 | II |
| `cargo clippy -D warnings` | clippy + `cargo machete` | II |
| `cargo test` | `cargo test --all-targets --all-features` | III |
| `desktop (windows)` | 桌面端 clippy（图形栈，单独 Windows job） | II |
| `mobile (flutter)` | codegen 漂移检查 + `flutter analyze` + `flutter test` | III |
| `android (build apk)` | Android debug APK（NDK 交叉编译） | III |
| `ios (build, no codesign)` | iOS debug 构建（不签名） | III |
| `Constitution governance scan` | 文件行数上限、TODO 归属、`#[allow(dead_code)]` 拦截 | I / II |
| `cargo audit` | 依赖漏洞 | VIII |
| `cargo deny check` | 许可证白名单、禁用依赖、依赖来源与撤回版本 | VIII |
| `Dependency review` | PR 引入的新依赖是否有高危漏洞（仅 PR 事件运行） | VIII |
| `Secret scan` | 私钥块、云密钥、平台令牌、误提交的 `.env` | VIII |

## 核对命令

```bash
# 分支保护是否开启、必需检查是否齐全
gh api repos/{owner}/{repo}/branches/main/protection \
  --jq '{approvals: .required_pull_request_reviews.required_approving_review_count,
         checks: [.required_status_checks.contexts[]]}'

# CI 里实际存在的 job 名（与上表比对）
gh workflow view ci.yml --json jobs --jq '[.jobs[].name]'
```

两份输出的差异就是漂移：要么改 `ci.yml` 的 job 名，要么改本表与平台设置，不要放着不管。

## 未覆盖的部分

分支保护管不到这些，仍靠人和 CI 守护：

- 宪法要求的「涉及认证、授权、密钥或加密的变更额外走安全审查」——没有自动化检查。
- 单开发者自我证明的完整性——只能靠 PR 模板里的清单自查。
- `CODEOWNERS` 的两人批准要求——批准数设为 0 时不会生效，属于已知取舍。
