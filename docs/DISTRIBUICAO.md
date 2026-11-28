# Distribuição e assinatura dos instaladores

Os instaladores do **Ferro Studio** são gerados pelo workflow
[`.github/workflows/release.yml`](../.github/workflows/release.yml) com o
[`tauri-action`](https://github.com/tauri-apps/tauri-action):

| Plataforma | Arquivos |
|---|---|
| Windows (x86_64 e ARM64) | `.exe` (NSIS, instala para o usuário, sem pedir administrador; x64 e ARM64) e `.msi` (x64) |
| Linux (x86_64 e ARM64) | `.deb`, `.rpm`, `.AppImage` e `.pkg.tar.zst` (Arch Linux) |
| macOS (Apple Silicon ARM64 e Intel x86_64) | `.dmg` e `.app` |

No **Arch Linux** (`x86_64` e `aarch64`), os pacotes `.pkg.tar.zst` são gerados automaticamente pelo grupo `linux_arch` em [`.github/workflows/release.yml`](../.github/workflows/release.yml) usando [`packaging/arch/PKGBUILD`](../packaging/arch/PKGBUILD) e [`packaging/arch/build-package.sh`](../packaging/arch/build-package.sh). Para instalar um pacote baixado: `sudo pacman -U portugol-studio-rs-*.pkg.tar.zst`. Para gerar localmente: `bash packaging/arch/build-package.sh`.

## Como publicar uma versão

O número da versão e o histórico vêm dos commits, que seguem o padrão
[Conventional Commits](https://www.conventionalcommits.org/pt-br/) (`feat:`,
`fix:`, `docs:` etc.). Não é preciso editar versões à mão.

1. Garanta que a `main` está com o CI verde.
2. Em *Actions → Preparar release → Run workflow*, escolha a versão:
   - `automatica` (padrão): `feat` sobe o número do meio (0.1.0 → 0.2.0);
     só `fix` e outros tipos sobem o último (0.1.0 → 0.1.1); mudança
     incompatível (`feat!:` ou `BREAKING CHANGE`) sobe o primeiro a partir da
     1.0;
   - `patch`, `minor` ou `major` para forçar o tipo;
   - um número exato, como `1.0.0`.
3. O workflow atualiza `Cargo.toml`, `Cargo.lock`, `package.json`,
   `package-lock.json` e `tauri.conf.json`, acrescenta a versão ao
   [`CHANGELOG.md`](../CHANGELOG.md), faz o commit `chore(release): vX.Y.Z`
   na `main`, cria a tag e gera os instaladores.
4. Em *Releases* aparece um **rascunho** com as notas agrupadas
   (Novidades, Correções, Documentação, Manutenção) e os instaladores.
   Revise o texto, teste um instalador e clique em **Publish release**.

A IDE consulta o último release publicado para avisar sobre atualizações; o
rascunho não aparece para os usuários até ser publicado.

Para ver a versão e as notas sem alterar nada:

```bash
scripts/release.sh --dry-run
scripts/release.sh minor --dry-run
```

Se a geração dos instaladores falhar depois que a tag foi criada (por exemplo,
uma instabilidade no runner), abra a execução do *Preparar release* e use
**Re-run failed jobs**: ela reaproveita a mesma tag e o mesmo commit. Não rode
o *Preparar release* de novo, porque ele calcularia a versão seguinte.

### Outros caminhos

- Enviar uma tag manualmente (`git tag v0.2.0 && git push origin v0.2.0`)
  também gera os instaladores, mas não atualiza versões nem o changelog.
- *Actions → Release → Run workflow* sem tag gera um pré-lançamento de teste
  `v<número>-dev`.

Nenhum secret é obrigatório. Sem certificados, os instaladores são gerados e
publicados sem assinatura; configure os secrets abaixo para assinar.

## Assinatura no Windows

Sem assinatura, o Windows SmartScreen mostra "O Windows protegeu o seu
computador" ao abrir o instalador. O workflow assina o `.exe` e o `.msi`
automaticamente quando encontra **uma** das duas configurações abaixo nos
*secrets* do repositório (*Settings → Secrets and variables → Actions*). Depois
de gerar, ele confere com `Get-AuthenticodeSignature` e falha se a assinatura
não for válida.

Nenhum certificado pode ser criado pelo próprio projeto: ele precisa ser
emitido no nome de quem distribui o programa.

### Opção A — Azure Trusted Signing (recomendada)

Serviço da Microsoft de assinatura na nuvem; não há arquivo de certificado
para guardar e o custo é mensal e baixo. Exige validação de identidade da
organização ou pessoa.

1. No portal do Azure, crie uma conta **Trusted Signing**, um **perfil de
   certificado** (Public Trust) e conclua a validação de identidade.
2. Crie um *App registration* (service principal) e dê a ele o papel
   **Trusted Signing Certificate Profile Signer** na conta.
3. Cadastre os secrets:

| Secret | Valor |
|---|---|
| `AZURE_CLIENT_ID` | id do aplicativo (service principal) |
| `AZURE_CLIENT_SECRET` | segredo do aplicativo |
| `AZURE_TENANT_ID` | id do tenant |
| `AZURE_SIGNING_ENDPOINT` | ex.: `https://eus.codesigning.azure.net` |
| `AZURE_SIGNING_ACCOUNT` | nome da conta Trusted Signing |
| `AZURE_CERTIFICATE_PROFILE` | nome do perfil de certificado |

### Opção B — certificado de assinatura de código (.pfx)

Certificado OV ou EV comprado de uma autoridade (DigiCert, Sectigo, SSL.com
etc.). Desde 2023 as autoridades só entregam certificados novos em token de
hardware ou HSM na nuvem; se o seu fornecedor permitir exportar `.pfx`:

```bash
base64 -w0 certificado.pfx   # no Windows: certutil -encode certificado.pfx saida.txt
```

| Secret | Valor |
|---|---|
| `WINDOWS_CERTIFICATE` | conteúdo do `.pfx` em base64 |
| `WINDOWS_CERTIFICATE_PASSWORD` | senha do `.pfx` |

Se o certificado estiver só num HSM na nuvem do fornecedor, use o utilitário
dele no lugar do `trusted-signing-cli`, ajustando o `signCommand` no passo
"Assinatura Windows com Azure Trusted Signing" do workflow.

### Sobre o SmartScreen

A assinatura remove o aviso de "editor desconhecido" e garante que o arquivo
não foi alterado. O SmartScreen também considera a **reputação** do
certificado: nas primeiras semanas de um certificado novo ainda pode aparecer
o aviso para alguns usuários, até acumular downloads.

## Assinatura e notarização no macOS

Sem assinatura e notarização (o padrão, quando não há secrets), o macOS bloqueia
o app na primeira abertura: use o botão direito → **Abrir**. Para distribuir sem
esse aviso é preciso uma conta no Apple Developer Program.

1. Crie um certificado **Developer ID Application** e exporte como `.p12`.
2. Gere uma *app-specific password* em appleid.apple.com.
3. Cadastre os secrets:

| Secret | Valor |
|---|---|
| `APPLE_CERTIFICATE` | `.p12` em base64 |
| `APPLE_CERTIFICATE_PASSWORD` | senha do `.p12` |
| `APPLE_SIGNING_IDENTITY` | ex.: `Developer ID Application: Nome (TEAMID)` |
| `APPLE_ID` | e-mail da conta Apple |
| `APPLE_PASSWORD` | a app-specific password |
| `APPLE_TEAM_ID` | id do time |

O `tauri-action` assina e envia para notarização automaticamente quando esses
secrets existem.

## Linux

Pacotes `.deb`/`.rpm`/`.AppImage` não exigem assinatura para instalar. O
checksum de cada arquivo aparece na página do release.

## Toolchain Rust e arquivos suportados

Programas **Ferro (`.fer`)** e exercícios **`.licao`** executam e
depuram diretamente na máquina virtual embutida na IDE, sem precisar instalar
nada além do instalador.

Os instaladores não incluem nem instalam `rustc`. A toolchain Rust
([rustup.rs](https://rustup.rs)) só é necessária para executar arquivos `.rs`,
projetos Cargo, a verificação real com `rustc` no Laboratório de Ownership e a
opção **Executar como Rust**.

Arquivos legados `.por` e `.pex` não são suportados nesta versão (ADR 0016).
