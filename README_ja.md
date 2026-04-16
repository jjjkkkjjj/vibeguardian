# vibeguardian (`vg`)

> VibeCodingで秘匿データをAIから守るCLIツール

`vg run -- npm run dev` のように普段のコマンドをラップするだけで、AIアシスタントがAPIキーに物理的に触れられない開発環境を構築します。

English documentation: [README.md](README.md)

---

## 仕組み

| 機能 | 説明 |
|---|---|
| **Inject Mode** | `~/.vibeguard/secrets.json` から実際のキーを読み込み、子プロセスのメモリ上にだけ展開。ディスクや `.env` には書き出しません。 |
| **Proxy Mode** | ローカルでリバースプロキシ（デフォルト `:8080`）を起動し、AIが生成したコードの `localhost:8080/proxy/stripe` 宛てリクエストに `Authorization` ヘッダを裏で付与して転送。 |
| **Log Mask Mode** | 子プロセスの stdout/stderr をリアルタイムでフックし、シークレット文字列が含まれていれば `***[MASKED]***` に置換して表示。 |

---

## インストール

### ダウンロード（推奨）

[GitHub Releases](https://github.com/jjjkkkjjj/vibeguardian/releases) から最新バイナリをダウンロードして `$PATH` に置いてください。

```bash
# macOS (Apple Silicon)
curl -L https://github.com/jjjkkkjjj/vibeguardian/releases/latest/download/vs-vX.X.X-aarch64-apple-darwin.tar.gz | tar xz
sudo mv vs /usr/local/bin/

# macOS (Intel)
curl -L https://github.com/jjjkkkjjj/vibeguardian/releases/latest/download/vs-vX.X.X-x86_64-apple-darwin.tar.gz | tar xz
sudo mv vs /usr/local/bin/

# Linux (x86_64)
curl -L https://github.com/jjjkkkjjj/vibeguardian/releases/latest/download/vs-vX.X.X-x86_64-unknown-linux-gnu.tar.gz | tar xz
sudo mv vs /usr/local/bin/
```

### cargo でビルド

```bash
cargo install --git https://github.com/jjjkkkjjj/vibeguardian vs
```

---

## クイックスタート

```bash
# 1. プロジェクトに vibeguard.toml を生成
vg init

# 2. シークレットをグローバル領域に保存（プロジェクトには絶対保存しない）
vg set stripe/secret_key              # 入力エコーなし（対話プロンプト）
vg set openai/api_key sk_...          # 引数渡しも可（シェル履歴に残る旨を警告）

# 3. 安全な環境でアプリを起動
vg run -- npm run dev
vg run --profile prod -- node server.js
```

実行時のターミナル出力例：

```
[Vibeguard] Proxy started at http://localhost:8080
[Vibeguard] Injected 2 env var(s) (profile: dev)
[Vibeguard] Log masking enabled
> next dev
...
```

---

## `vibeguard.toml` — 設定ファイル

プロジェクトルートに置きます。**実際のAPIキーは一切含まず、Gitにコミットしても安全**です。

```toml
[project]
name = "my-app"
default_profile = "dev"

# ── Inject Mode ──────────────────────────────────────────────────────────────
# secret:// で始まる値は自動的に解決されます
# secret://global/...  → ~/.vibeguard/secrets.json          （全プロジェクト共有）
# secret://project/... → ~/.vibeguard/projects/my-app/secrets.json （このプロジェクト専用）
[env.dev]
DATABASE_URL        = "secret://global/supabase/dev_db_url"
STRIPE_KEY          = "secret://project/stripe/secret_key"   # プロジェクトスコープ
NEXT_PUBLIC_API_URL = "http://localhost:8080/proxy/api"   # プロキシ経由なので平文でOK

[env.prod]
DATABASE_URL = "secret://global/supabase/prod_db_url"
STRIPE_KEY   = "secret://project/stripe/secret_key"

# ── Proxy Mode ───────────────────────────────────────────────────────────────
[proxy]
port = 8080  # デフォルト

[[proxy.routes]]
path   = "/proxy/stripe"
target = "https://api.stripe.com"
inject_headers = { Authorization = "Bearer ${secret://project/stripe/secret_key}" }

[[proxy.routes]]
path   = "/proxy/openai"
target = "https://api.openai.com/v1"
inject_headers = { Authorization = "Bearer ${secret://global/openai/api_key}" }
```

AIは `vibeguard.toml` を読んで「リクエスト先は `localhost:8080/proxy/stripe` だな」と理解してコードを書きますが、`secret://` の実際の値には物理的に触れられません。

---

## コマンドリファレンス

### `vg run [OPTIONS] -- <CMD>`

| フラグ | 説明 |
|---|---|
| `-p, --profile <PROFILE>` | 使用する環境プロファイル（デフォルト: `dev`） |
| `--no-mask` | ログマスクを無効化（非推奨） |
| `--no-proxy` | ローカルプロキシを起動しない |
| `-- <CMD>` | 実行するコマンド（例: `npm run dev`） |

### `vg init`

カレントディレクトリに `vibeguard.toml` テンプレートを生成します。既に存在する場合はエラー。

### `vg set <PATH> [VALUE]`

デフォルトでは **グローバルストア**（`~/.vibeguard/secrets.json`）に保存します。  
`--project` を指定すると、カレントディレクトリの `vibeguard.toml` からプロジェクト名を読み取り、  
**プロジェクトスコープストア**（`~/.vibeguard/projects/<name>/secrets.json`）に保存します。

| フラグ | 説明 |
|---|---|
| `--project` | グローバルの代わりにプロジェクトストアに保存 |

- `VALUE` を省略するとエコーなし入力プロンプトを表示
- `VALUE` を引数で渡した場合、シェル履歴への露出を警告

```bash
vg set stripe/secret_key                   # → ~/.vibeguard/secrets.json
vg set --project stripe/secret_key sk_...  # → ~/.vibeguard/projects/my-app/secrets.json
```

### `vg status`

カレントディレクトリの `vibeguard.toml` を読み取り、注入予定の環境変数名とプロキシルート一覧を**値をマスクして**表示します。

---

## セキュリティ設計

- 全シークレットファイルをファイルパーミッション `0o600`（所有者のみ読み書き可）で書き込みます
- `vibeguard.toml` に実値は一切含まれません
- ログマスクは Aho-Corasick 法による O(n) の線形時間処理で、大量ログでもパフォーマンスへの影響はありません

### シークレットスコープ

| スコープ | URI プレフィックス | 保存先 | 用途 |
|---|---|---|---|
| **global** | `secret://global/...` | `~/.vibeguard/secrets.json` | 全プロジェクト共有キー（个人の OpenAI キーなど） |
| **project** | `secret://project/...` | `~/.vibeguard/projects/<name>/secrets.json` | プロジェクト専用キー（Stripe のテストキーなど） |

```bash
# グローバルに保存（全プロジェクト共有）
vg set global/openai/api_key

# このプロジェクト専用に保存（vibeguard.toml からプロジェクト名を取得）
vg set --project stripe/secret_key
```

---

## 開発

```bash
# Docker 環境でビルド
docker compose run --rm dev cargo build

# テスト
docker compose run --rm dev cargo test

# Lint
docker compose run --rm dev cargo clippy -- -D warnings

# ホストから手軽に実行
./cargo-docker build
./cargo-docker test
./cargo-docker clippy -- -D warnings
```

### ファイル構成

```
src/
├── main.rs          エントリーポイント
├── lib.rs           ライブラリルート（tests/ から参照）
├── cli.rs           Clap コマンド定義
├── inject/          Inject Mode — env var 解決ヘルパー
├── mask/            Log Mask Mode — Aho-Corasick ベースのマスカー
├── proxy/           Proxy Mode — axum リバースプロキシ
├── config/          設定ファイルとシークレットストアのパーサ
└── commands/        サブコマンド実装
tests/
├── mask.rs          LogMasker インテグレーションテスト
└── config/
    ├── resolver.rs  secret:// 解決ロジックのテスト
    └── secrets.rs   シークレットストア参照ロジックのテスト
```

---

## ライセンス

MIT
