<div align="center">

# 🐍 snk-rs

**Uma cobra animada que percorre o seu contribution graph do GitHub.**
**An animated snake that crawls your GitHub contribution graph.**

[![CI](https://github.com/junhinhow/snk-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/junhinhow/snk-rs/actions/workflows/ci.yml)
[![Forgejo CI](https://forgejo.multiservidores.com.br/ajr/snk-rs/actions/workflows/ci.yml/badge.svg)](https://forgejo.multiservidores.com.br/ajr/snk-rs/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/snk-rs.svg)](https://crates.io/crates/snk-rs)
[![docs.rs](https://img.shields.io/docsrs/snk-rs.svg)](https://docs.rs/snk-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-16a34a.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2021%20edition-orange.svg)](https://www.rust-lang.org/)

</div>

<div align="center">

![preview da snake](media/preview.svg)

</div>

---

## ✨ O que faz / What it does

**PT:** O `snk-rs` lê o calendário de contribuições do GitHub e transforma em um **SVG animado e autocontido** — sem JavaScript, sem dependências externas no momento da renderização. A cobra nasce no canto inferior esquerdo, percorre a grade com busca em largura (BFS) e **come cada dia que tem contribuição**, limpando a célula conforme passa.

**EN:** `snk-rs` reads the GitHub contribution calendar and turns it into a **self contained animated SVG** — no JavaScript, no runtime dependencies. The snake starts at the bottom left corner, walks the grid with a breadth first search and **eats every day that has contributions**, clearing the cell as it passes.

| Recurso / Feature | Detalhe / Detail |
|---|---|
| 🗓️ Duas fontes de dados | HTML público (sem token) ou GraphQL API (com token) / Public HTML (no token) or GraphQL API (with token) |
| 🎨 Paletas | `github`, `github-light`, `github-dark` + cores customizadas / plus custom colours |
| 📦 Saída | SVG único, animado por CSS, ~25 KB / single animated SVG, ~25 KB |
| 🧪 Testado | 27 testes unitários e de integração / 27 unit and integration tests |
| 🧩 Reutilizável | Biblioteca (`snk_rs`) + binário (`snk-rs`) / library + binary |
| 🪶 Leve | 6 dependências, sem runtime de JS / 6 dependencies, no JS runtime |

---

## 🙏 Inspiração / Inspiration

Este projeto é uma **implementação independente em Rust** da ideia popularizada pelo
[`Platane/snk`](https://github.com/Platane/snk) (MIT). A referência original inspirou o conceito e o visual; o código aqui é do zero, com outra arquitetura.

> Inspired by [`Platane/snk`](https://github.com/Platane/snk) by Platane — MIT licensed.
> This is an independent re-implementation written in Rust. No code was copied.

**O que muda em relação ao original / What differs from the original**

| | `Platane/snk` | `snk-rs` |
|---|---|---|
| Linguagem / Language | Go | **Rust** |
| Fonte pública / Public source | GraphQL com token | **HTML público, sem token** |
| Saída / Output | SVG + GIF | **SVG (GIF planejado)** |
| API / API | Docker action | **CLI + biblioteca Rust** |
| Uso da API / API usage | Container | **Binário nativo, ~1 MB** |

---

## 📦 Instalação / Installation

```bash
# Do repositório / From the repository
cargo install --git https://github.com/junhinhow/snk-rs.git

# Ou a partir do crates.io / Or from crates.io
cargo install snk-rs
```

<details>
<summary>Compilando do zero / Building from source</summary>

```bash
git clone https://github.com/junhinhow/snk-rs.git
cd snk-rs
cargo build --release
./target/release/snk-rs --help
```

Requer Rust 1.74+ / Requires Rust 1.74+
</details>

---

## 🚀 Uso / Usage

```bash
# SVG no terminal / SVG to the terminal
snk-rs --user junhinhow

# Salvar em arquivo / Save to a file
snk-rs --user junhinhow --output snake.svg

# Tema escuro com a cobra verde / Dark theme with a green snake
snk-rs --user junhinhow --palette github-dark --color-snake "#16A34A"

# Com token: contagens exatas e contribuições privadas / With a token: exact counts and private contributions
GITHUB_TOKEN=ghp_xxx snk-rs --user junhinhow
```

### Opções / Options

| Flag | Padrão / Default | Descrição / Description |
|---|---|---|
| `-u, --user <USER>` | *obrigatório / required* | Usuário do GitHub / GitHub user |
| `-o, --output <FILE>` | stdout | Arquivo de saída / Output file |
| `-p, --palette <NAME>` | `github-dark` | `github`, `github-light`, `github-dark` |
| `--color-snake <COLOR>` | roxo / purple | Cor da cobra / Snake colour (hex ou nome) |
| `--color-dots <COLORS>` | — | 5 cores (nível 0..4) / five colours, comma separated |
| `--cell <PX>` | `11` | Lado da célula / Cell size |
| `--gap <PX>` | `2` | Espaço entre células / Space between cells |
| `--margin <PX>` | `4` | Margem externa / Outer margin |
| `--duration <SEC>` | `10` | Duração do ciclo / Length of one loop |
| `--from <DATE>` | — | Início do período (`YYYY-MM-DD`) / Start of the period |
| `--to <DATE>` | — | Fim do período / End of the period |
| `--token <TOKEN>` | `$GITHUB_TOKEN` | Token do GitHub / GitHub token |

### 🐙 GitHub Actions

```yaml
name: generate contribution snake

on:
  schedule:
    - cron: "0 0 * * *"
  workflow_dispatch:

permissions:
  contents: write

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
        with:
          toolchain: stable
      - run: cargo install --path .
      - run: snk-rs --user ${{ github.repository_owner }} --output snake.svg
      - uses: peaceiris/actions-gh-pages@v4
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: .
          publish_branch: output
```

> ⚠️ O token é **opcional**. Sem ele só aparecem contribuições públicas — que é o
> comportamento desejado num README de perfil. Com `GITHUB_TOKEN` a API GraphQL
> entra em ação e o contador exato de cada dia é usado.
>
> The token is **optional**. Without it only public contributions show up, which
> is what you want on a profile README. With `GITHUB_TOKEN` the GraphQL API is
> used and the exact daily count is taken into account.

### 🦊 Forgejo Actions

O mesmo workflow funciona no Forgejo, trocando apenas a action do checkout:

```yaml
      - uses: code.forgejo.org/actions/checkout@v4
```

---

## 🏗️ Arquitetura / Architecture

```
src/
├── main.rs            # entrypoint da CLI / CLI entrypoint
├── lib.rs             # raiz da biblioteca / library root
├── cli.rs             # definição dos argumentos / argument definitions
├── palette.rs         # paletas e cores / palettes and colours
├── grid.rs            # grade de dias / grid of days
├── snake.rs           # planeamento do caminho (BFS) / path planning (BFS)
├── render.rs          # geração do SVG / SVG generation
├── error.rs           # tipos de erro / error types
└── github/
    ├── mod.rs         # cliente HTTP / HTTP client
    ├── model.rs       # ContributionDay, ContributionCalendar
    ├── html.rs        # página pública de contribuições / public contributions page
    └── graphql.rs     # API GraphQL / GraphQL API
```

**Fluxo / Flow**

```
GitHub ──▶ GitHubClient ──▶ ContributionCalendar ──▶ Grid ──▶ SnakePlan ──▶ SVG
```

- **`Grid`** alinha as semanas de segunda a domingo, exatamente como o GitHub desenha, e garante que nenhum dia sobrescreva outro.
- **`SnakePlan`** usa BFS para sempre encontrar o dia com contribuição mais próximo; como a cobra pode cruzar o próprio rastro, ela nunca fica presa.
- **`render`** reduz o caminho aos cantos, o que mantém o SVG pequeno, e usa uma única animação de `stroke-dashoffset` para o rastro mais um `@keyframes` para a cabeça.

---

## 🎨 Paletas / Palettes

```bash
snk-rs --user junhinhow --palette github-light
snk-rs --user junhinhow --palette github-dark --color-snake emerald
snk-rs --user junhinhow --color-dots "#161b22,#0e4429,#006d32,#26a641,#39d353"
```

Cores acceptadas em hexadecimal (`#16a34a`, `16a34a`, `#abc`) ou por nome
(`green`, `emerald`, `purple`, `red`, ...). / Colours can be given in hex or by
name.

---

## 🗺️ Roadmap

- [ ] Exportação em GIF (hoje é só SVG)
- [ ] Temas com gradiente e fundo transparente
- [ ] Opção para desenhar a cobra com corpo segmentado
- [ ] Publicação no `crates.io`
- [ ] Modo `--dry-run` que só mostra o resumo do calendário

---

## 🤝 Contribuindo / Contributing

Leia o [CONTRIBUTING.md](CONTRIBUTING.md). Resumo / Summary:

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
```

---

## 📜 Licença / License

[MIT](LICENSE) — © Altair Jr (junhinhow)

Este projeto é uma implementação independente inspirada em
[`Platane/snk`](https://github.com/Platane/snk) de Platane, sob licença MIT.
Consulte [NOTICE](NOTICE).
</div>
