'use client';
import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { usePlatform } from '@/hooks/usePlatform';
import { Check, Download, HardDrive, Loader2, LockKeyhole, Mic, Volume2 } from 'lucide-react';
import { integrationPrimary as primary, integrationSecondary as secondary } from '@/lib/atlassian';

const WHISPER='large-v3-turbo-q5_0';
const SUMMARY='qwen3.5:4b';
type Model={name:string;status:string|{type:string}};

export function LocalSetup({onComplete}:{onComplete:()=>void}) {
  const platform = usePlatform();
  const isWindows = platform === 'windows';
  const permissionHelp = isWindows ? 'Windows 설정 → 개인정보 및 보안 → 마이크에서 마이크 액세스와 데스크톱 앱의 마이크 액세스를 허용해 주세요.' : '시스템 설정 → 개인정보 보호 및 보안에서 마이크 및 화면·시스템 오디오 기록 권한을 확인해 주세요.';
  const [ready,setReady]=useState({whisper:false,summary:false});
  const [progress,setProgress]=useState({whisper:0,summary:0});
  const [busy,setBusy]=useState('');
  const [error,setError]=useState('');
  const [permission,setPermission]=useState({mic:false,audio:false});
  const refresh=async()=>{
    await invoke('whisper_init');
    const [w,s]=await Promise.all([invoke<Model[]>('whisper_get_available_models'),invoke<Model[]>('builtin_ai_list_models')]);
    setReady({whisper:w.some(m=>m.name===WHISPER&&m.status==='Available'),summary:s.some(m=>m.name===SUMMARY&&typeof m.status==='object'&&m.status.type==='available')});
  };
  useEffect(()=>{
    let disposed=false;const cleanup:(()=>void)[]=[];
    const setup=async()=>{
      const listeners=await Promise.all([
        listen<{modelName:string;progress:number}>('model-download-progress',e=>{if(e.payload.modelName===WHISPER)setProgress(p=>({...p,whisper:e.payload.progress}));}),
        listen<{model:string;progress:number}>('builtin-ai-download-progress',e=>{if(e.payload.model===SUMMARY)setProgress(p=>({...p,summary:e.payload.progress}));})
      ]);
      if(disposed){listeners.forEach(f=>f());return;}cleanup.push(...listeners);
      try{await refresh();}catch{if(!disposed)setError('모델 상태를 읽지 못했습니다. 앱 준비가 끝난 뒤 다시 확인해 주세요.');}
    };void setup();return()=>{disposed=true;cleanup.forEach(f=>f());};
  },[]);
  const download=async()=>{
    setError('');setBusy('download');
    try{
      await invoke('whisper_init');
      if(!ready.whisper)await invoke('whisper_download_model',{modelName:WHISPER});
      await refresh();
      if(!ready.summary)await invoke('builtin_ai_download_model',{modelName:SUMMARY});
      await refresh();
    }catch(e){setError(String(e));}finally{setBusy('');}
  };
  const finish=async()=>{
    setBusy('finish');setError('');
    try{
      await invoke('complete_onboarding',{model:SUMMARY});
      localStorage.setItem('primaryLanguage','ko');localStorage.setItem('summaryLanguageDefault','ko');
      localStorage.setItem('analytics_consent','false');
      await invoke('set_language_preference',{language:'ko'});
      onComplete();
    }catch(e){setError(String(e));setBusy('');}
  };
  const request=async(type:'mic'|'audio')=>{
    setBusy(type);setError('');
    try{const granted=await invoke<boolean>(type==='mic'?'trigger_microphone_permission':'trigger_system_audio_permission_command');setPermission(p=>({...p,[type]:granted}));if(!granted)setError(permissionHelp);}
    catch{setError(permissionHelp);}finally{setBusy('');}
  };
  return <main className="flex min-h-screen w-full items-center justify-center overflow-y-auto bg-[#f7f9fa] px-6 py-10 text-slate-900"><div className="w-full max-w-2xl">
    <div className="mb-5 flex items-center gap-2 text-xs font-semibold tracking-[0.2em] text-teal-700"><LockKeyhole size={16}/> MEETILY LOCAL</div>
    <h1 className="text-3xl font-semibold tracking-tight">내 PC에서 시작하는 회의록</h1><p className="mt-3 text-sm leading-7 text-slate-500">한국어 전사와 AI 요약을 로컬에서 처리합니다.<br/>공유할 내용은 확인한 뒤 Confluence·Jira에 게시하세요.</p>
    <div className="mt-7 rounded-2xl border border-slate-200 bg-white p-6"><div className="mb-5 flex items-center gap-2 font-semibold"><HardDrive size={18}/> 로컬 AI 준비</div>
      {[{key:'whisper' as const,name:'한국어 전사',model:'Whisper large-v3-turbo · 약 0.6GB'},{key:'summary' as const,name:'회의 요약',model:'Qwen 3.5 4B · 약 2.7GB'}].map(m=><div key={m.key} className="mb-4 rounded-xl bg-slate-50 p-4"><div className="flex items-center justify-between"><div><p className="text-sm font-semibold">{m.name}</p><p className="mt-1 text-xs text-slate-500">{m.model}</p></div>{ready[m.key]?<span className="flex items-center gap-1 text-xs text-teal-700"><Check size={15}/> 준비 완료</span>:<span className="text-xs text-slate-500">{progress[m.key]>0?`${progress[m.key]}%`:'다운로드 필요'}</span>}</div>{!ready[m.key]&&busy==='download'&&<div className="mt-3 h-1.5 overflow-hidden rounded-full bg-slate-200"><div className="h-full bg-teal-600 transition-all" style={{width:`${progress[m.key]}%`}}/></div>}</div>)}
      <p className="mb-4 text-xs leading-5 text-slate-500">최초 모델 다운로드에는 인터넷이 필요합니다. 모델 준비 후 전사·요약에는 인터넷이나 AI 서비스 API 키가 필요하지 않습니다.</p>
      <div className="flex gap-2"><button disabled={!!busy||(ready.whisper&&ready.summary)} className={primary} onClick={()=>void download()}>{busy==='download'?<Loader2 size={16} className="animate-spin"/>:<Download size={16}/>} {ready.whisper&&ready.summary?'모델 준비 완료':'로컬 모델 다운로드'}</button><button disabled={!!busy} className={secondary} onClick={()=>{setError('');void refresh().catch(()=>setError('모델 상태 확인에 실패했습니다.'));}}>상태 확인</button></div>
    </div>
    <div className="mt-4 rounded-2xl border border-slate-200 bg-white p-6"><p className="mb-4 text-sm font-semibold">녹음 권한</p><div className="flex flex-wrap gap-2"><button disabled={!!busy} className={secondary} onClick={()=>void request('mic')}><Mic size={16}/>{permission.mic?'마이크 허용됨':'마이크 권한 요청'}</button>{!isWindows && <button disabled={!!busy} className={secondary} onClick={()=>void request('audio')}><Volume2 size={16}/>{permission.audio?'시스템 오디오 확인됨':'시스템 오디오 권한 요청'}</button>}</div><p className="mt-3 text-xs leading-5 text-slate-500">{isWindows ? '온라인 회의의 상대방 소리를 담으려면 녹음 화면에서 사용 중인 출력 장치를 시스템 오디오로 선택하세요. 별도 화면 기록 권한은 필요하지 않습니다.' : '온라인 회의의 상대방 소리를 담으려면 시스템 오디오 권한도 필요합니다.'} 녹음 파일 가져오기는 녹음 권한 없이 사용할 수 있습니다.</p></div>
    {error&&<p role="alert" className="mt-4 rounded-lg bg-red-50 p-4 text-sm leading-6 text-red-700">{error}</p>}
    <div className="mt-6 flex items-center justify-between gap-4"><p className="text-xs text-slate-400">Meetily Community 기반 Meetily Local</p><button disabled={!!busy} className={primary} onClick={()=>void finish()}>{busy==='finish'&&<Loader2 size={15} className="animate-spin"/>}{ready.whisper&&ready.summary?'앱 시작하기':'연결부터 등록하기 →'}</button></div>
  </div></main>;
}
