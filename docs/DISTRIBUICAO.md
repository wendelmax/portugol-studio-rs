# Distribuição e assinatura dos instaladores

Os instaladores são gerados pelo workflow
[`.github/workflows/release.yml`](../.github/workflows/release.yml) com o
[`tauri-action`](https://github.com/tauri-apps/tauri-action):

| Plataforma | Arquivos |
|---|---|
| Windows x64 | `.exe` (NSIS, instala para o usuário, sem pedir administrador) e `.msi` |
| Linux x64 | `.deb`, `.rpm` e `.AppImage` |
| macOS (Apple Silicon e Intel) | `.dmg` e `.app` |

## Como publicar uma versão

1. Atualize a versão em `Cargo.toml` (`[workspace.package]`),
   `apps/desktop/package.json` e `apps/desktop/src-tauri/tauri.conf.json`.
2. Crie e envie a tag:

   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```

3. O workflow cria um **release em rascunho** com os instaladores. Revise e
   publique em *Releases* no GitHub.

Também dá para rodar o workflow manualmente (*Actions → Release → Run
workflow*); nesse caso o release sai como pré-lançamento `v<número>-dev`.

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

Sem assinatura e notarização, o macOS bloqueia o app ("não pode ser aberto
porque o desenvolvedor não pode ser verificado"). Requer uma conta no Apple
Developer Program.

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
