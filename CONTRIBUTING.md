# Contribuindo / Contributing

Obrigado por querer contribuir! Este projeto é pequeno e direto, e as mesmas
regras valem para GitHub e Forgejo.

Thank you for wanting to contribute! This project is small and focused, and the
same rules apply on GitHub and Forgejo.

## Ambiente / Environment

```bash
rustup update stable
git clone https://github.com/junhinhow/snk-rs.git
cd snk-rs
cargo build
```

Requer Rust 1.74 ou superior. / Requires Rust 1.74 or newer.

## Antes de abrir um PR / Before opening a PR

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Os três precisam passar. A CI roda exatamente esses comandos.

All three must pass. CI runs exactly those commands.

## Regras de estilo / Style rules

- Rust idiomático, sem `unwrap()` fora de testes.
- Idempotent Rust, no `unwrap()` outside of tests.
- Comentários e documentação em inglês no código; README bilíngue (PT/EN).
- Comments and documentation in English in the code; bilingual README (PT/EN).
- Toda função pública precisa de `///` com pelo menos uma frase.
- Every public function needs a `///` doc comment with at least one sentence.
- Novas dependências precisam ser justificadas no PR.
- New dependencies must be justified in the PR.

## Commits / Commits

Mensagens no formato Conventional Commits:

```
feat: add GIF export
fix: keep the Sunday of each week in its own cell
docs: explain the BFS planner
```

## O que combina / What fits

- Um novo palette ou uma nova forma de cor.
- Melhorias no renderizador que reduzam o tamanho do SVG.
- Exportação em GIF.
- Opções de linha de comando que não quebrem a interface atual.

## O que não combina / What does not fit

- Dependências pesadas que não agregam valor.
- Funcionalidade que dependa de um serviço pago ou de uma chave obrigatória.
- Código copiado de outros projetos sem a devida atribuição.

## Licença / License

Ao contribuir você concorda com a [MIT](LICENSE).
By contributing you agree to the [MIT](LICENSE) license.
