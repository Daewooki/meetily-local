# Meetily Local 개발 지침

이 저장소는 Meetily Community 기반의 로컬 회의록 앱입니다. 현재 배포 대상은 Apple Silicon Mac / macOS 14.2 이상입니다.

## 구성과 명령

- `frontend/src`: Next.js 정적 UI
- `frontend/src-tauri/src`: Rust/Tauri 녹음·전사·요약·Atlassian 연동
- `llama-helper`: 로컬 LLM 프로세스
- `backend/`: 원본 참고용 코드, 이 배포판에서 실행하지 않음
- 전체 빌드: `bash scripts/build-local-mac.sh`
- 패키징: `bash scripts/package-local-mac.sh`
- 소스 배포 검사: `python3 scripts/check-release-source.py`
- 검증·릴리스 절차: `docs/RELEASING.md`

## 유지할 동작

외부 AI 호출, 사용 통계, 자동 업데이트, 실행 중 실행 파일 다운로드를 다시 켜지 않습니다. 제품 방향 변경을 명시적으로 요청받은 경우 통신과 데이터 흐름을 먼저 검토합니다. 모델은 설치 파일과 별도로 다운로드합니다.

Atlassian 요청은 사용자 동작으로 시작합니다. 토큰은 네이티브 보관소에서만 사용하고 저장 후 화면·로그에 반환하지 않습니다. Jira 보드는 선택사항이며 이슈 생성과 보드 필터를 구분합니다. 게시 전 사용자가 제목·본문·대상을 검토하고 확인합니다.

앱 ID `ai.meetily.local`, 데이터 위치, 저장 연결의 호환성을 유지합니다. 사용자 회의·모델·보관소를 소스나 설치 파일에 넣지 않습니다. 테스트는 합성 자료와 격리된 저장소로 실행합니다.

## 기여

기준 브랜치는 `main`입니다. `CONTRIBUTING.md`와 PR 템플릿을 따릅니다. `docs/upstream-workflows/`와 `docs/BUILDING.md`는 원본 참고 자료이며 현재 배포 절차가 아닙니다. 변경에 맞는 검증을 실행하고 결과와 한계를 정확히 기록합니다.
