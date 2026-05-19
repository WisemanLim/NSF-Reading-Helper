# NSF Reading Helper — 설치 및 사용 가이드 (멀티플랫폼)

**플랫폼:** macOS 11+ (ARM / Intel) · Windows 10/11 · Linux (Ubuntu 20.04+, Fedora, Arch)
**웹:** Chromium 110+ / Firefox 110+ / Safari 16+ — `docker compose` 한 줄로 배포

| 플랫폼 | 빌드 파일 (v0.1.0) |
| :-- | :-- |
| macOS Apple Silicon | `NSF Reading Helper_0.1.0_aarch64.dmg` |
| macOS Intel | `NSF Reading Helper_0.1.0_x64.dmg` |
| macOS Universal | `NSF Reading Helper_0.1.0_universal.dmg` |
| Windows (x64) | `NSF Reading Helper_0.1.0_x64_en-US.msi` (또는 `.exe` NSIS) |
| Linux (Debian/Ubuntu) | `nsf-reading-helper_0.1.0_amd64.deb` / `_arm64.deb` |
| Linux (Fedora/RHEL) | `nsf-reading-helper-0.1.0-1.x86_64.rpm` / `_aarch64.rpm` |
| Linux (배포판 무관) | `nsf-reading-helper_0.1.0_amd64.AppImage` |
| Web (Docker) | `docker compose -f docker-compose.local.yml up -d` → http://localhost:8000 |

---

## 1. 사전 준비

### 1.1 시스템 요구사항

| 항목 | 최소 | 권장 |
| :-- | :-- | :-- |
| OS | macOS 11 / Windows 10 / Ubuntu 20.04 | 최신 |
| CPU | x86_64 또는 arm64 | Apple Silicon · 최신 Intel · ARM64 |
| RAM | 1 GB | 4 GB 이상 |
| 디스크 | 80 MB (설치) + 작은 SQLite 사전 (~1 MB) | 500 MB 이상 |
| 네트워크 | MathJax CDN 최초 로드 시에만 필요 | — |

> **AS-It-Sounds 와 달리 Whisper 모델/ffmpeg 의존성이 없습니다.** 본 앱은 수식만 처리하므로 리소스 요구가 매우 가볍습니다.

### 1.2 (옵션) TTS 패키지

데스크탑은 OS 기본 TTS 를 사용합니다 — 별도 설치 불필요한 경우가 대부분.

| OS | 명령 | 필요 시 추가 설치 |
| :-- | :-- | :-- |
| macOS | `say` | 기본 포함 (Voice & Spoken Content 의 한국어 음성 다운로드 권장) |
| Windows | SAPI / PowerShell `System.Speech.Synthesis` | 기본 포함 |
| Linux | `espeak-ng` 또는 `speech-dispatcher` | `sudo apt install espeak-ng` |

웹/브라우저는 `window.speechSynthesis` 를 사용합니다 (별도 설치 없음).

### 1.3 (옵션) OCR 사이드카

