# Ownership

Rust の所有権を、暗記するルールではなく**世界の物理法則**として体験するパズルゲーム。
コードは一行も書きません。

A puzzle game where Rust's ownership rules *are* the rules of the world.
You never type code — you move a crate between scopes, and the program writes itself.

**▶ [ブラウザで遊ぶ / Play in the browser](https://yosihitoyasudasub.github.io/rust-game/)**

---

## 遊び方

- **値は箱**。世界に1つしか存在しません
- **スコープは部屋**。ドアに**関数シグネチャ**が書いてあります
- 箱を矢印の先の部屋へドラッグする = **ムーブ**
- 渡した後の変数は取り消し線になり、触ると `error[E0382]` が出ます
- `fn log(s: String)` のように**値を返さない**部屋に入れると、スコープの終わりで箱は破棄されます

右のパネルには、いま自分がやった操作に対応する Rust のソースが**1行ずつ書き足されていきます**。
ドラッグ中は「これから書かれる行」が先に見えます。

| キー | |
|---|---|
| Space / クリック | ブリーフィングを閉じる |
| H | このステージの狙いを再表示 |
| C | コメント表示の切り替え |
| R | やり直し |
| N | 次のステージへ |
| L | 日本語 / English |

## 設計方針

- **パズルとして面白いことが先、Rust は題材** — 学べるだけの退屈なものにはしない
- **メンタルモデルが先、構文は後** — 先に体で分かってから、対応するコードを見せる
- **答えを言わない** — シグネチャの言い換えまでは助けるが、「だから破棄される」は言わない。それを結び付けるのがプレイヤーの仕事

## ビルド

```sh
cargo run                       # デスクトップ
```

ブラウザ向け:

```sh
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/ownership.wasm docs/
```

`docs/start.cmd`（Windows）をダブルクリックするとローカルサーバが立ち上がってブラウザが開きます。
`docs/index.html` を直接開いても動きません — wasm の取得に `fetch()` を使うため、`file://` では
CORS で拒否されます。HTTP 経由で配信する必要があります。

### フォント

画面に出る文字だけに絞り込んだサブセットを埋め込んでいます（wasm 全体で約 766 KB / gzip 268 KB）。
**テキストを追加・変更したら必ず再生成してください。** しないと新しい文字が空白になります。

```sh
cargo run --release --manifest-path tools/subset-font/Cargo.toml
```

ライセンスは [assets/LICENSES.md](assets/LICENSES.md) を参照。

## 技術構成

[macroquad](https://github.com/not-fl3/macroquad) 単体。WASM バンドルが小さく、
静的ファイルを置くだけで配信できるのが選定理由です。

## ライセンス

コードは MIT。同梱フォントは SIL Open Font License 1.1（[詳細](assets/LICENSES.md)）。
