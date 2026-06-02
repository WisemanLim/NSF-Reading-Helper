# src-tauri/src/core/tts.rs — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: OS 별 TTS 명령 spawn (macOS `say`, Linux `espeak-ng`, Windows PowerShell).
- LOC: 62
- 주요 export: `speak(text, opts)`, `stop()`
- 외부 패키지: `tokio::process::Command`
- 부수효과: OS 프로세스 spawn, kill
- 동시성: async (실제론 process spawn 만)

---

```rust path=src-tauri/src/core/tts.rs lines=1-8
//! OS-level TTS shim. Web build goes through window.speechSynthesis directly;
//! desktop falls through to platform tools (macOS `say`, Linux `espeak-ng`,
//! Windows PowerShell). Best-effort — failures are non-fatal.

use crate::core::error::AppResult;
use crate::core::models::TtsOptions;
use tokio::process::Command;
```

```rust path=src-tauri/src/core/tts.rs lines=9-20
pub async fn speak(text: &str, opts: &TtsOptions) -> AppResult<()> {
    #[cfg(target_os = "macos")]
    {
        let rate_wpm = (180.0 * opts.rate).clamp(80.0, 400.0) as i64;
        let _ = Command::new("say")
            .args(["-r", &rate_wpm.to_string(), text])
            .spawn()
            .map(|mut c| async move {
                let _ = c.wait().await;
            });
        return Ok(());
    }
```

**분석 / Analysis**

- **`.args(["-r", ...., text])`** — args 배열로 전달 → 쉘 인터프리테이션 없음(macOS 의 `say` 가 execve 로 직접 받음). **셸 명령 주입 안전**.
- `rate_wpm = (180 * rate).clamp(80, 400)` — 정상 발화 속도 범위. opts.rate 가 1.0 이면 180wpm.
- `.spawn().map(|mut c| async move { let _ = c.wait().await; })` — **버그/안티패턴**: `.map()` 가 반환하는 `Result<impl Future, _>` 가 어디서도 await 되지 않음 → 그 async block 은 **실행되지 않는 future**. 결과적으로 `c.wait().await` 는 호출되지 않고, 자식 프로세스는 graceful wait 없이 종료 시 zombie 가 될 수 있음.
- *왜*: 사용자는 "Ok 즉시 반환" 을 원함 — fire-and-forget. 그러나 spawned future 가 실행 안 되는 건 의도에 어긋남. `tokio::spawn(async move { let _ = c.wait().await; })` 가 의도일 것.
- *흔한 실수*: `Result::map` 은 `Result<U,E>` 를 반환할 뿐, async block 을 자동 실행하지 않음.
- 결국 zombies 가 쌓이는지는 OS 가 정리(부모 프로세스 종료 시).

```rust path=src-tauri/src/core/tts.rs lines=21-34
    #[cfg(target_os = "linux")]
    {
        let _lang = opts.lang.as_str();
        let _ = Command::new("espeak-ng")
            .args([
                "-v",
                if opts.lang.starts_with("ko") { "ko" } else { "en" },
                "-s",
                &(160.0 * opts.rate).clamp(80.0, 400.0).to_string(),
                text,
            ])
            .spawn();
        return Ok(());
    }
```

**분석 / Analysis**

- `_lang = opts.lang.as_str()` — 사용되지 않는 변수(언더스코어 시작이라 컴파일 경고 회피). *왜*: 코드가 진화하며 남은 흔적. cleanup 가능.
- args 배열 전달이라 셸 주입 안전.
- `.spawn()` 결과 무시 — wait 도 하지 않음. zombie 우려는 macOS 와 동일.
- 한국어가 아니면 모두 `en` 으로 fallback — 일본어/중국어 등 지원 안 됨.

```rust path=src-tauri/src/core/tts.rs lines=35-50
    #[cfg(target_os = "windows")]
    {
        let ps = format!(
            "Add-Type -AssemblyName System.Speech; (New-Object System.Speech.Synthesis.SpeechSynthesizer).Speak('{}')",
            text.replace('\'', "''")
        );
        let _ = Command::new("powershell").args(["-Command", &ps]).spawn();
        let _ = opts.rate;
        return Ok(());
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        let _ = (text, opts);
        Ok(())
    }
}
```

