# NSF Reading Helper

> **자연과학 공식 읽기 도우미.** 수학·물리·화학·지구과학·생물의 수식과 기호를 **렌더링하고**, **한국어로 읽어주며**, **기호 하나하나의 뜻과 분야별 의미를 풀어주는** 교육형 도구. 데스크탑(Tauri 2)과 웹(axum + nginx) 양쪽 모두 동일한 React 코드베이스로 동작합니다.

상세 명세는 [`.doc/PRD.md`](.doc/PRD.md), 웹 배포는 [`README-Web.md`](README-Web.md) 참조.

---

## 핵심 특징

- **수식 입력 4종**: LaTeX · MathML · AsciiMath · Unicode 기호 직접
- **MathJax v3 + mhchem** 렌더링 (화학식 `\ce{...}` 지원)
- **Speech Rule Engine (SRE)** 기반 한국어 자연어 읽기
- **기호 사전** — 60+ 핵심 기호 시드, 분야별(수학/물리/화학/생물/지구과학/통계/논리) 의미 분기
- **퍼지 검색** — 한글·영문·LaTeX·Unicode·별칭 통합
- **데스크탑**: Tauri 2 + Rust + SQLite (완전 오프라인)
- **웹**: 동일 코드 + axum 백엔드 + nginx + Let's Encrypt 자동 TLS
- **접근성**: `role="math"`, `aria-label` SRE 읽기, 다크 모드

---

## 디렉토리 구조

```
NSF-Reading-Helper/
├── .doc/                        # PRD 및 참고 문서
├── src/                         # React 19 + TS (데스크탑/웹 공유)
│   ├── App.tsx
│   ├── pages/                   # Home/Analyze/Search/Library/Settings
│   ├── components/              # FormulaEditor, RenderView, SymbolPanel ...
│   ├── ipc/                     # client.ts (스위처) + tauriClient + webClient
│   ├── store/useAppStore.ts     # zustand
│   ├── lib/                     # mathjax · sre · tts · env · cn · detectFormat
│   └── types.ts
├── src-tauri/                   # 데스크탑 (Rust + Tauri 2)
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/default.json
│   └── src/
│       ├── main.rs, lib.rs
│       ├── commands.rs          # #[tauri::command] 핸들러
│       └── core/                # db · analyzer · seed · models · settings · tts · error
├── web-server/                  # 웹 백엔드 (axum, core 재사용)
│   ├── Cargo.toml
│   └── src/{main.rs, core.rs}   # core.rs 는 #[path] 로 src-tauri/src/core/* 포함
├── deploy/nginx.app.conf        # 프론트 컨테이너 nginx 설정
├── Dockerfile.web               # 백엔드 이미지
├── Dockerfile.web-frontend      # 프론트 이미지
├── docker-compose.yml           # nginx-proxy + acme + backend + frontend
├── .env.example
├── README.md
└── README-Web.md
```

---

## 사전 요구

| 항목 | 데스크탑 빌드 | 웹 (Docker) |
| :-- | :-- | :-- |
| Node | 20+, pnpm 10+ | (도커 안에서) |
| Rust | rustup, cargo | (도커 안에서) |
| OS 도구 | macOS: `brew install cmake`, Linux: `webkit2gtk-4.1-dev` | Docker 24+ |

```bash
# macOS
brew install node pnpm cmake
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Ubuntu
sudo apt install -y curl build-essential pkg-config cmake \
  libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
curl -fsSL https://get.pnpm.io/install.sh | sh -
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

---

## 빠른 시작

### 데스크탑 (개발)

```bash
pnpm install

# 최초 1회 — Tauri 아이콘 생성 (src-tauri/icons/*)
# 1024×1024 PNG 1장을 준비해서 (없으면 placeholder 자동 생성):
python3 -c "from PIL import Image,ImageDraw,ImageFont;
img=Image.new('RGBA',(1024,1024),(11,13,16,255));
d=ImageDraw.Draw(img); d.ellipse([128,128,896,896],fill=(122,198,255,255));
f=ImageFont.truetype('/System/Library/Fonts/Supplemental/Times New Roman.ttf',720);
d.text((420,80),'∫',fill=(11,13,16,255),font=f); img.save('icons-source.png')"
pnpm tauri icon icons-source.png -o src-tauri/icons && rm icons-source.png

pnpm tauri:dev
```

첫 실행 시 `nsf-reading.sqlite` 가 생성되고 시드 사전이 자동 설치됩니다.

데이터 저장 위치:

| OS | 경로 |
| :-- | :-- |
| macOS | `~/Library/Application Support/com.nsfreading.app/` |
| Windows | `%APPDATA%\com.nsfreading.app\` |
| Linux | `~/.local/share/com.nsfreading.app/` |

### 웹 (Docker, TLS 자동)

```bash
cp .env.example .env
$EDITOR .env                       # DOMAIN, LETSENCRYPT_EMAIL 수정
docker compose up -d --build
```

상세는 [`README-Web.md`](README-Web.md).

### 웹 (로컬 개발, 컨테이너 없이)

```bash
# 백엔드 (port 8080)
cd web-server
NSF_DATA_DIR=./.dev-data cargo run --release

# 프론트엔드 (port 1420)
VITE_NSF_API_BASE=http://localhost:8080 pnpm dev
```

---

## 데스크탑 빌드

```bash
# 현재 플랫폼
pnpm tauri:build

# macOS Universal
pnpm tauri:build:mac

