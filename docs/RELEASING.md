# Apple Silicon Mac 빌드·릴리스

Windows x64 빌드와 검증은 [Windows 안내](WINDOWS.md)를 따릅니다. 플랫폼별 검증 결과를 구분하고, 빌드 성공만으로 실제 녹음·모델 성능 검증이 완료됐다고 기록하지 않습니다.

## 빌드

Apple Silicon Mac, macOS SDK/Xcode Command Line Tools, Node.js 22 이상, Rust stable, CMake, Python 3.11 이상, make를 준비합니다. 첫 빌드는 npm·Cargo·FFmpeg 소스를 내려받습니다.

```sh
bash scripts/build-local-mac.sh
```

스크립트는 SHA-256이 고정된 FFmpeg 8.0.3 소스를 빌드하고, llama-helper를 빌드한 뒤 Tauri 앱을 생성합니다. FFmpeg는 외부 코덱 라이브러리·GPL 옵션·네트워크 프로토콜을 사용하지 않습니다. 모델은 설치 파일에 포함하지 않습니다.

앱: `target/release/bundle/macos/Meetily Local.app`.

앱의 배포·사용 안내 링크는 기본적으로 [Daewooki/meetily-local](https://github.com/Daewooki/meetily-local)을 가리킵니다. 다른 Fork를 배포한다면 빌드 전에 `NEXT_PUBLIC_PROJECT_REPOSITORY`를 해당 저장소 주소로 지정합니다.

개발 실행 전에도 `bash scripts/build-ffmpeg-local-mac.sh`와 helper 빌드가 필요합니다. `backend/`의 서버는 사용하지 않습니다. 원본 배포 워크플로는 `docs/upstream-workflows/`에 보관했으며 이 저장소에서 자동 실행하지 않습니다.

## 검증

```sh
MACOSX_DEPLOYMENT_TARGET=14.2 CARGO_BUILD_JOBS=8 cargo test --locked --release -p meetily --lib atlassian::tests
MACOSX_DEPLOYMENT_TARGET=14.2 CARGO_BUILD_JOBS=8 cargo test --locked --release -p meetily --lib local_build_rejects_external_ai_without_opening_a_socket
MACOSX_DEPLOYMENT_TARGET=14.2 CARGO_BUILD_JOBS=8 cargo test --locked --release -p meetily --lib summary::
MACOSX_DEPLOYMENT_TARGET=14.2 CARGO_BUILD_JOBS=8 cargo test --locked --release -p meetily --lib external_link_tests
MACOSX_DEPLOYMENT_TARGET=14.2 CARGO_BUILD_JOBS=8 cargo test --locked --release -p meetily --lib analytics::
cargo test --locked --release -p llama-helper
(cd frontend && npx --yes bun@1.3.14 test tests && node tests/lib/editor-security.test.cjs)
python3 scripts/check-release-source.py
```

앱 번들에 개인 토큰, `atlassian-vault.json`, 회의 DB, 녹음, `.env`, 모델 가중치를 넣지 않습니다. 릴리스 제작 스크립트는 빌드된 앱과 고지·설치 안내만 복사합니다. 새 사용자 데이터 폴더의 첫 실행을 확인한 뒤 공개합니다.

## 설치 파일 만들기

```sh
bash scripts/package-local-mac.sh
```

`release-artifacts/<버전>/`에 DMG, ZIP, SHA256SUMS.txt, FFmpeg 대응 소스를 만듭니다. 이 폴더는 Git에서 제외됩니다. 소스 커밋과 같은 버전의 태그를 만들고 GitHub Release에 이 파일들을 업로드합니다. 토큰/인증서/키를 스크립트나 저장소에 넣지 않습니다.

## 서명과 공증

현재 배포물은 ad-hoc 서명이고 Apple 공증을 받지 않았습니다. 사용자 PC의 정책에 따라 실행이 거부될 수 있습니다. Developer ID Application 인증서와 조직에서 관리하는 공증 자격 증명이 준비되면 Tauri의 서명·공증 절차를 적용해 재배포합니다. 개발용 Apple Development 인증서를 배포용 인증서로 대신 사용하지 않습니다.

[공식 Tauri macOS 서명 안내](https://v2.tauri.app/distribute/sign/macos/)

## 업데이트

설치할 버전을 확인한 뒤 앱을 종료하고 새 `Meetily Local.app`으로 교체합니다. 앱 ID `ai.meetily.local`과 사용자 데이터 폴더는 유지합니다. 자동 업데이트는 비활성화되어 있습니다. 서명 변경 시 Keychain 접근 확인이 다시 나타날 수 있습니다.

## 의존성 점검

릴리스 전에 `pnpm audit --prod`와 `cargo audit` 결과를 실제 배포 경로와 함께 검토합니다. 버전 기반 경고를 숨기거나 모두 수정했다고 간주하지 않습니다. Next.js는 정적 파일 생성에만 사용하며 `next start` 서버는 앱에 포함하지 않습니다. 유지 중인 보안 패치와 남은 의존성 항목은 [의존성 검토](DEPENDENCY-REVIEW.md)에 기록합니다.

`frontend/patches/`의 Tiptap 패치는 frozen lockfile 설치 시 자동 적용됩니다. 편집기 버전을 변경하면 패치 적용 여부와 `editor-security.test.cjs`를 확인하세요.
