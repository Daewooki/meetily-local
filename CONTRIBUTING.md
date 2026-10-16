# 함께 만들기

Meetily Local은 각자 PC에서 회의록을 정리하고 필요한 결과를 팀에 공유하는 도구입니다. 작은 수정부터 Fork와 Pull Request를 환영합니다.

## 어떤 기여든 환영합니다

- 한국어 전사·요약 오류를 재현하는 합성 예시와 개선
- 팀에 맞는 회의록 템플릿·시스템 프롬프트
- Wiki·Jira 연동, 사용성·접근성, 문서 개선
- Windows·Linux·Intel Mac 지원과 해당 환경의 검증

현재 설치 파일과 검증 대상은 Apple Silicon Mac입니다. 다른 운영체제 지원에는 녹음, 모델 실행, 빌드·패키징뿐 아니라 OS에 맞는 토큰 보관 방식도 함께 필요합니다. 큰 변경은 먼저 Issue로 방향을 공유해 주세요.

## Fork에서 PR까지

1. 이 저장소를 자신의 GitHub 계정으로 Fork합니다.
2. 자신의 Fork를 clone하고 `main`에서 작업 브랜치를 만듭니다.
3. 변경한 기능을 확인하고 목적이 분명한 커밋을 만듭니다.
4. 자신의 브랜치에서 이 저장소의 **main**을 대상으로 PR을 엽니다.
5. 해결한 문제, 변경 후 동작, 확인한 환경·결과, 남은 제한을 적습니다.

문서·오타 수정에 테스트 코드를 추가할 필요는 없습니다. 기능 수정에는 해당 문제를 재현하고 회귀를 막을 수 있는 검증을 포함해 주세요. 자동 CI는 현재 제공하지 않으므로 로컬 확인 결과를 PR에 기록합니다.

## 개발 환경

Apple Silicon Mac, macOS 14.2 이상, Xcode Command Line Tools, Node.js 22 이상, Rust stable, CMake, Python 3.11 이상, make가 필요합니다.

```sh
bash scripts/build-local-mac.sh
```

전체 빌드 및 로컬 개발 실행은 [빌드·릴리스 안내](docs/RELEASING.md)를 참고하세요. `backend/`는 원본 보관용이며 이 앱에서 실행하지 않습니다.

주요 위치:

| 변경 대상 | 위치 |
| --- | --- |
| 화면 | `frontend/src` |
| 로컬 처리·Atlassian 연동 | `frontend/src-tauri/src` |
| 기본 회의록 템플릿 | `frontend/src-tauri/templates/standard_meeting.json` |
| 한국어 시스템 지시·구조화 처리 | `frontend/src-tauri/src/summary/korean.rs`, `korean_report.rs` |
| 모델 실행 프로세스 | `llama-helper` |

기본 한국어 템플릿을 수정하면 자유 형식 Markdown 경로를 사용합니다. 기본 템플릿의 구조 검수와 동일하다고 가정하지 말고 [요약 품질 검증](docs/SUMMARY-QUALITY.md)을 참고하세요.

## 데이터와 라이선스

실제 회의 녹음·전사문, 회사 문서, API 토큰, 계정 연결 파일, 인증서, 개인 경로를 Issue·PR·테스트·배포 파일에 넣지 않습니다. 합성 예시를 사용하고 제출 전 `python3 scripts/check-release-source.py`를 실행해 주세요.

외부 AI 호출·사용 통계·자동 업로드를 추가하는 변경은 로컬 처리 원칙에 영향을 줍니다. 통신 대상과 전달 데이터를 명확히 설명하고 기존 기본 동작을 유지해 주세요. 저장한 API 토큰을 화면이나 로그에 반환하지 않습니다.

기여 코드는 이 저장소의 [MIT 라이선스](LICENSE.md)로 공유합니다. 새 의존성을 추가하면 라이선스와 배포 고지도 함께 검토합니다.

## 원본 Meetily에 기여하기

이 포크의 PR 대상은 `main`입니다. 원본 Meetily에 제안할 때는 [원본 기여 지침](https://github.com/Zackriya-Solutions/meetily/blob/devtest/CONTRIBUTING.md)을 따르세요. 현재 지침은 `devtest`에서 기능 브랜치를 만들고 `devtest`를 대상으로 PR을 제출하도록 안내합니다.

배포용 브랜치 전체를 원본에 보내기보다, Atlassian 게시·한국어 요약 품질·보안 수정 등을 각각 검토 가능한 변경으로 나눕니다. 원본의 모델 선택·다른 플랫폼 지원을 포크의 정책으로 바꾸지 않습니다. 자세한 제안 범위는 [원본 기여 계획](docs/UPSTREAM-CONTRIBUTION.md)을 참고하세요.
