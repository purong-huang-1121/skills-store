# skills-store

链上 DeFi 智能体技能包，集成主流协议操作与自动化交易策略。支持 Claude Code、OpenClaw 等 AI 编程助手。

## 包含技能

### 核心技能（dApp 协议集成）

`skills-store` 是主入口技能，整合以下所有协议能力：

| 协议 | 功能 |
|------|------|
| **Aave V3** | 查看市场、账户信息、存款、取款、借款、还款 |
| **Morpho Blue** | 市场列表、MetaMorpho 金库、存取款、持仓查询 |
| **Uniswap V3** | 链上代币兑换、报价、Token 搜索 |
| **Hyperliquid** | 永续合约/现货交易、资金费率、仓位管理 |
| **Ethena** | sUSDe 质押/赎回、APY 查询、余额 |
| **Polymarket** | 预测市场搜索、报价、买卖份额 |
| **Kalshi** | 美国合规预测市场、活动/市场浏览、交易 |
| **dapp-composer** | 跨协议组合操作、多步骤 DeFi 工作流 |

### 自动化策略技能

| 技能 | 子命令 | 描述 |
|------|--------|------|
| `strategy-auto-rebalance` | `auto-rebalance` | USDC 跨协议（Aave/Compound/Morpho）自动调仓，Base/Ethereum |
| `strategy-grid-trade` | `grid` | ETH/USDC 网格交易机器人，Base 链 |
| `strategy-ranking-sniper` | `ranking-sniper` | SOL 排行榜狙击策略，3 层安全过滤 + 6 层出场体系 |
| `strategy-signal-tracker` | `signal-tracker` | 聪明钱/KOL/巨鲸信号跟单，17 点安全过滤 |
| `strategy-memepump-scanner` | `scanner` | Pump.fun 迁移代币自动扫描交易，3 信号动量检测 |

## 快速安装

### 推荐方式（npx）

```bash
npx skills add skills-store
```

支持 Claude Code、OpenClaw、Cursor、Codex CLI 等环境，自动检测安装路径。

### Shell 脚本安装 CLI（macOS / Linux）

```bash
curl -sSL https://raw.githubusercontent.com/purong-huang-1121/skills-store/main/install.sh | sh
```

自动检测平台，下载对应二进制，验证 SHA256，安装至 `~/.cargo/bin/skills-store`。

## 配置

### OKX API（必须）

策略技能需要 OKX API 凭证，在 [OKX 开发者平台](https://web3.okx.com/onchain-os/dev-portal) 申请。

配置至 `~/.cargo/bin/.env`：

```env
OKX_API_KEY="your-api-key"
OKX_SECRET_KEY="your-secret-key"
OKX_PASSPHRASE="your-passphrase"
```

未配置时自动使用内置公共密钥（限速、不稳定，仅用于评估）。

### 钱包私钥（SOL/EVM 策略必须）

```env
# SOL 策略（ranking-sniper / signal-tracker / memepump-scanner）
SOL_PRIVATE_KEY="your-solana-private-key-base58"
SOL_ADDRESS="your-solana-address"

# EVM 策略（auto-rebalance / grid-trade）
EVM_PRIVATE_KEY="0x your-evm-private-key"
EVM_ADDRESS="0x your-evm-address"
```

### Telegram 通知（可选）

```env
TELEGRAM_BOT_TOKEN="your-bot-token"
TELEGRAM_CHAT_ID="your-chat-id"
```

## 支持链

Solana、Ethereum、Base、BSC、Arbitrum、Polygon、XLayer 及 20+ 其他链。

## 使用示例

**查询 Aave 市场利率**
> "查一下 Aave 上 USDC 的供款利率"

**开启排行榜狙击策略**
> "帮我启动 SOL 排行榜狙击"

**查看策略钱包余额**
> `skills-store ranking-sniper balance`

**网格交易**
> "帮我在 Base 上开一个 ETH/USDC 网格策略"

**聪明钱跟单**
> "帮我启动聪明钱信号跟单策略"

## 免责声明

- 内置公共 API Key 仅供测试评估，可能随时限速或不可用，由此产生的任何损失概不负责
- 自动化交易策略涉及真实资产，请充分了解风险后使用
- 生产环境请务必使用自己的 API Key 和钱包私钥，并妥善保管

## License

Apache-2.0
