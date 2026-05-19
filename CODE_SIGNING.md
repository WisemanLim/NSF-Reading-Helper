# Code Signing Playbook

> ⚠️ **현재 GitHub Actions 자동 빌드는 [`release.yml`](.github/workflows/release.yml) 내에서 주석 처리되어 비활성** 상태입니다. 본 가이드는 활성화 시점에 필요한 시크릿/절차를 기록하며, 활성화하려면 워크플로 파일의 YAML 주석을 해제하세요.
>
> NSF Reading Helper 의 데스크탑 빌드에 **Apple Developer ID** (macOS) 와 **Windows Authenticode** (Windows) 서명을 적용하고, 추가로 **Tauri Updater** 서명 키를 설정하는 단계별 가이드. GitHub Actions 릴리스 워크플로 [`release.yml`](.github/workflows/release.yml) 의 시크릿 입력과 1:1 대응됩니다.
>
> **세 종류의 서명을 혼동하지 말 것:**
>
> 1. **플랫폼 서명** — OS 설치 경고 우회용. macOS는 Apple Developer ID, Windows는 Authenticode.
> 2. **공증 (Notarization)** — macOS Gatekeeper 신뢰. Apple 서버에 빌드 업로드 → 스캔 후 티켓 부착.
> 3. **Updater 서명** — Tauri 자체 키. 자동 업데이트 시 다운로드 산출물 무결성 검증.

---

## 0. 한눈에 보기 (필요 자산 매트릭스)

