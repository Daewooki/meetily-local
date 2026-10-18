# 1.0.0 의존성 검토

2026-10-15에 배포 lockfile을 기준으로 `pnpm audit --prod`와 `cargo audit`를 실행하고 Apple Silicon Mac의 의존성 경로를 확인했습니다. 자동 검사만으로 보안 진단이나 조직의 사용 승인을 대신하지 않습니다. 아래 결과를 무시 목록으로 숨기지 않았습니다.

2026-10-16 공개용 소스에서 두 검사를 다시 실행했습니다. npm 경고 25건(낮음 2·중간 12·높음 9·심각 2), Rust 취약점 고지 5건과 유지보수 중단 7·unsound 2건으로, 아래 검토 대상은 동일했습니다.

## 반영한 수정

- 편집기의 ProseMirror View를 1.42.3으로 갱신해 붙여넣기 XSS 수정 반영
- Tiptap 2.27.2의 속성 병합에 prototype 변경 키 차단 패치 적용. 기존 BlockNote와의 호환성을 유지하며 악성 속성 입력 회귀 테스트 추가
- Markdown·링크 처리, UUID, PostCSS·관련 빌드 도구의 수정 버전 반영
- 앱의 reqwest를 0.12.28로 갱신하고 Atlassian에는 시스템 TLS를 명시적으로 선택
- crossbeam-epoch, plist/quick-xml, rustls, anyhow, event-listener 등 호환되는 수정 버전 반영
- PostHog 전송 코드·의존성과 사용하지 않는 네이티브 updater 의존성 제거
- 외부 링크 열기를 HTTPS로 제한하고 쉘을 거치지 않도록 처리

Tiptap 패치: `frontend/patches/@tiptap__core@2.27.2.patch`. `pnpm install --frozen-lockfile`로 적용되며 `frontend/tests/lib/editor-security.test.cjs`에서 prototype 변조와 정상 속성 병합을 확인합니다. 버전 기반 audit에는 패치 적용 여부가 반영되지 않습니다.

## 남아 있는 경고와 적용 범위

| 항목 | 이 배포판에서의 판단 |
| --- | --- |
| Next.js 14의 서버 관련 경고 | 앱은 `output: export`, 이미지 최적화 비활성 구성의 정적 파일만 포함합니다. Next 서버·Server Actions·이미지 최적화 API는 배포하지 않습니다. 개발 서버는 127.0.0.1에만 바인딩하며 외부 서비스로 배포하지 않습니다. Next/React의 메이저 업그레이드는 후속 작업입니다. |
| Tiptap GHSA-cp6q-959q-f8rh | 위 소스 패치와 회귀 검증으로 보완했습니다. 버전 번호는 유지하므로 audit에는 남습니다. |
| braces GHSA-vfj7-8cjw-p6xm | Tailwind/Chokidar 개발·파일 감시 경로입니다. 해당 Node 실행 코드는 설치 앱에 포함하지 않습니다. 수정 버전은 검사 시점에 제시되지 않았습니다. |
| ringbuf RUSTSEC-2026-0293 | Mac 오디오 버퍼에서 사용합니다. 취약 조건은 원소의 Drop이 panic하는 경우이며 이 앱의 버퍼 원소는 Drop을 구현하지 않는 f32입니다. 다른 원소 타입을 도입하면 재검토해야 합니다. |
| quick-xml 0.37.5, rkyv 0.7, rsa 0.9 | 전체 lockfile 검사에 포함되지만 Apple Silicon Mac의 normal/build 의존성 경로에서는 확인되지 않았습니다. 다른 OS를 지원할 때 다시 검토합니다. Mac의 plist 경로는 quick-xml 0.42로 갱신했습니다. |
| atty·glib 및 유지보수 중단 고지 | atty는 nnnoiseless→clap 경로로 Mac에도 포함됩니다. 해당 unsound 고지는 Windows 구현의 포인터 정렬 문제이며, 현재 Mac의 Unix 구현과 구분합니다. glib는 Mac 실행 앱의 normal/build 경로에 없습니다. HTML 처리·매크로 관련 유지보수 중단 고지도 남아 있어 상위 라이브러리 변경을 추적해야 합니다. |

최종 버전 기반 검사에는 npm 경고 25건(Next.js·Tiptap·braces), Rust 취약점 고지 5건과 유지보수/unsound 고지가 남습니다. 이는 “알려진 취약점 0건”인 배포판이 아닙니다. 위 판단은 현재 소스와 Mac 배포 방식에 한정되며, 서버 배포나 다른 OS 지원 시 그대로 적용할 수 없습니다.

## 참고와 재현

- [ProseMirror 보안 고지](https://github.com/advisories/GHSA-c8x8-7fp4-3x9w)
- [Tiptap 보안 고지](https://github.com/ueberdosis/tiptap/security/advisories/GHSA-cp6q-959q-f8rh)
- [ringbuf 보안 고지](https://rustsec.org/advisories/RUSTSEC-2026-0293.html)
- [빌드·검증 명령](RELEASING.md)

원시 검사 출력과 의존성 경로는 Git에서 제외한 `local-validation/v1-*-audit-*.json` 등에 보관합니다. 재현 시점의 고지 데이터베이스에 따라 결과는 달라질 수 있습니다.

## Windows 확장 검토 (2026-10-19)

Windows x64의 normal/build 의존성 경로를 별도로 확인했습니다.

- `nnnoiseless`의 사용하지 않는 CLI 기본 기능을 끄면서 `clap 3`와 `atty`를 제거했습니다. 오디오 처리는 기존 `DenoiseState`를 유지합니다.
- 사용처가 없는 직접 의존성 `esaxx-rs`를 제거했습니다. Windows MSVC의 정적/동적 CRT 혼합 가능성도 함께 제거됩니다.
- `quick-xml 0.37.5`는 Windows 알림의 `tauri-winrt-notification` 경로에 있습니다. 이 버전의 해당 라이브러리에서는 텍스트/이미지 속성의 `escape` 함수만 사용하며, 고지 대상인 XML Reader/NamespaceResolver 파싱 경로는 호출하지 않습니다. 경고 자체를 무시 처리하지 않았으며, 상위 라이브러리 갱신을 추적합니다.
- `rkyv`와 `rsa`는 Windows x64 normal/build 경로에서도 확인되지 않았습니다.
- Windows 설치 파일은 ONNX Runtime, Microsoft Visual C++ 런타임과 WebView2를 포함합니다. 이 바이너리와 운영체제의 보안 업데이트는 Cargo/npm 검사 범위에 포함되지 않습니다.

이 기록은 기존 Mac 1.0.0 배포 파일의 의존성 구성을 소급해서 변경하지 않습니다.
