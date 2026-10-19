# Windows x64 사용·빌드 안내

Windows x64 CPU 지원은 1.1.0 변경 사항입니다. Windows CI에서 빌드·설치 검사가 통과했으며, 실제 녹음 장치와 처리 속도는 아래 수동 확인 범위에 남아 있습니다. 기존 1.0.0 DMG/ZIP은 Mac용입니다. [Windows CI](https://github.com/Daewooki/meetily-local/actions/workflows/windows.yml)의 성공한 실행에서 `meetily-local-windows-x64` 아티팩트를 받을 수 있으며, 실제 장비 검증 범위는 아래와 같이 구분합니다.

## 대상 환경

| 항목 | 기준 |
| --- | --- |
| 운영체제 | Windows 11 x64 |
| CPU | AVX2 지원 Intel/AMD 64비트 CPU |
| 메모리 | 16GB 이상 권장 |
| 저장 공간 | 앱·모델용 6GB 이상 + 녹음 저장 공간 |
| GPU | 필수 아님. 현재 빌드는 CPU 전사·요약 |

Windows ARM64·32비트용 설치 파일은 제공하지 않습니다. CPU 속도에 따라 전사가 실시간보다 느릴 수 있습니다. Whisper·Qwen 모델은 Mac과 같지만 Metal 가속을 사용할 수 없으므로 같은 속도를 보장하지 않습니다. GPU 가속과 다양한 녹음 장치 검증 기여를 환영합니다.

## 설치와 첫 사용

1. Windows용 `*-setup.exe`를 실행합니다. 현재 사용자 계정에 설치하며, 코드를 서명하는 인증서는 아직 적용하지 않았습니다. 조직의 실행 정책에서 차단될 수 있습니다.
2. 앱을 열고 **로컬 모델 다운로드**를 선택합니다. Whisper와 Qwen 모델은 약 3.3GB이며 최초 준비에는 인터넷이 필요합니다.
3. **마이크 권한 요청**을 선택합니다. 접근이 거부되면 Windows 설정 → 개인정보 및 보안 → 마이크에서 마이크 액세스와 데스크톱 앱의 마이크 액세스를 확인합니다.
4. 녹음 화면에서 마이크와 상대방 소리가 나오는 출력 장치를 선택합니다. Windows의 시스템 소리는 WASAPI loopback으로 캡처하며 Mac의 화면 기록 권한 창은 사용하지 않습니다. 녹음 파일 가져오기도 가능합니다.
5. 전사·요약 결과를 검토하고 필요한 경우 Wiki·Jira에 게시합니다. 모델 준비 후 전사·요약에는 외부 AI API가 필요하지 않습니다.

설치 파일에는 로컬 변환용 FFmpeg, 모델 실행 helper, ONNX Runtime, Visual C++ 런타임 및 WebView2 오프라인 설치 프로그램을 포함합니다. 모델 가중치·계정·회의 자료는 포함하지 않습니다. WebView2와 운영체제의 업데이트 정책은 앱과 별도입니다.

## Wiki·Jira 연결

[공통 연결·게시 안내](USAGE.md#add-your-atlassian-connection)를 따릅니다. 회사명에 관계없이 자신의 Atlassian Cloud 사이트·계정 이메일·API 토큰을 사용합니다. Confluence는 페이지 또는 폴더를 상위 대상으로 지정할 수 있고, Jira 보드는 선택사항입니다.

API 토큰은 AES-256-GCM으로 암호화해 앱 데이터 폴더에 저장합니다. 암호화 키는 **현재 Windows 사용자의 자격 증명 관리자**에 보관하고 이 PC에서만 유지합니다. 해당 키를 삭제하거나 다른 PC의 연결 파일만 복사하면 기존 토큰을 복호화할 수 없으므로 연결을 다시 등록해야 합니다. 회의 DB와 녹음 파일 자체의 암호화는 별도 기능입니다.

앱 데이터 기본 경로는 `%APPDATA%\ai.meetily.local`입니다. 녹음 경로는 앱에서 별도 선택할 수 있습니다. 삭제·업데이트 전에 앱을 종료하고 필요한 자료를 백업하세요. 업데이트는 새 설치 파일로 진행하며 자동 업데이트 확인은 하지 않습니다.

## 소스에서 빌드

Windows x64에서 Visual Studio 2022 C++ Desktop Build Tools와 Windows SDK, Rust stable/MSVC, Node.js 22, pnpm **10.11.0**, Python 3.11 이상, CMake, LLVM/libclang, PowerShell 7, MSYS2를 준비합니다. MSYS2 MINGW64 셸에는 GCC, make, nasm, diffutils, tar가 필요합니다. 정확한 CI 구성은 `.github/workflows/windows.yml`에 있습니다.

MSYS2 MINGW64 셸에서 저장소 루트로 이동한 뒤:

```bash
bash scripts/build-ffmpeg-local-windows.sh
```

이후 x64 Visual Studio 개발 환경을 불러온 PowerShell 7에서:

```powershell
./scripts/build-local-windows.ps1
```

결과는 `target/release/bundle/nsis/`입니다. FFmpeg는 Mac과 동일한 공식 소스·SHA-256에서 빌드하며 file/pipe 외 네트워크 프로토콜을 끕니다. ONNX Runtime도 빌드 과정에서 고정 해시를 검증합니다. 실행 시 누락된 실행 파일을 자동으로 내려받지 않습니다.

배포할 때는 설치 파일 외에 FFmpeg 대응 소스 `ffmpeg-8.0.3.tar.xz`, `build-ffmpeg-local-windows.sh`, 해당 빌드의 의존성 고지와 SHA-256 목록도 함께 제공해야 합니다. CI 아티팩트에 함께 모읍니다.

## 검증 범위

CI는 실제 Windows runner에서 Rust/화면 회귀 테스트, 격리된 테스트 자격 증명의 저장·재조회·복호화, 설치 파일 생성과 설치, 주요 자원 포함 여부, helper 표준 입출력 통신, 합성 오디오 변환 및 앱 창 실행을 검사합니다. 실제 Atlassian 계정이나 회의 자료는 사용하지 않습니다.

CI에는 실제 마이크·회의용 헤드셋이 없습니다. 아래 항목은 사용자 장비에서 추가 확인해야 하며, 설치·단위 테스트 성공만으로 검증됐다고 간주하지 않습니다.

- 마이크와 스피커/헤드셋 동시 녹음, 장치 변경·연결 해제
- 한국어 전사 품질과 CPU별 실시간 처리 속도
- Qwen 요약 생성 속도와 긴 회의의 메모리 사용량
- 사용자 계정의 Wiki·Jira 권한과 실제 게시 결과
- Windows 11의 설치·재부팅·업데이트 후 재실행

이상 동작을 제보할 때 Windows 버전, CPU/RAM, 재현 순서와 합성 예시를 적어 주세요. 실제 회의록·API 토큰은 첨부하지 않습니다.
