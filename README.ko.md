# Meetily Local

각자의 PC에서 회의를 녹음·전사·요약하고, 검토한 내용을 Confluence와 Jira에 게시하는 데스크톱 앱입니다. [Meetily Community](https://github.com/Zackriya-Solutions/meetily)를 기반으로 만든 독립적인 커뮤니티 포크이며, 공식 Meetily 배포판이나 Atlassian 제품은 아닙니다.

[사용법](LOCAL-MAC.md) · [영상 안내](docs/VIDEO-GUIDE.md) · [통신 범위](NETWORK-POLICY.md) · [검증 기록](LOCAL-VALIDATION.md) · [기여 안내](CONTRIBUTING.md)

## 공개된 설치 파일의 사용 환경 (1.0.0)

| 항목 | 사양 |
| --- | --- |
| 하드웨어 | Apple Silicon Mac, M1 이상 |
| 운영체제 | macOS 14.2 이상 |
| 메모리 | 16GB 이상 권장 |
| 저장 공간 | 5GB 이상 + 녹음 저장 공간 별도 |

Windows x64 지원은 [자동 빌드](https://github.com/Daewooki/meetily-local/actions/workflows/windows.yml)로 검증 중입니다. [Windows 사용·빌드 안내](docs/WINDOWS.md)에서 사양과 검증 범위를 확인하세요. 기존 1.0.0 설치 파일은 Apple Silicon Mac용입니다. Linux·Intel Mac은 아직 배포하지 않으며, 플랫폼 지원과 검증에 참여해 주실 분을 환영합니다.

## 이 포크에서 추가한 것

로컬 녹음·전사·요약 엔진은 원본 Meetily Community의 기능입니다. 이 포크는 **검토 후 Wiki·Jira 게시**, **원문 근거를 확인하는 한국어 회의록**, **외부 AI 호출을 제한한 설정**에 집중합니다.

## 할 수 있는 일

- 마이크·시스템 오디오 녹음 또는 녹음 파일 가져오기
- 로컬 Whisper 전사와 Qwen 기반 한국어 회의록 작성
- 원문 근거를 확인하는 기본 회의록 및 사용자 템플릿·프롬프트
- Confluence 페이지 또는 폴더 아래에 새 회의록 페이지 게시
- Jira 프로젝트에 이슈 생성, 선택적 보드·컴포넌트 지정

모델 약 3.3GB를 준비하면 전사·요약은 오프라인으로 동작합니다. 모델 다운로드와 Atlassian 연결·조회·게시에는 네트워크를 사용합니다. 녹음과 전체 전사문은 자동 업로드하지 않으며, 게시 본문에 직접 넣은 내용은 전송됩니다.

API 토큰은 AES-256-GCM으로 암호화하고 키는 macOS Keychain 또는 Windows 자격 증명 관리자에 보관합니다. 녹음·회의 DB 전체 암호화는 현재 앱에서 제공하지 않습니다. 사용 통계, 자동 업데이트 확인, 외부 AI 호출은 비활성화했습니다.

## 설치와 사용

[1.0.0 릴리스](https://github.com/Daewooki/meetily-local/releases/tag/v1.0.0)에서 DMG 또는 ZIP을 받거나 [직접 빌드](docs/RELEASING.md)합니다. 소스에는 설치 파일·모델·개인 데이터가 포함되지 않습니다. 현재 빌드는 Apple 공증을 받지 않은 ad-hoc 서명이며, 관리 중인 기기에서는 실행이 제한될 수 있습니다.

설치 후 모델 준비 → 녹음·파일 가져오기 → 요약 검토 → 선택한 결과 게시 순서로 사용합니다. 입력 예시와 오류 해결은 [Mac 사용법](LOCAL-MAC.md)을 참고하세요. 계정과 API 토큰은 사용자가 직접 준비합니다.

AI 요약은 사실과 다를 수 있으므로 담당자·기한·결정 사항을 확인하세요. 기존 Wiki 페이지 수정과 Jira 담당자·기한·필수 사용자 정의 필드 지정은 아직 지원하지 않습니다. 보드에 이슈가 표시되려면 해당 보드의 필터·상태 조건을 충족해야 합니다.

## 함께 만들기

버그 제보, 한국어 품질 개선, 접근성, 템플릿과 프롬프트, 다른 운영체제 지원 등 Fork와 PR을 환영합니다. 실제 녹음·회의록·API 토큰 대신 합성 예시로 제보해 주세요. [기여 절차](CONTRIBUTING.md)와 [의존성 검토](docs/DEPENDENCY-REVIEW.md)를 확인할 수 있습니다.

Meetily Community v0.4.1의 원본 MIT 라이선스·저작권 고지를 유지합니다. FFmpeg는 LGPL-2.1-or-later이며 대응 소스와 빌드 스크립트를 함께 제공해야 합니다. [서드파티 고지](THIRD_PARTY_NOTICES.md)를 참고하세요.