수식 이미지를 LaTeX 로 변환하는 [`pix2tex`](https://github.com/lukas-blecher/LaTeX-OCR) 기반 사이드카는 **선택 사항**. Docker 가 설치되어 있어야 합니다 (Docker Desktop, OrbStack 또는 podman). 자세한 내용은 [5절](#5-ocr-사이드카-옵션).

---

## 2. 설치

### 2.1 macOS (.dmg)

**Apple Silicon (M1/M2/M3/M4):** `NSF Reading Helper_0.1.0_aarch64.dmg`
**Intel:** `NSF Reading Helper_0.1.0_x64.dmg`
**둘 다 지원하는 단일 빌드:** `..._universal.dmg`

1. DMG 더블클릭 → 마운트
2. **NSF Reading Helper.app** 아이콘을 **Applications** 폴더로 드래그
3. DMG 창 닫기 → 디스크 추출

**첫 실행 — Gatekeeper 우회 (미서명 빌드):**

방법 A — 권한 허용 (권장)
1. Applications 폴더에서 `NSF Reading Helper.app` **우클릭** → **열기**
2. 경고창 다시 **열기** 클릭

방법 B — 격리 속성 제거 (터미널)
```bash
xattr -cr "/Applications/NSF Reading Helper.app"
```

> **"손상되었거나 신뢰할 수 없는 개발자"** 메시지는 동일한 방법으로 해결. Apple Developer ID 서명 부재일 뿐이며 코드는 동일합니다.

### 2.2 Windows (.msi / .exe)

1. `NSF Reading Helper_0.1.0_x64_en-US.msi` 더블클릭
2. **"Windows에서 PC를 보호했습니다"** SmartScreen → **추가 정보** → **실행**
3. 설치 마법사 진행 (기본 경로: `C:\Program Files\NSF Reading Helper\`)
4. 시작 메뉴 → "NSF Reading Helper" 실행

> SmartScreen 경고는 Authenticode 서명 부재 시 발생.

### 2.3 Linux

**Debian / Ubuntu (.deb)**
```bash
sudo dpkg -i nsf-reading-helper_0.1.0_amd64.deb
sudo apt --fix-broken install     # 의존성 누락 시
```
실행:
```bash
nsf-reading-helper
```

**Fedora / RHEL (.rpm)**
```bash
sudo dnf install ./nsf-reading-helper-0.1.0-1.x86_64.rpm
```

**AppImage (배포판 무관)**
```bash
chmod +x NSF\ Reading\ Helper_0.1.0_amd64.AppImage
./NSF\ Reading\ Helper_0.1.0_amd64.AppImage
```
> Ubuntu 22.04+ 는 `sudo apt install libfuse2`.

**런타임 의존성 (Linux):**
- `libwebkit2gtk-4.1-0` — WebView
- `libappindicator3-1` — 시스템 트레이
- `librsvg2-2` — SVG 렌더링
- (옵션) `espeak-ng` — TTS

Ubuntu 누락 시 한 번에:
```bash
sudo apt install -y libwebkit2gtk-4.1-0 libappindicator3-1 librsvg2-2 espeak-ng
```

### 2.4 웹 (Docker)

도메인이 있는 서버:
```bash
git clone https://github.com/<your-org>/NSF-Reading-Helper.git && cd NSF-Reading-Helper
cp .env.example .env
$EDITOR .env   # DOMAIN, LETSENCRYPT_EMAIL 수정
docker compose up -d --build
```
→ `https://<DOMAIN>` (Let's Encrypt 자동)

로컬/사내망 (TLS 없음):
```bash
docker compose -f docker-compose.local.yml up -d --build
```
→ http://localhost:8000

상세: [README-Web.md](../README-Web.md).

---

## 3. 사용

### 3.1 첫 실행 — 시드 사전 자동 설치

처음 실행 시 SQLite 가 비어 있으면 230+ 핵심 기호가 자동 시드됩니다 (수학·물리·화학·생물·지구과학·통계·논리 등 16개 도메인). 다운로드 없이 앱 번들에 임베드되어 있어 오프라인 동작합니다.

저장 위치:

| OS | 경로 |
| :-- | :-- |
| macOS | `~/Library/Application Support/com.nsfreading.app/nsf-reading.sqlite` |
| Windows | `%APPDATA%\com.nsfreading.app\nsf-reading.sqlite` |
| Linux | `~/.local/share/com.nsfreading.app/nsf-reading.sqlite` |

### 3.2 수식 입력 (4가지 포맷)

좌측 사이드바 **분석** 페이지 → 입력란에 수식을 붙여넣거나 직접 입력:

| 포맷 | 예시 | 비고 |
| :-- | :-- | :-- |
| LaTeX | `\int_0^1 x^2 \, dx` | 가장 표준적, mhchem(`\ce{}`) 포함 |
| MathML | `<math>...</math>` | XML 직접 |
| AsciiMath | `sum_(i=1)^n i^2` | 간편 문법 |
| Unicode | `∫₀¹ x² dx`, `Σ`, `√` | 기호 직접 |

포맷은 자동 감지 (입력 옆에 감지된 포맷 표시). 하단 샘플 버튼 클릭으로 빠르게 삽입.

**단축키:** `Ctrl/⌘ + Enter` — 분석 실행. `Ctrl/⌘ + K` — 검색 페이지 이동.

### 3.3 분석 결과

분석 버튼 클릭 → 다음을 동시 표시:

1. **렌더링** — MathJax v3 (chtml). 화학식은 mhchem 지원.
2. **한국어 읽기** — 자체 템플릿 + SRE + `ko-clearspeak.json` 후처리:
   - `\int_0^1 x^2 \, dx` → "0 부터 1 까지 x 제곱 을(를) x 에 대하여 적분"
   - `\sum_{i=1}^n i^2` → "i 가 1 부터 n 까지일 때 i 제곱 의 합"
   - `\frac{a}{b}` → "b 분의 a"
3. **TTS 재생** — 좌측 ▶ 버튼. 속도 0.5×~2.0× 조절. macOS 는 `say`, Windows 는 SAPI, Linux 는 `espeak-ng`, 웹은 `speechSynthesis`.
4. **토큰 리스트** — 수식의 각 명령/기호. 클릭하거나 화살표 키로 순회.
5. **기호 패널** — 클릭한 토큰의 명칭·한글 읽기·뜻·분야별 의미·예시 수식·유사 기호.

### 3.4 검색

좌측 사이드바 **검색** → 통합 검색창. 동시 매칭:
- 한글 명칭("적분", "시그마")
- 영어 명칭("integral", "sigma")
- LaTeX 명령(`\int`, `\Sigma`)
- Unicode(`∫`, `Σ`)
- 별칭/약어/오타("인테그럴", "시그머", "\inn")

검색 엔진:
- 1단계: 정확 매치 (id/이름/LaTeX)
- 2단계: 부분 문자열 (LIKE)
- 3단계: FTS5 trigram (3자 이상 입력 시)
- 4단계: Levenshtein 편집 거리 (오타 보정, 길이의 1/3 + 1 이내)

결과 항목 클릭 → 우측에 상세 패널.

### 3.5 라이브러리

**라이브러리** 페이지 — 최근 분석 100개, 즐겨찾기한 기호 목록. 기호 패널의 ★ 버튼으로 즐겨찾기 토글.

### 3.6 설정 (좌측 사이드바 하단)

| 항목 | 설명 | 기본값 |
| :-- | :-- | :-- |
| **읽기 규칙** | Clearspeak(자연스러움) / Mathspeak(구조 명시) | Clearspeak |
| **읽기 언어** | 한국어 / English | 한국어 |
| **TTS 속도** | 0.5× ~ 2.0× | 1.0× |
| **UI 언어** | 한국어 / English | 한국어 |
| **업데이트 확인** (데스크탑) | 새 버전 확인 + 자동 다운로드/설치 | — |
| **사전 팩 가져오기/내보내기** (데스크탑) | `.nsfpack` 파일 import/export | — |

### 3.7 사전 팩 (`.nsfpack`)

`.nsfpack` 은 `manifest.json` + `symbols.json` 을 담은 zip 입니다. 사전 확장/공유에 사용:

**가져오기:**
1. 설정 → **팩 가져오기** → `.nsfpack` 파일 선택
2. 진행 결과: `[이름 v버전] 가져옴: 기호 N개 추가, M개 중복`
3. 기존 기호와 중복(같은 latex/unicode)이면 자동 건너뜀
4. 가져온 기호는 `pack:<팩이름>` 별칭으로 추적

**내보내기:**
1. 설정 → **팩 내보내기** → 저장 위치 선택
2. 현재 DB 의 모든 기호 + 도메인 + 별칭 + 예시 + 분야별 의미 export

`symbols.json` 스키마는 [`src-tauri/data/seed.json`](../src-tauri/data/seed.json) 과 동일.

### 3.8 자동 업데이트 (데스크탑)

설정 → **업데이트 확인** → 새 버전이 있으면 `v0.x.y 설치` 버튼 → 다운로드 + 검증(Tauri 서명) + 설치 + 재시작 안내.

> 서명 키는 GitHub Actions 릴리스 파이프라인이 발급 — [`release.yml`](.github/workflows/release.yml) (현재 워크플로는 주석 처리되어 비활성).

---

## 4. 한국어 읽기 (ko-clearspeak)

SRE 기본 한국어 룰셋은 제한적이라, 본 앱은 다음 3단 파이프라인으로 자연스러운 한국어 읽기를 생성합니다:

1. **템플릿 매칭** ([`src/lib/ko-clearspeak.json`](../src/lib/ko-clearspeak.json)) — 14개 패턴 (적분/합/곱/극한/분수/제곱근/이항계수 등)
2. **SRE fallback** — 템플릿 미매치 시 `speech-rule-engine` 의 `ko` locale 호출
3. **후처리** — 영문 잔재(`integral` → "적분", `over` → "분의" 등 22 규칙)와 그리스 lexicon 적용

예시:

| 입력 | 출력 |
| :-- | :-- |
| `\int_0^1 x^2 \, dx` | 0 부터 1 까지 x 제곱 을(를) x 에 대하여 적분 |
| `\sum_{i=1}^n i^2` | i 가 1 부터 n 까지일 때 i 제곱 의 합 |
| `e^{i\pi} + 1 = 0` | e 의 i 파이 승 + 1 = 0 |
| `\frac{a}{b}` | b 분의 a |
| `\sqrt{b^2 - 4ac}` | 루트 b 제곱 - 4 a c |
| `x = \frac{-b \pm \sqrt{...}}{2a}` | … (이차방정식 근의 공식 자연어) |

룰셋은 JSON 이므로 직접 편집해서 더 정밀하게 만들 수 있습니다.

---

## 5. OCR 사이드카 (옵션)

수식 이미지 → LaTeX 변환. 데스크탑/웹 모두 같은 사이드카 활용.

### 5.1 사이드카 기동 (Docker)

```bash
# 로컬 (NSF 백엔드/프론트 + OCR)
docker compose -f docker-compose.local.yml --profile ocr up -d --build

# 운영 (TLS + OCR)
docker compose --profile ocr up -d --build
```

- 첫 요청 시 pix2tex 모델 자동 다운로드 (~수백 MB) — `nsf-ocr-cache` 볼륨 캐시
- CPU 전용 기본. GPU 가속 필요 시 [`Dockerfile.ocr`](../Dockerfile.ocr) 의 base 이미지를 CUDA 이미지로 교체
- 컨테이너 헬스체크: `GET /ocr/healthz`

### 5.2 사용

분석 페이지에서 텍스트 입력란 아래의 **수식 이미지 끌어다 놓기** 영역에:
- 이미지 파일 드래그앤드롭
- 클릭 → 파일 선택
- 이미지 클립보드 → `⌘V` / `Ctrl+V` 붙여넣기

OCR 결과(LaTeX) 가 입력란에 자동 채워집니다. 사이드카가 꺼져 있으면 이 영역은 UI 에서 자동으로 사라집니다 (헬스체크 실패).

### 5.3 데스크탑에서 OCR 사용하기

데스크탑 앱이 Docker 사이드카를 호출하려면 사이드카의 OCR URL 을 빌드 시 지정:

```bash
VITE_NSF_OCR_URL=http://localhost:8090 pnpm tauri:dev
```

기본값(`/ocr`)은 웹 빌드용 — Tauri 빌드에서는 명시적 URL 권장.

---

## 6. 데이터 저장 위치

| OS | 데이터 디렉터리 |
| :-- | :-- |
| macOS | `~/Library/Application Support/com.nsfreading.app/` |
| Windows | `%APPDATA%\com.nsfreading.app\` |
| Linux | `~/.local/share/com.nsfreading.app/` |

구조:
```
com.nsfreading.app/
├── nsf-reading.sqlite      # 사전 + 히스토리 + 즐겨찾기 + 노트 + 설정
└── (필요 시 사용자 .nsfpack 가져오기 결과 모두 SQLite 안에 합쳐짐)
```

### 6.1 백업

**macOS / Linux**
```bash
cp -R ~/Library/Application\ Support/com.nsfreading.app/ ~/Backups/nsf/   # macOS
cp -R ~/.local/share/com.nsfreading.app/ ~/Backups/nsf/                    # Linux
```

**Windows (PowerShell)**
```powershell
Copy-Item -Recurse "$env:APPDATA\com.nsfreading.app" "$env:USERPROFILE\Backups\nsf"
```

### 6.2 완전 초기화

**macOS**
```bash
rm -rf ~/Library/Application\ Support/com.nsfreading.app/
```

**Linux**
```bash
rm -rf ~/.local/share/com.nsfreading.app/
```

**Windows (PowerShell)**
```powershell
Remove-Item -Recurse -Force "$env:APPDATA\com.nsfreading.app"
```

> SQLite 만 삭제하면 다음 부팅 시 기본 시드 사전이 재설치됩니다.

---

## 7. 트러블슈팅

### 7.1 공통

| 증상 | 원인 / 해결 |
| :-- | :-- |
| **수식이 렌더링 안 됨** | MathJax CDN(`cdn.jsdelivr.net`) 차단 가능. 방화벽/사내망 환경 → 자체 CDN 미러로 `src/lib/mathjax.ts` 의 `CDN` 상수 교체 후 재빌드 |
| **TTS 재생이 안 됨** | OS TTS 백엔드 확인 (macOS Voice 설치, Windows 한국어 보이스, Linux `espeak-ng` 설치) |
| **검색 결과 비어 있음** | 사전이 비었을 수 있음. 설정 → "팩 가져오기" 또는 데이터 디렉터리의 `nsf-reading.sqlite` 삭제 후 재실행 (자동 시드) |
| **한국어 읽기가 어색** | `src/lib/ko-clearspeak.json` 의 `templates` / `sre_post` 규칙 추가/수정 (재빌드 필요) |
| **OCR 박스가 안 보임** | 사이드카 미기동. `docker compose --profile ocr up -d` 확인. 헬스체크: `curl http://localhost:8000/ocr/healthz` |

### 7.2 macOS 전용

| 증상 | 해결 |
| :-- | :-- |
| **"손상되었거나 신뢰할 수 없는 개발자"** | `xattr -cr "/Applications/NSF Reading Helper.app"` |
| **앱이 즉시 종료** | `Console.app` → "NSF Reading Helper" 검색. TTS 보이스 누락이 흔한 원인 |
| **로그 실시간 확인** | `log stream --predicate 'subsystem == "com.nsfreading.app"' --info` |
| **`say` 한국어 음성 없음** | 시스템 설정 → 손쉬운 사용 → 음성 → 한국어 보이스(예: Yuna) 다운로드 |

### 7.3 Windows 전용

| 증상 | 해결 |
| :-- | :-- |
| **SmartScreen "Windows에서 PC를 보호했습니다"** | **추가 정보 > 실행** |
| **WebView2 누락 오류** | Windows 11 기본 포함. Win 10 은: `winget install Microsoft.EdgeWebView2Runtime` |
| **앱이 흰 화면 / 즉시 닫힘** | WebView2 미설치 또는 손상 → 위 명령으로 재설치 |
| **이벤트 뷰어** | `Windows Logs > Application` → 소스 `NSF Reading Helper` |
| **한국어 TTS 음성 없음** | 설정 → 시간 및 언어 → 음성 → 한국어 음성 추가 |

### 7.4 Linux 전용

| 증상 | 해결 |
| :-- | :-- |
| **`error while loading shared libraries: libwebkit2gtk-4.1.so.0`** | `sudo apt install libwebkit2gtk-4.1-0` |
| **`libappindicator3.so.1: cannot open shared object`** | `sudo apt install libappindicator3-1` |
| **AppImage `dlopen(): libfuse.so.2`** | `sudo apt install libfuse2` |
| **TTS 무음** | `sudo apt install espeak-ng`. 또는 `sudo apt install speech-dispatcher` |
| **Wayland 렌더링 깨짐** | `GDK_BACKEND=x11 nsf-reading-helper` 로 X11 모드 |
| **시스템 트레이 표시 안 됨** | GNOME: `gnome-shell-extension-appindicator` 설치 후 재로그인 |
| **로그** | `journalctl --user -f` (앱 실행 중) 또는 터미널에서 `nsf-reading-helper` 직접 실행 |

### 7.5 웹 (Docker)

| 증상 | 해결 |
| :-- | :-- |
| **`acme` 가 인증서 발급 실패** | DNS 미전파 또는 80 포트 차단. `dig +short $DOMAIN`, `curl -I http://$DOMAIN/.well-known/acme-challenge/test` |
| **`502 Bad Gateway`** | backend 컨테이너 부팅 중 또는 사망. `docker compose logs backend` |
| **로컬에서 503** | TLS/도메인 미설정. `docker compose -f docker-compose.local.yml up` 사용 |
| **검색 안 됨** | 사전이 비었을 수 있음. `docker compose exec backend ls -la /data` 로 SQLite 확인 |

---

## 8. 단축키

| 기능 | 단축키 |
| :-- | :-- |
| 검색 페이지로 이동 + 포커스 | `Ctrl + K` / `⌘ + K` |
| 분석 실행 | `Ctrl + Enter` / `⌘ + Enter` (수식 입력란 포커스 시) |
| 본문으로 건너뛰기 (스크린리더) | Tab 첫 번째 누르면 스킵 링크 노출 |
| 토큰 순회 | `←` / `→` / `↑` / `↓` |
| 토큰 처음/끝 | `Home` / `End` |
| 토큰 선택 | `Enter` / `Space` |
| TTS 재생 | 분석 후 SpeechControls 의 ▶ 버튼 (포커스 후 `Enter`) |
| 개발자 도구 (macOS) | `⌥ ⌘ I` |
| 개발자 도구 (Win/Linux) | `Ctrl + Shift + I` |

---

## 9. 자주 묻는 질문

**Q. 인터넷 없이도 동작?**
A. 분석/읽기/검색/TTS 모두 오프라인. 단, MathJax 가 CDN 에서 로드됩니다 — 처음 한 번만 캐시되고 이후 오프라인. 완전 에어갭 환경이면 MathJax 를 자체 호스팅 또는 빌드 번들에 포함하도록 [`src/lib/mathjax.ts`](../src/lib/mathjax.ts) 수정.

**Q. 사전을 더 늘리고 싶다.**
A. 두 가지 방법:
- 직접 편집: [`src-tauri/data/seed.json`](../src-tauri/data/seed.json) → 재빌드. 230 → 500+ 까지 PRD M2 로드맵.
- `.nsfpack` 만들어서 import: 같은 스키마의 JSON 을 zip 으로 묶으면 됨 (manifest.json + symbols.json).

**Q. 화학식만 잘 안 나옴?**
A. MathJax `mhchem` 패키지로 `\ce{H2 + O2 -> H2O}` 형태 지원. 일반 LaTeX 인라인은 `\rightarrow`, `(s)`, `(aq)` 등 사전에 등록되어 있음.

**Q. 손글씨로 그린 기호도 찾을 수 있나?**
A. v0.1 미지원. Detexify 통합은 V2 로드맵.

**Q. 자동 업데이트 어떻게 동작?**
A. Tauri Updater 가 빌드 시 임베드된 endpoint URL 에서 manifest 를 조회 → 새 버전 있으면 서명 검증 + 다운로드 + 설치. GitHub Release 의 `latest.json` 을 endpoint 로 권장 — [`tauri.conf.json`](../src-tauri/tauri.conf.json) `plugins.updater` 참조.

**Q. SRE 한국어 결과가 어색한데 직접 고칠 수 있나?**
A. 네. [`src/lib/ko-clearspeak.json`](../src/lib/ko-clearspeak.json) 의 `templates` (정규식 → 한국어 문장), `sre_post` (영문→한국어 치환), `lexicon` (그리스 문자 발음) 세 곳을 편집하면 됩니다. 재빌드 후 즉시 반영.

**Q. 웹과 데스크탑 데이터가 공유되나?**
A. V1 에서는 독립. 각자의 SQLite 가 있습니다. V2 에서 클라우드 동기화 검토.

**Q. 변환 결과 외부 도구로 확인?**
A. REST 모드(`docker compose -f docker-compose.local.yml up`) 에서 `curl`:
```bash
curl -X POST http://localhost:8000/api/expressions/analyze \
  -H 'content-type: application/json' \
  -d '{"rawInput":"\\int_0^1 x^2 dx","format":"latex","locale":"ko"}'
```

**Q. 모든 플랫폼을 한 번에 빌드?**
A. **각 플랫폼은 네이티브 호스트에서만 안정 빌드 가능** — `rusqlite` / `ring` 의 네이티브 의존성 때문. GitHub Actions CI 권장 (현재 자동화는 비활성):

```bash
# 사용 전: .github/workflows/release.yml 의 YAML 주석을 해제하세요.
git tag v0.1.0 && git push --tags
```
[`release.yml`](.github/workflows/release.yml) 이 macOS/Win/Linux 러너에서 동시 빌드 후 자동 릴리스 (draft). **현재 트리거는 주석 처리되어 비활성 상태**.

대안: VM(Parallels/UTM/VMware) 에서 직접 빌드.

---

## 10. 라이선스 및 크레딧

- **NSF Reading Helper** — MIT
- **MathJax** — Apache-2.0
- **Speech Rule Engine** — Apache-2.0
- **Tauri** — MIT / Apache-2.0
- **axum, tower-http** — MIT
- **rusqlite (SQLite)** — Public Domain
- **pix2tex (LaTeX-OCR)** — MIT (옵션 사이드카)
- **unicode-math 시드 (참고)** — LPPL

문의/이슈: GitHub Issues 또는 [README.md](../README.md) 참조.