| 자산 | 용도 | 발급처 / 도구 | 비용 | 만료 |
| :-- | :-- | :-- | :-- | :-- |
| Apple Developer Program 멤버십 | macOS 서명 + 앱스토어 자격 | [developer.apple.com](https://developer.apple.com/programs/) | $99/년 | 1년 |
| **Developer ID Application** 인증서 | macOS 외부 배포 서명 | Apple Developer 포털 | 멤버십 포함 | 5년 |
| **App-specific password** | 공증 도구 인증 | [appleid.apple.com](https://appleid.apple.com) | 무료 | 만료 없음 |
| Authenticode 코드사인 인증서 (OV 또는 EV) | Windows 서명 | DigiCert / Sectigo / SSL.com / GlobalSign | $200~$500/년 (OV), $400~$800/년 (EV) | 1-3년 |
| (옵션) Azure Key Vault | EV 키 HSM 보관 | Microsoft Azure | 사용량 과금 | — |
| Tauri Updater 키페어 | 자동 업데이트 무결성 | `pnpm tauri signer generate` | 무료 | 없음 |

---

## 1. Tauri Updater 키페어 (먼저 생성)

> 인증서 발급을 기다리는 동안에도 이 키만 있으면 **서명된 업데이트** 발행은 가능합니다.

### 1.1 키 생성

```bash
# 비밀번호 입력 → 비밀키 + 공개키 생성
pnpm tauri signer generate -w ~/.tauri/nsf-reading-helper.key
```

생성된 파일:
- `~/.tauri/nsf-reading-helper.key` — 비밀키 (절대 공개 금지, 백업 필수)
- `~/.tauri/nsf-reading-helper.key.pub` — 공개키 (`tauri.conf.json` 에 넣을 base64 문자열 출력됨)

### 1.2 `tauri.conf.json` 에 공개키 입력

```json
"plugins": {
  "updater": {
    "active": true,
    "dialog": true,
    "endpoints": [
      "https://releases.nsfreading.example.com/{{target}}/{{current_version}}"
    ],
    "pubkey": "여기에-출력된-base64-공개키-붙여넣기"
  }
}
```

### 1.3 GitHub Actions 시크릿 등록

저장소 Settings → Secrets and variables → Actions:

| 시크릿 | 값 |
| :-- | :-- |
| `TAURI_SIGNING_PRIVATE_KEY` | 비밀키 파일 내용 전체 (base64 그대로) |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | 1.1 에서 입력한 비밀번호 |

빌드 시 `tauri-action` 이 자동으로 산출물에 `.sig` 서명 파일을 함께 생성합니다.

---

## 2. macOS — Apple Developer ID

### 2.1 Apple Developer Program 가입

1. [developer.apple.com/programs](https://developer.apple.com/programs/) → Enroll
2. 개인 또는 조직(D-U-N-S 번호 필요)으로 가입, $99 결제
3. 1~3일 내 승인

### 2.2 인증서 발급

**방법 A — Xcode 자동 (권장):**
```bash
# Xcode → Settings → Accounts → Apple ID 추가 → Manage Certificates → +
# "Developer ID Application" 선택
```
키체인에 자동 설치됨.

**방법 B — 수동 (Apple Developer 포털):**
1. https://developer.apple.com/account/resources/certificates → +
2. "Developer ID Application" 선택
3. CSR 업로드 (Keychain Access → Certificate Assistant → Request a Certificate)
4. 발급된 `.cer` 다운로드 → 더블클릭 → 키체인 등록

### 2.3 인증서 export → CI

```bash
# 키체인에서 인증서+개인키를 .p12 로 export
# Keychain Access → "Developer ID Application: ..." 우클릭 → Export
# 형식: Personal Information Exchange (.p12)
# 비밀번호 설정

# CI 업로드용 base64 변환
base64 -i nsf-developer-id.p12 -o nsf-developer-id.p12.base64
cat nsf-developer-id.p12.base64 | pbcopy  # 클립보드 복사
```

서명 ID 확인:
```bash
security find-identity -v -p codesigning
# 출력 예: "Developer ID Application: Your Name (TEAMID123)"
```

### 2.4 공증용 app-specific password

1. [appleid.apple.com/account/manage](https://appleid.apple.com/account/manage) → 로그인
2. **App-Specific Passwords** → **Generate** → 이름: `nsf-notarize`
3. 표시된 `xxxx-xxxx-xxxx-xxxx` 비밀번호 복사

### 2.5 Team ID 확인

[developer.apple.com/account](https://developer.apple.com/account) → Membership 페이지의 **Team ID** (10자리, 예: `ABCDEF1234`)

### 2.6 GitHub Actions 시크릿 (macOS)

| 시크릿 | 값 |
| :-- | :-- |
| `APPLE_CERTIFICATE` | 2.3 의 base64 문자열 |
| `APPLE_CERTIFICATE_PASSWORD` | `.p12` export 비밀번호 |
| `APPLE_SIGNING_IDENTITY` | `Developer ID Application: Your Name (TEAMID123)` |
| `APPLE_ID` | Apple Developer 계정 이메일 |
| `APPLE_PASSWORD` | 2.4 의 app-specific password |
| `APPLE_TEAM_ID` | 2.5 의 Team ID |

### 2.7 로컬 빌드 (개발자 머신에서)

```bash
# .env.local (gitignore'd)
export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAMID123)"
export APPLE_ID="you@example.com"
export APPLE_PASSWORD="xxxx-xxxx-xxxx-xxxx"
export APPLE_TEAM_ID="TEAMID123"

pnpm tauri:build:mac
# 산출물 .dmg / .app 자동 서명 + 공증 + 티켓 부착 (staple)
```

### 2.8 검증

```bash
# 서명 확인
codesign -dv --verbose=4 "/Applications/NSF Reading Helper.app"

# 공증 티켓 확인
xcrun stapler validate "/Applications/NSF Reading Helper.app"
# → "The validate action worked!"

# Gatekeeper 평가
spctl -a -t exec -vv "/Applications/NSF Reading Helper.app"
# → "source=Notarized Developer ID"
```

---

## 3. Windows — Authenticode

### 3.1 인증서 종류 선택

| 종류 | SmartScreen 신뢰 | 가격 | 비고 |
| :-- | :-- | :-- | :-- |
| **OV (Organization Validation)** | 천천히 누적 (다운로드 수에 비례) | $200~$500/년 | 일반 코드 서명 |
| **EV (Extended Validation)** | **즉시 신뢰** | $400~$800/년 | HSM/USB 토큰 필수 |

**EV 권장** — SmartScreen "Windows에서 PC를 보호했습니다" 경고가 첫 다운로드부터 사라집니다.

### 3.2 발급처

- [DigiCert](https://www.digicert.com/signing/code-signing-certificates) — 가장 보편적
- [Sectigo](https://www.sectigo.com/code-signing-certificates)
- [SSL.com](https://www.ssl.com/certificates/ev-code-signing/)
- [GlobalSign](https://www.globalsign.com/en/code-signing-certificate)

조직 인증서일 경우 D-U-N-S 번호 / 사업자등록증 필요. 발급까지 1-7영업일.

### 3.3 EV 인증서 — Azure Key Vault 경로 (권장)

EV 인증서는 HSM 격리가 필수입니다. Azure Key Vault HSM 에 저장하면 CI 에서도 안전하게 서명 가능.

```bash
# 1. Azure 계정 생성, Key Vault Premium SKU 만들기
# 2. CSR 생성 (Key Vault 내부에서) → 인증기관에 제출 → 발급된 cert 를 Key Vault 에 import
# 3. Service Principal 생성 (CI 가 Key Vault 접근용)
az ad sp create-for-rbac --name "nsf-codesign-ci" --role "Key Vault Crypto User"
# → appId, password, tenant 메모
```

### 3.4 OV 인증서 — `.pfx` 파일 경로 (간단)

발급사가 제공하는 `.pfx` (PKCS#12) 파일 사용:

```powershell
# CI 업로드용 base64
[Convert]::ToBase64String([IO.File]::ReadAllBytes("nsf-codesign.pfx")) | Set-Clipboard
```

### 3.5 GitHub Actions 시크릿 (Windows OV)

| 시크릿 | 값 |
| :-- | :-- |
| `WINDOWS_CERTIFICATE` | base64 인코딩된 `.pfx` |
| `WINDOWS_CERTIFICATE_PASSWORD` | `.pfx` 비밀번호 |

`tauri-action` 이 자동으로 `signtool` 호출.

### 3.6 GitHub Actions 시크릿 (Windows EV via Azure Key Vault)

`tauri.conf.json` 의 `bundle.windows.signCommand` 를 [AzureSignTool](https://github.com/vcsjones/AzureSignTool) 호출로 교체:

```json
"bundle": {
  "windows": {
    "signCommand": "AzureSignTool sign -kvu \"$env:AZURE_KEY_VAULT_URI\" -kvi \"$env:AZURE_CLIENT_ID\" -kvs \"$env:AZURE_CLIENT_SECRET\" -kvc \"$env:AZURE_CERT_NAME\" -kvt \"$env:AZURE_TENANT_ID\" -tr http://timestamp.digicert.com -td sha256 -fd sha256 \"%1\""
  }
}
```

추가 시크릿:

| 시크릿 | 값 |
| :-- | :-- |
| `AZURE_KEY_VAULT_URI` | `https://nsf-codesign.vault.azure.net/` |
| `AZURE_CLIENT_ID` | 3.3 의 appId |
| `AZURE_CLIENT_SECRET` | 3.3 의 password |
| `AZURE_TENANT_ID` | 3.3 의 tenant |
| `AZURE_CERT_NAME` | Key Vault 내 인증서 이름 |

### 3.7 검증

```powershell
# 서명 확인
signtool verify /pa /v "C:\Path\To\NSF Reading Helper_0.1.0_x64.msi"
# → "Successfully verified"

# 또는 Windows 탐색기에서 파일 우클릭 → 속성 → 디지털 서명 탭
```

---

## 4. CI 통합 — `release.yml` 시크릿 체크리스트

| 시크릿 | 사용처 | 1절 | 2절 | 3절 |
| :-- | :-- | :-: | :-: | :-: |
| `TAURI_SIGNING_PRIVATE_KEY` | 모든 OS (Updater) | ✓ | | |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | 모든 OS (Updater) | ✓ | | |
| `APPLE_CERTIFICATE` | macOS | | ✓ | |
| `APPLE_CERTIFICATE_PASSWORD` | macOS | | ✓ | |
| `APPLE_SIGNING_IDENTITY` | macOS | | ✓ | |
| `APPLE_ID` | macOS (notarize) | | ✓ | |
| `APPLE_PASSWORD` | macOS (notarize) | | ✓ | |
| `APPLE_TEAM_ID` | macOS (notarize) | | ✓ | |
| `WINDOWS_CERTIFICATE` | Windows (OV) | | | ✓ |
| `WINDOWS_CERTIFICATE_PASSWORD` | Windows (OV) | | | ✓ |
| `AZURE_*` | Windows (EV) | | | ✓ |

워크플로 실행: 태그 푸시 → 자동 빌드/서명/공증/릴리스(드래프트).

```bash
git tag v0.1.0
git push --tags
# → release.yml 트리거. 3 OS 매트릭스에서 빌드
```

---

## 5. 릴리스 manifest (`latest.json`)

Tauri Updater 는 `endpoints` 의 URL 에서 다음 형식의 JSON 을 받습니다:

```json
{
  "version": "0.1.1",
  "notes": "Bug fixes + 200 new symbols",
  "pub_date": "2026-05-19T00:00:00Z",
  "platforms": {
    "darwin-aarch64": {
      "signature": "tauri-signed-blob-from-CI",
      "url": "https://releases.nsfreading.example.com/v0.1.1/NSF_Reading_Helper_0.1.1_aarch64.app.tar.gz"
    },
    "darwin-x86_64":  { "signature": "...", "url": "..." },
    "linux-x86_64":   { "signature": "...", "url": "..." },
    "windows-x86_64": { "signature": "...", "url": "..." }
  }
}
```

CI 가 GitHub Release 에 산출물 + `latest.json` 을 업로드한 후, GitHub Pages 또는 정적 호스팅으로 `endpoints` URL 을 노출하세요. 또는 GitHub Release 자체 URL 사용:

```
https://github.com/<org>/NSF-Reading-Helper/releases/latest/download/latest.json
```

`tauri-action` 의 `updaterJsonPreferNsis: true` 옵션으로 자동 생성됩니다.

---

## 6. 키 백업 및 회전

### 6.1 백업

```bash
# Tauri Updater 키 — 1Password / Bitwarden / 하드웨어 토큰
cat ~/.tauri/nsf-reading-helper.key   # 한 번 복사해서 안전한 곳에 보관

# Apple .p12 — 키체인 동기화 또는 외장 암호화 저장소
# Windows .pfx — Azure Key Vault 또는 USB 토큰
```

> **분실 = 자동 업데이트 불가**. 새 키로 교체 후 기존 사용자는 수동 재설치 필요.

### 6.2 키 회전 절차

1. 새 키페어 발급
2. **두 공개키 모두 신뢰**하도록 잠시 `tauri.conf.json` 유지 (분기 기간)
3. 신규 업데이트는 새 키로 서명, 모든 사용자가 신규 버전 받을 때까지 기다림
4. 구 공개키 제거, 다음 릴리스부터 단일 키 운영

---

## 7. 비용 요약 (1년 기준 예시)

| 항목 | 비용 |
| :-- | :-- |
| Apple Developer Program | $99 |
| Windows OV 코드사인 (Sectigo) | ~$200 |
| Windows EV 코드사인 (Sectigo) | ~$400 |
| Azure Key Vault Premium (옵션) | ~$5/월 + 호출당 |
| **합계 (OV 경로)** | **~$300/년** |
| **합계 (EV 경로)** | **~$500/년 + Azure** |

---

## 8. 문제 해결

| 증상 | 원인 / 해결 |
| :-- | :-- |
| macOS 빌드가 `errSecInternalComponent` | `.p12` 비밀번호 또는 키체인 잠금 — `security unlock-keychain` 또는 `KEYCHAIN_PASSWORD` 시크릿 추가 |
| macOS 공증 응답 `Invalid` | 코드 서명 미완료 또는 hardened runtime 미설정 — `tauri.conf.json` 의 `bundle.macOS.hardenedRuntime: true` 확인 |
| Windows `signtool.exe : SignerSign() failed` | 인증서 만료 / 비밀번호 오류 / 타임스탬프 서버 차단 — `-tr http://timestamp.digicert.com` 변경 시도 |
| Tauri Updater `signature verification failed` | `tauri.conf.json` 의 `pubkey` 가 빌드 키와 불일치 — 재발급 후 다시 빌드 |
| EV 인증서가 CI 에서 안 됨 | EV 는 HSM 필수 — `.pfx` 가 아닌 Azure Key Vault + AzureSignTool 경로 사용 |
| 사용자가 여전히 "확인되지 않은 개발자" | macOS 공증 staple 누락 또는 캐시. `xattr -cr` 우회만 가능 |

---

## 9. 참고

- Apple Notarization: <https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution>
- Tauri Updater: <https://tauri.app/plugin/updater/>
- Tauri Signing: <https://tauri.app/distribute/sign/>
- AzureSignTool: <https://github.com/vcsjones/AzureSignTool>
- SmartScreen 기준: <https://learn.microsoft.com/en-us/windows/security/threat-protection/windows-defender-smartscreen/microsoft-defender-smartscreen-overview>
