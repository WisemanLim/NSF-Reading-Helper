# NSF Reading Helper — Web Deployment

> 데스크탑 Tauri 앱과 **동일한 Rust 코어**(분석기 + SQLite 사전)를 그대로 재사용하는 웹앱 배포 가이드.
> `nginx-proxy` + `acme-companion` 으로 Let's Encrypt 인증서를 자동 발급/갱신하며, `docker compose up -d` 한 줄로 띄울 수 있습니다.

---

## 목차

1. [구성 개요](#구성-개요)
2. [시스템 요구사항](#시스템-요구사항)
3. [빠른 시작 (5분 배포)](#빠른-시작-5분-배포)
4. [디렉터리 / 파일 구조](#디렉터리--파일-구조)
5. [환경 변수](#환경-변수)
6. [빌드](#빌드)
7. [배포](#배포)
8. [운영 / 데이터](#운영--데이터)
9. [TLS / 도메인](#tls--도메인)
10. [데스크탑 코드와의 관계](#데스크탑-코드와의-관계)
11. [API 명세](#api-명세)
12. [트러블슈팅](#트러블슈팅)
13. [로컬 개발 (컨테이너 없이)](#로컬-개발-컨테이너-없이)
14. [보안 / 인증 메모](#보안--인증-메모)

---

## 구성 개요

```
                ┌────────────────────────────────────────────────┐
public 80/443   │   nginx-proxy  ──┐                            │
                │                  │  VIRTUAL_HOST 라우팅        │
                │   acme-companion │  + Let's Encrypt 자동 발급  │
                │                  ▼                            │
                │            ┌──────────────┐                   │
                │            │  frontend    │  React/Vite SPA   │
                │            │  (nginx)     │  + /api 리버스    │
                │            └──────┬───────┘                   │
                │                   │  http://backend:8080      │
                │            ┌──────▼───────┐                   │
                │            │  backend     │  axum + analyzer  │
                │            │  (Rust)      │  + SQLite 사전    │
                │            └──────────────┘                   │
                └────────────────────────────────────────────────┘
```

- **백엔드 (`backend`)** — `web-server/` 크레이트. axum 0.7. 데스크탑의 `src-tauri/src/core/*` 를 `#[path]` 로 그대로 재사용 (분석기, SQLite, 시드 사전).
- **프론트엔드 (`frontend`)** — 동일한 React/Vite 빌드. 런타임에 `window.__TAURI_INTERNALS__` 존재 여부로 데스크탑/웹을 감지해 IPC 어댑터를 스위치 ([`src/ipc/client.ts`](src/ipc/client.ts)).
- **TLS** — `nginxproxy/nginx-proxy` + `acme-companion` 페어가 라벨 기반 가상 호스트 생성, Let's Encrypt 인증서 자동 발급·갱신.

---

## 시스템 요구사항

| 항목 | 최소 | 권장 |
| :-- | :-- | :-- |
| OS | Linux x86_64 / arm64 (Docker 가능한 호스트) | Ubuntu 22.04 LTS |
| CPU | 1 vCPU | 2 vCPU |
| RAM | 512 MB | 1 GB |
| 디스크 | 500 MB | 5 GB |
| 네트워크 | 80/443 인바운드 | 도메인 1개 + 정방향 DNS |
| Docker | 24.x + `docker compose` v2 | 최신 |

> NSF Reading Helper 는 무거운 ML 모델을 사용하지 않으므로 리소스 요구가 낮습니다.

---

## 빠른 시작 (5분 배포)

### A. 운영 (TLS · 도메인 사용)

```bash
# 1. 코드 가져오기
git clone https://github.com/<your-org>/NSF-Reading-Helper.git
cd NSF-Reading-Helper

# 2. .env 작성
cp .env.example .env
$EDITOR .env   # DOMAIN, LETSENCRYPT_EMAIL 수정

# 3. DNS가 가리키는 서버에서 (인증서 발급에 필요)
docker compose up -d --build

# 4. 로그 확인 (인증서 발급은 1~2분 소요)
docker compose logs -f acme nginx-proxy backend
```

브라우저에서 `https://<DOMAIN>` 접속 → 첫 페이지에서 수식 입력 → "분석/읽기" → 토큰 클릭으로 기호 설명 확인.

### B. 로컬 (TLS 없음, localhost)

도메인·인증서 없이 docker 만으로 띄우려면 **`docker-compose.local.yml`** 을 사용하세요. nginx-proxy/acme-companion 없이 backend + frontend 만 기동되어 `https → 503` 문제가 발생하지 않습니다.

```bash
# 기본 포트 8000
docker compose -f docker-compose.local.yml up -d --build

# 포트 바꾸기 (예: 3000)
NSF_HOST_PORT=3000 docker compose -f docker-compose.local.yml up -d --build

# 정지 / 재빌드
docker compose -f docker-compose.local.yml down
docker compose -f docker-compose.local.yml up -d --build --force-recreate
```

브라우저에서 **http://localhost:8000** 접속.

| 항목 | 운영 (`docker-compose.yml`) | 로컬 (`docker-compose.local.yml`) |
| :-- | :-- | :-- |
| TLS | Let's Encrypt 자동 | 없음 (plain HTTP) |
| 도메인 | 필수 (`DOMAIN`) | 불필요 (localhost) |
| 노출 포트 | 80, 443 | `NSF_HOST_PORT` (기본 8000) |
| nginx-proxy | 사용 | 미사용 |
| acme-companion | 사용 | 미사용 |
| 데이터 볼륨 | `nsf-data` | `nsf-data` (양쪽 호환) |
| 사용 시점 | 공개 도메인 배포 | 개발자 PC, 사내망, NAS |

---

## 디렉터리 / 파일 구조

```
.
├── docker-compose.yml          # nginx-proxy + acme + backend + frontend
├── .env.example                # DOMAIN, LETSENCRYPT_EMAIL 템플릿
├── Dockerfile.web              # 백엔드 (Rust → debian-slim)
├── Dockerfile.web-frontend     # 프론트엔드 (pnpm build → nginx alpine)
├── deploy/
│   └── nginx.app.conf          # 프론트 컨테이너 내부 nginx 설정
├── web-server/                 # axum 크레이트 (코어 재사용)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs             # REST 라우터 + 핸들러
│       └── core.rs             # #[path] 로 src-tauri/src/core/* 재포함
├── src/                        # React 프론트엔드 (데스크탑과 공유)
│   ├── ipc/
│   │   ├── client.ts           # 스위처 (tauri ↔ web)
│   │   ├── tauriClient.ts      # 데스크탑 (invoke)
│   │   └── webClient.ts        # 웹 (fetch)
│   ├── pages/                  # Home/Analyze/Search/Library/Settings
│   ├── components/             # FormulaEditor, RenderView, SymbolPanel ...
│   ├── lib/
│   │   ├── mathjax.ts          # MathJax 3 부트스트랩
│   │   ├── sre.ts              # Speech Rule Engine 한국어 읽기
│   │   └── env.ts              # isTauri 런타임 감지
│   └── ...
└── src-tauri/                  # 데스크탑 (변경 없음)
    └── src/core/               # 백엔드/데스크탑 공통 코어 (db, analyzer, seed, models)
```

---

## 환경 변수

`.env` 에 정의되어 `docker compose` 가 자동 로드합니다.

| 변수 | 기본값 | 용도 |
| :-- | :-- | :-- |
| `DOMAIN` | — (필수) | 공개 도메인. nginx-proxy `VIRTUAL_HOST`, acme `LETSENCRYPT_HOST` |
| `LETSENCRYPT_EMAIL` | — (필수) | Let's Encrypt 갱신 알림 메일 |
| `CLIENT_MAX_BODY_SIZE` | `32m` | nginx-proxy(에지) 최대 요청 크기 |
| `RUST_LOG` | `info` | 백엔드 로그 레벨 |

백엔드 컨테이너 내부 추가:

| 변수 | 기본값 | 용도 |
| :-- | :-- | :-- |
| `NSF_DATA_DIR` | `/data` | SQLite 사전/히스토리/즐겨찾기 디렉터리 |
| `NSF_BIND` | `0.0.0.0:8080` | 리스닝 주소 |

---

## 빌드

```bash
# 두 이미지 빌드
docker compose build

# 백엔드만
docker compose build backend

# 프론트엔드만
docker compose build frontend

# 캐시 무시
docker compose build --no-cache backend
```

### 백엔드 (`Dockerfile.web`)

- 빌드: `rust:1.82-bookworm`
- 런타임: `debian:bookworm-slim` + `ca-certificates`
- BuildKit 캐시 마운트로 `cargo` 레지스트리·`target/` 재사용
- 결과 바이너리: `/usr/local/bin/nsf-reading-helper-web`
- 첫 빌드 3–6분. 캐시되면 30초 이내.

### 프론트엔드 (`Dockerfile.web-frontend`)

- `node:20-bookworm-slim` + `pnpm@10.33` (corepack)
- `pnpm install && pnpm build` → `dist/`
- 런타임: `nginx:1.27-alpine` + [`deploy/nginx.app.conf`](deploy/nginx.app.conf)
- 빌드 ARG `VITE_NSF_API_BASE` 로 API base 지정 가능 (기본: 빈 문자열 = 같은 오리진)

---

## 배포

```bash
# 최초 기동
docker compose up -d --build

# 상태
docker compose ps

# 인증서 발급 진행
docker compose logs -f acme

# 컴포넌트 분리 모니터링
docker compose logs -f backend
docker compose logs -f frontend
docker compose logs -f nginx-proxy

# 코드 수정 후 재배포
docker compose up -d --build

# 정지
docker compose stop

# 완전 삭제 (데이터 유지)
docker compose down

# 데이터까지 삭제 (주의 — SQLite, 인증서 전부 날아감)
docker compose down -v
```

---

## 운영 / 데이터

### 데이터 위치 (Docker 볼륨)

| 볼륨 | 컨테이너 경로 | 내용 |
| :-- | :-- | :-- |
| `nsf-data` | `backend:/data` | `nsf-reading.sqlite` (사전 + 사용자 데이터) |
| `certs` | `*:/etc/nginx/certs` | Let's Encrypt 인증서 |
| `vhost` | `*:/etc/nginx/vhost.d` | nginx-proxy 동적 vhost |
| `html` | `*:/usr/share/nginx/html` | ACME HTTP-01 챌린지 파일 |
| `acme` | `acme:/etc/acme.sh` | acme.sh 계정/상태 |

볼륨 위치 (호스트):
```bash
docker volume inspect nsf-data | grep Mountpoint
# 보통 /var/lib/docker/volumes/<프로젝트>_nsf-data/_data
```

### 백업

```bash
# SQLite 백업
docker run --rm -v nsf-data:/data -v "$PWD":/backup \
  alpine tar czf /backup/nsf-data-$(date +%F).tgz -C /data .

# 복원
docker run --rm -v nsf-data:/data -v "$PWD":/backup \
  alpine sh -c "cd /data && tar xzf /backup/nsf-data-2026-05-19.tgz"
```

### 사전 초기화

`nsf-data` 볼륨의 SQLite 가 비어 있으면 백엔드 부팅 시 자동으로 시드(약 60개 핵심 기호)를 설치합니다. 사전을 초기화하려면 컨테이너 정지 후 SQLite 파일을 삭제하면 됩니다:

```bash
docker compose stop backend
docker run --rm -v nsf-data:/data alpine rm -f /data/nsf-reading.sqlite
docker compose start backend
```

### 로그

```bash
docker compose logs -f
docker compose logs -f backend
# 영구 debug 모드:  .env 의 RUST_LOG=debug 후 docker compose up -d backend
```

---

## TLS / 도메인

1. **DNS** — `DOMAIN` 이 서버의 공개 IP 로 A/AAAA 레코드를 가리킬 것.
2. **포트** — 호스트 `80`, `443` 비어 있어야 함.
3. **방화벽** — `ufw allow 80/tcp && ufw allow 443/tcp`.
4. **이메일** — `LETSENCRYPT_EMAIL` 은 실제 받을 수 있는 주소 권장.
5. **첫 발급** — `docker compose up -d` 후 30초~수분. `docker compose logs acme` 로 확인.
6. **갱신** — acme-companion 이 매시간 만료 30일 미만 인증서 자동 갱신.

### 스테이징(테스트) 인증서

`.env` 에 추가:
```env
ACME_CA_URI=https://acme-staging-v02.api.letsencrypt.org/directory
```
정식 운영 직전에 제거 → `docker compose up -d acme` 로 재발급.

---

## 데스크탑 코드와의 관계

핵심은 **데스크탑 코드 그대로 재사용** 입니다.

```rust
// web-server/src/core.rs
#[path = "../../src-tauri/src/core/error.rs"]    pub mod error;
#[path = "../../src-tauri/src/core/models.rs"]   pub mod models;
#[path = "../../src-tauri/src/core/seed.rs"]     pub mod seed;
#[path = "../../src-tauri/src/core/db.rs"]       pub mod db;
#[path = "../../src-tauri/src/core/analyzer.rs"] pub mod analyzer;
#[path = "../../src-tauri/src/core/settings.rs"] pub mod settings;
```

`web-server/src/main.rs` 만 새로 작성 — axum 라우터 핸들러가 Tauri command 와 1:1 대응.

프론트엔드도 동일 코드:

```ts
// src/ipc/client.ts
export const ipc = isTauri ? tauriIpc : webIpc;
```

| 호출 | 데스크탑 | 웹 |
| :-- | :-- | :-- |
| `ipc.analyze(...)` | `invoke("analyze")` | `POST /api/expressions/analyze` |
| `ipc.symbolSearch(q)` | `invoke("symbol_search")` | `GET /api/symbols/search?q=...` |
| `ipc.symbolGet(id)` | `invoke("symbol_get")` | `GET /api/symbols/:id` |
| `ipc.settingsGet()` | `invoke("settings_get")` | `GET /api/settings` |
| `ipc.ttsSpeak(...)` | OS TTS (say/espeak/SAPI) | `window.speechSynthesis` |

---

## API 명세

전체 응답은 `application/json`. 에러 시 `{ "error": "..." }`.

| 메서드 | 경로 | 설명 |
| :-- | :-- | :-- |
| GET | `/api/healthz` | 헬스 체크 |
| POST | `/api/expressions/analyze` | 입력 수식 → 토큰/읽기 |
| GET | `/api/symbols/search?q=&domain=&symbolType=&limit=` | 기호 검색 |
| GET | `/api/symbols/:id` | 기호 상세 (별칭/분야/예시/유사기호) |
| GET | `/api/symbols/:id/examples` | 예시 수식 |
| GET | `/api/history` | 최근 분석 100개 |
| GET | `/api/favorites` | 즐겨찾기 목록 |
| POST | `/api/favorites` | `{ symbolId }` 즐겨찾기 추가 |
| DELETE | `/api/favorites/:id` | 즐겨찾기 삭제 |
| GET | `/api/notes` | 노트 목록 |
| POST | `/api/notes` | 노트 추가/갱신 |
| GET | `/api/settings` | 설정 조회 |
| PUT | `/api/settings` | 설정 저장 |

### POST `/api/expressions/analyze` 예시

요청:
```json
{ "rawInput": "\\int_0^1 x^2 \\, dx", "format": "latex", "locale": "ko" }
```

응답:
```json
{
  "expressionId": 12,
  "normalizedLatex": "\\int_0^1 x^2 \\, dx",
  "speech": { "ko": "적분 기호, ..." },
  "tokens": [
    {
      "index": 0,
      "tokenText": "\\int",
      "role": "command",
      "symbolId": 21,
      "nameKo": "적분 기호",
      "readingKo": "인테그럴",
      "meaningKo": "연속적인 변화량의 누적..."
    }
  ]
}
```

> 백엔드는 토큰별 메타데이터를 채워줄 뿐, 자연스러운 한국어 문장 읽기는 프론트엔드의 SRE 가 생성해 덮어씁니다.

---

## 트러블슈팅

| 증상 | 원인 / 해결 |
| :-- | :-- |
| `acme` 가 `Connection refused` 로 챌린지 실패 | DNS 미전파 또는 80 포트 차단. `dig +short $DOMAIN`, `curl -I http://$DOMAIN/.well-known/acme-challenge/test` |
| `502 Bad Gateway` | `backend` 가 죽었거나 부팅 중. `docker compose logs backend` |
| `Cargo` 빌드 실패 | `--no-cache` 로 재빌드. Rust 버전 차이면 `Dockerfile.web` 의 `rust:1.82` 를 최신으로 |
| MathJax 로드 실패 | CDN 차단. `src/lib/mathjax.ts` 의 `CDN` 상수를 사내 미러로 변경 |
| SRE 한국어 읽기 어색 | SRE 한국어 룰셋이 제한적 — 향후 `ko-clearspeak.json` 커스텀 룰 추가 예정 |
| Apple Silicon 빌드 매우 느림 | `linux/amd64` qemu 에뮬레이션. 호스트가 arm64 이면 BuildKit 이 자동으로 native arch 선택 |
| `ERR_CONNECTION_REFUSED` (http://) | http→https 리디렉트는 nginx-proxy 자동. 인증서 발급 완료 후 적용. acme 로그 확인 |
| 검색 결과가 비어있음 | SQLite 사전이 비어 있을 수 있음. `docker compose exec backend ls /data` 로 `nsf-reading.sqlite` 확인. 없으면 backend 재시작 시 자동 시드 |

---

## 로컬 개발 (컨테이너 없이)

### 백엔드만

```bash
# 사전 요구: Rust 1.82+, gcc/clang
cd web-server
NSF_DATA_DIR=./.dev-data RUST_LOG=info cargo run --release
# → http://localhost:8080
```

### 프론트엔드만 (개발 서버)

```bash
VITE_NSF_API_BASE=http://localhost:8080 pnpm dev
# → http://localhost:1420 (Vite). 브라우저에서 열면 web 모드로 동작
```

`VITE_NSF_API_BASE` 는 빌드 시점에 정적 치환됩니다. 컨테이너 배포 빌드 시에는 비워두세요 (같은 오리진 = nginx 가 `/api` 를 프록시).

### 데스크탑은 그대로

웹 작업 중에도 `pnpm tauri:dev` 는 정상 작동. 두 모드는 같은 `src/` 를 공유하며 `isTauri` 로 분기될 뿐입니다.

---

## 보안 / 인증 메모

본 배포는 **단일 인스턴스 / 인증 없음** 기본형입니다 (`/api/*` 공개).
- 사설망/VPN 뒤에서만 노출하거나
- nginx-proxy 에 HTTP Basic Auth (`/etc/nginx/htpasswd/<DOMAIN>` 파일 + `BASIC_AUTH=true`)
- 또는 Cloudflare Access / Tailscale Funnel / Authelia 등 외부 인증 레이어 전치

추후 사용자 계정 기반 격리가 필요해지면 `web-server/` 에 미들웨어 + DB 컬럼 (user_id) 추가가 자연스러운 진화 경로입니다.

---

## 라이선스 / 출처

- 본체: MIT (예정)
- [MathJax](https://www.mathjax.org) — Apache-2.0
- [Speech Rule Engine](https://speechruleengine.org) — Apache-2.0
- [nginxproxy/nginx-proxy](https://github.com/nginx-proxy/nginx-proxy) — MIT
- [nginxproxy/acme-companion](https://github.com/nginx-proxy/acme-companion) — MIT
- [axum](https://github.com/tokio-rs/axum) — MIT