# Windows / Linux
pnpm tauri:build:win
pnpm tauri:build:linux
pnpm tauri:build:linux:arm
```

크로스컴파일은 `rusqlite`·`ring` 의 네이티브 의존성 때문에 일반적으로 같은 OS 호스트에서만 가능합니다. 멀티 OS 빌드는 GitHub Actions 매트릭스 권장.

---

## 아키텍처 한눈에

```
┌──────────────────────────────────────────────────────────────┐
│              React 19 + TypeScript + Vite + Zustand          │
│                    (src/ — 데스크탑/웹 100% 공유)              │
├──────────────────────────────────────────────────────────────┤
│  Pages: Home · Analyze · Search · Library · Settings         │
│  Engines: MathJax v3 (CDN) + SRE (npm)                       │
├──────────────────────────┬───────────────────────────────────┤
│  ipc/client.ts           │  isTauri ? tauriIpc : webIpc      │
└────────────┬─────────────┴───────────────────────┬───────────┘
             │                                     │
             ▼ invoke()                            ▼ fetch()
┌────────────────────────────┐    ┌──────────────────────────────┐
│   src-tauri/ (Tauri 2)     │    │   web-server/ (axum)         │
│   commands.rs              │    │   main.rs (REST endpoints)   │
│   ↓ 동일 모듈 ↓             │    │   ↓ #[path] 재포함 ↓          │
│   core/ ◀──────────────────┴────┤   core.rs → src-tauri/core/* │
│   db · analyzer · seed · models │   db · analyzer · seed ...   │
│   settings · tts · error        │   settings · error           │
└────────────────────────────┘    └──────────────────────────────┘
           │                                    │
           ▼                                    ▼
       SQLite (local)                   SQLite (Docker volume)
```

핵심 — 한 줄도 코드를 옮기지 않고 백엔드 코어를 데스크탑·웹이 공유합니다. `web-server/src/core.rs`:

```rust
#[path = "../../src-tauri/src/core/error.rs"]    pub mod error;
#[path = "../../src-tauri/src/core/models.rs"]   pub mod models;
#[path = "../../src-tauri/src/core/seed.rs"]     pub mod seed;
#[path = "../../src-tauri/src/core/db.rs"]       pub mod db;
#[path = "../../src-tauri/src/core/analyzer.rs"] pub mod analyzer;
#[path = "../../src-tauri/src/core/settings.rs"] pub mod settings;
```

---

## 시드 사전

`src-tauri/src/core/seed.rs` 에 약 60개 핵심 기호가 임베드됩니다.

분야별 도메인 9개: `math-algebra`, `math-calculus`, `math-set-logic`, `math-geometry`, `physics-general`, `chemistry`, `biology`, `earth-science`, `statistics`, `logic`.

커버 범위:
- 그리스 문자 대/소 (α β γ δ ε θ λ μ π ρ σ τ φ ω, Σ Π Δ Φ Ω)
- 미적분 (∫ ∮ ∂ ∇ ∞ √)
- 관계 (≤ ≥ ≠ ≈ ≡)
- 집합·논리 (∈ ∉ ⊂ ⊆ ∪ ∩ ∀ ∃ ¬)
- 화살표 (→ ⇌)
- 연산자 (± × ÷ ·)
- 함수·상수 (sin cos tan log ln lim e ℏ)
- 화학 (`\ce{...}`)
- 단위 (°)

V1 목표(500+) 까지의 확장은 [`PRD.md` §9.4](.doc/PRD.md) 참조.

---

## IPC 계약 (양 모드 동일)

| 호출 | 데스크탑 (Tauri command) | 웹 (REST) |
| :-- | :-- | :-- |
| `ipc.analyze(input)` | `invoke("analyze")` | `POST /api/expressions/analyze` |
| `ipc.symbolSearch(q, opts)` | `invoke("symbol_search")` | `GET /api/symbols/search` |
| `ipc.symbolGet(id)` | `invoke("symbol_get")` | `GET /api/symbols/:id` |
| `ipc.symbolExamples(id)` | `invoke("symbol_examples")` | `GET /api/symbols/:id/examples` |
| `ipc.historyList()` | `invoke("history_list")` | `GET /api/history` |
| `ipc.favoritesAdd/Remove/List` | `invoke("favorites_*")` | `POST/DELETE/GET /api/favorites` |
| `ipc.notesUpsert / List` | `invoke("notes_*")` | `POST/GET /api/notes` |
| `ipc.settingsGet/Set` | `invoke("settings_*")` | `GET/PUT /api/settings` |
| `ipc.ttsSpeak/Stop` | OS TTS bridge (say/espeak/SAPI) | `window.speechSynthesis` (lib/tts.ts) |

---

## 개발 스크립트

```bash
pnpm dev               # Vite dev server (1420)
pnpm tauri:dev         # Tauri + Vite (데스크탑)
pnpm build             # Vite production build → dist/
pnpm typecheck         # tsc --noEmit
pnpm tauri:build       # 현재 OS용 인스톨러
```

---

## 로드맵

V1 (MVP)는 본 저장소가 1차 셋업한 상태입니다 — PRD `M0` + 일부 `M1`/`M2`.

다음 단계:
- M1 완료: SRE 한국어 룰셋 (`ko-clearspeak.json`) 정밀화
- M2: 시드 500+ 확장 (전문가 큐레이션)
- M3: 검색 퍼지 (Levenshtein/FTS5), A11y 패스
- M4: 코드 서명, 자동 업데이트, `.nsfpack`
- M5: 웹 분석 모니터링 (Sentry/Plausible)
- V1.5: OCR (pix2tex), 손그림 검색, 화학 mhchem 확장

세부 — [`.doc/PRD.md` §13](.doc/PRD.md).

---

## 라이선스

본체 MIT (예정). 의존성:
- MathJax — Apache-2.0
- Speech Rule Engine — Apache-2.0
- Tauri — MIT/Apache-2.0
- axum, tower-http — MIT
- nginx-proxy, acme-companion — MIT
