import { useEffect, useState } from 'react';
import { getVersion } from '@tauri-apps/api/app';
import { invoke } from '@tauri-apps/api/core';
import { ExternalLink, LockKeyhole } from 'lucide-react';
import { usePlatform } from '@/hooks/usePlatform';
import { projectRepository, upstreamRepository } from '@/lib/project-links';

export function About() {
  const isWindows = usePlatform() === 'windows';
  const [version,setVersion] = useState('');
  const [error,setError] = useState('');
  useEffect(()=>{void getVersion().then(setVersion);},[]);
  const open=async(url:string)=>{
    try { await invoke('open_external_url',{url}); }
    catch { setError('링크를 열지 못했습니다. 배포 파일에 포함된 사용 안내를 확인해 주세요.'); }
  };
  return <div className="space-y-6 p-6 text-sm leading-7 text-slate-600">
    <div><div className="mb-2 flex items-center gap-2 text-teal-700"><LockKeyhole size={18}/> MEETILY LOCAL</div><h2 className="text-xl font-semibold text-slate-900">Meetily Local</h2><p>v{version}</p></div>
    <p>녹음·한국어 전사·요약은 이 PC에서 처리합니다. 모델 다운로드와 직접 실행한 Atlassian 연결·게시에는 인터넷을 사용합니다.</p>
    <p>외부 AI 서비스, 사용 통계, 자동 업데이트 확인은 비활성화되어 있습니다. 업데이트는 이 배포판의 설치 파일로 진행합니다.</p>
    {projectRepository && <div className="flex flex-wrap gap-3">{[['/releases','배포 파일'],[isWindows ? '/blob/main/docs/WINDOWS.md' : '/blob/main/LOCAL-MAC.md','사용 안내'],['/issues','문제 제보'],['/blob/main/CONTRIBUTING.md','Fork·PR 참여']].map(([path,label])=><button key={path} onClick={()=>void open(projectRepository+path)} className="inline-flex items-center gap-1 rounded-lg border px-3 py-2 text-teal-700">{label}<ExternalLink size={14}/></button>)}</div>}
    <button onClick={()=>void open(upstreamRepository)} className="inline-flex items-center gap-1 text-teal-700">원본 Meetily 프로젝트 <ExternalLink size={14}/></button>
    <div className="border-t pt-4 text-xs leading-6"><p>Meetily Community v0.4.1 기반 · Copyright © 2024 Zackriya Solutions · MIT License</p><p>Meetily Local은 독립적인 커뮤니티 포크입니다. 이 배포판의 문제는 해당 배포 저장소에 제보해 주세요.</p><p>FFmpeg 8.0.3 (LGPL-2.1-or-later)을 별도 실행 파일로 포함합니다. 해당 소스는 같은 릴리스에서 제공하며, 라이선스 고지는 {isWindows ? '설치 폴더의 licenses' : '앱의 Contents/Resources/licenses'}에 있습니다.</p></div>
    {error && <p role="alert" className="text-red-700">{error}</p>}
  </div>;
}