**분석 / Analysis**

- **셸 명령 주입 위험**: PowerShell 명령 문자열을 `format!` 로 직접 구성. `text` 는 `text.replace('\'', "''")` 로 작은따옴표만 escape. 안전한 문자:
  - 작은따옴표 `'` → `''` (PowerShell 의 single-quoted string escape).
- *위험한 문자가 그대로 들어감*:
  - `;` → 새 명령. 단일 따옴표 안에선 무해(PS 가 리터럴로 처리).
  - `` ` `` (backtick) → PS escape — 단일 따옴표 안에선 리터럴이지만, 사용자가 PowerShell 의 보다 복잡한 구문(예: subexpression `$(...)`) 사용 시도해도 single-quoted 안에선 모두 리터럴.
- **실제 위험성 평가**: PowerShell 의 single-quoted string 은 `''` 외 모든 문자를 리터럴로 취급. 즉 작은따옴표만 escape 하면 *대부분* 안전. 그러나 사용자가 `'; Stop-Process -Name notepad; '` 형태 입력 시 escape 가 정확히 적용되어 안전.
- *그러나*: `args(["-Command", &ps])` 는 PowerShell 에 단일 문자열을 -Command 인자로 전달. PowerShell 은 -Command 인자를 **다시 파싱**. 즉 `''` escape 이 PowerShell 의 single-quoted string 안에서 의도대로 작동하는지가 핵심. 답: PowerShell 은 -Command 인자를 받아 그대로 실행 → single-quoted 안에선 OK. **실질 위험은 낮으나 보안 리뷰에서 자주 지적되는 안티패턴**.
- 안전한 대안:
  - 임시 `.ps1` 파일에 저장 + `-File` 사용. 또는
  - `-EncodedCommand` 로 Base64 인코딩 전달. 또는
  - Windows SAPI 를 Rust 에서 직접 호출(`win-tts` crate 등).
- `let _ = opts.rate` — Windows 분기에서 rate 를 사용하지 않음. PowerShell 명령에 rate 매개변수 없음. `SetRate(-10..10)` API 호출이 가능했는데 누락.

```rust path=src-tauri/src/core/tts.rs lines=52-62
pub async fn stop() -> AppResult<()> {
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("killall").arg("say").spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("killall").arg("espeak-ng").spawn();
    }
    Ok(())
}
```

**분석 / Analysis**

- `killall` — 동일 이름의 *모든* `say`/`espeak-ng` 프로세스 종료. 사용자가 다른 앱에서 동일 명령 실행 중이라면 의도치 않게 함께 종료. 데스크톱 단일 사용자 환경에선 일반적으로 OK.
- Windows 분기 누락 — PowerShell 합성 중지 방법이 없음. 사용자가 "정지" 누르면 아무 일도 안 일어남(Windows).

**짚어볼 점 / Watch-outs**
- 🔴 **Spawned future 누수**(macOS) — `.map(|mut c| async move ...)` 미실행. zombie 가능성.
- 🟡 **Windows PowerShell 명령 주입** — 현재 escape 로 대부분 안전하나 안티패턴.
- 🟡 **Windows stop() 누락**.
- 🟡 **Linux 의 `_lang` 미사용 변수** 잔재.

---

**파일 수준 평가 / File-Level Verdict**

- 강점: cfg-gate 로 플랫폼 분리 명확.
- 위험: spawned future 누수, Windows PS 패턴, stop() 누락.
- 개선 권고:
  1. macOS/Linux 의 `.spawn()` 결과를 `tokio::spawn(async move { let _ = c.wait().await; })` 로 감싸 zombie 회피.
  2. Windows TTS 를 SAPI 라이브러리(`tts` crate 등) 직접 호출 또는 임시 ps1 파일 + `-File`.
  3. Windows `stop()` 추가 — PowerShell 인스턴스 종료 또는 SAPI Cancel.
  4. Linux `_lang` 변수 제거.
- 재읽기 권고 레벨: 3 — 보안/리소스 누수 측면에서 다시 봐야 할 핫스팟.
