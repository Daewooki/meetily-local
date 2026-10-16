'use client';
import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useRouter } from 'next/navigation';
import { BookOpen, CheckCircle2, ExternalLink, ListTodo, Loader2, Send, ShieldCheck } from 'lucide-react';
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/components/ui/dialog';
import { AtlassianConnection, confluenceParentLabel, PublicationReceipt, integrationField as field, integrationPrimary as primary, integrationSecondary as secondary } from '@/lib/atlassian';

export function AtlassianPublisher({ meetingId, meetingTitle, getMarkdown }: { meetingId: string; meetingTitle: string; getMarkdown: () => Promise<string> }) {
  const router = useRouter();
  const [open,setOpen]=useState(false);
  const [kind,setKind]=useState<'confluence'|'jira'>('confluence');
  const [connections,setConnections]=useState<AtlassianConnection[]>([]);
  const [selected,setSelected]=useState('');
  const [issueType,setIssueType]=useState('');
  const [components,setComponents]=useState<{id:string;name:string}[]>([]);
  const [componentId,setComponentId]=useState('');
  const [componentsBusy,setComponentsBusy]=useState(false);
  const [componentsError,setComponentsError]=useState('');
  const [title,setTitle]=useState('');
  const [body,setBody]=useState('');
  const [approved,setApproved]=useState(false);
  const [busy,setBusy]=useState(false);
  const [error,setError]=useState('');
  const [receipt,setReceipt]=useState<PublicationReceipt|null>(null);
  const connection=connections.find(c=>c.id===selected);

  useEffect(()=>{
    let active=true;
    setComponents([]);setComponentId('');setComponentsError('');setApproved(false);
    if(!open||kind!=='jira'||!selected){setComponentsBusy(false);return;}
    setComponentsBusy(true);
    void invoke<{id:string;name:string}[]>('atlassian_jira_components',{connectionId:selected})
      .then(items=>{if(active)setComponents(items);})
      .catch(e=>{if(active)setComponentsError(String(e));})
      .finally(()=>{if(active)setComponentsBusy(false);});
    return()=>{active=false;};
  },[open,kind,selected]);

  const begin=async (target: 'confluence'|'jira')=>{
    setOpen(true);setKind(target);setBusy(true);setError('');setReceipt(null);setApproved(false);setTitle(meetingTitle);setBody('');setConnections([]);setSelected('');setIssueType('');
    try {
      const [all,markdown]=await Promise.all([invoke<AtlassianConnection[]>('atlassian_connections'), getMarkdown()]);
      const matching=all.filter(c=>c.kind===target);setConnections(matching);setSelected(matching[0]?.id??'');setIssueType(matching[0]?.issueTypes[0]?.id??'');
      setBody(markdown);
      if (!markdown.trim()) setError('게시할 요약이 없습니다. 먼저 요약을 생성하거나 내용을 작성해 주세요.');
      if(target==='jira') setTitle('');
    }catch(e){setError(String(e));}finally{setBusy(false);}
  };
  const publish=async()=>{
    if(!approved||!connection||busy||componentsBusy)return;
    setBusy(true);setError('');
    try { setReceipt(await invoke<PublicationReceipt>('atlassian_publish',{input:{connectionId:selected,meetingId,title,body,issueTypeId:kind==='jira'?issueType:null,componentId:kind==='jira'&&componentId?componentId:null}})); }
    catch(e){setError(String(e));}finally{setBusy(false);}
  };
  return <>
    <div className="flex gap-1.5"><button onClick={()=>void begin('confluence')} className="flex items-center gap-1.5 rounded-md border border-teal-200 bg-teal-50 px-3 py-2 text-xs font-medium text-teal-800"><BookOpen size={14}/> Wiki 게시</button><button onClick={()=>void begin('jira')} className="flex items-center gap-1.5 rounded-md border border-slate-200 px-3 py-2 text-xs font-medium text-slate-700"><ListTodo size={14}/> Jira 등록</button></div>
    <Dialog open={open} onOpenChange={v=>{if(!busy)setOpen(v);}}><DialogContent className="max-h-[90vh] max-w-2xl overflow-y-auto rounded-2xl p-7">
      <div className="flex items-center gap-2 text-[11px] font-semibold tracking-widest text-teal-700"><ShieldCheck size={14}/> REVIEW & PUBLISH</div><DialogTitle>{kind==='confluence'?'Wiki에 회의록 게시':'Jira에 할 일 등록'}</DialogTitle><DialogDescription>아래에서 확인한 제목·본문과 선택한 이슈 설정을 Atlassian Cloud로 전송합니다. 원본 녹음과 전체 전사문은 포함하지 않습니다.</DialogDescription>
      {receipt ? <div className="py-8 text-center"><CheckCircle2 className="mx-auto mb-4 h-12 w-12 text-teal-600"/><h3 className="text-lg font-semibold">{kind==='confluence'?'회의록이 게시되었습니다':'이슈가 등록되었습니다'}</h3><p className="mt-2 text-sm text-slate-500">{receipt.title}</p><button className={`${primary} mt-6`} onClick={()=>void invoke('open_external_url',{url:receipt.url})}>Atlassian에서 열기 <ExternalLink size={15}/></button></div> : <div className="space-y-4">
        {busy && !connections.length ? <div className="flex justify-center py-12"><Loader2 className="animate-spin text-teal-700"/></div> : !connections.length ? <div className="rounded-xl bg-slate-50 p-6 text-center"><p className="mb-4 text-sm text-slate-600">먼저 {kind==='confluence'?'Wiki':'Jira'} 연결을 등록해 주세요.</p><button className={primary} onClick={()=>{setOpen(false);router.push('/connections');}}>연결 관리로 이동</button></div> : <>
          <label className="block space-y-1.5 text-sm font-medium">게시할 연결<select disabled={busy} className={field} value={selected} onChange={e=>{setSelected(e.target.value);setIssueType(connections.find(c=>c.id===e.target.value)?.issueTypes[0]?.id??'');setApproved(false);}}>{connections.map(c=><option key={c.id} value={c.id}>{c.name} · {c.kind==='confluence'?c.spaceKey:c.projectKey}</option>)}</select></label>
          <div className="rounded-lg bg-slate-50 px-3 py-2 text-xs leading-5 text-slate-500">{connection?.siteUrl}<br/>{kind==='confluence'?`Space: ${connection?.spaceKey} · ${connection ? confluenceParentLabel(connection) : ''}`:`Project: ${connection?.projectKey}`}<br/>게시 후에는 해당 Space·Project의 접근 권한이 적용됩니다.</div>
          {kind==='jira' && connection?.boardId && <div className="rounded-lg bg-blue-50 px-3 py-2 text-xs leading-6 text-blue-800"><p>선택한 보드: {connection.boardName} · {connection.boardId}</p>{connection.boardFilter && <p className="break-words">연결 시 확인한 표시 조건: {connection.boardFilter}</p>}{connection.boardNote && <p>{connection.boardNote}</p>}<p>이슈는 프로젝트에 생성됩니다. 보드 표시는 현재 필터·스프린트·상태 조건에 따라 달라지며, 보드에 맞춘 필드 자동 지정은 하지 않습니다.</p></div>}
          {kind==='jira'&&<label className="block space-y-1.5 text-sm font-medium">이슈 유형<select disabled={busy} className={field} value={issueType} onChange={e=>{setIssueType(e.target.value);setApproved(false);}}>{connection?.issueTypes.map(t=><option key={t.id} value={t.id}>{t.name}</option>)}</select></label>}
          {kind==='jira' && <div className="space-y-1.5"><label className="block space-y-1.5 text-sm font-medium">컴포넌트 <span className="font-normal text-slate-400">선택</span><select disabled={busy||componentsBusy} className={field} value={componentId} onChange={e=>{setComponentId(e.target.value);setApproved(false);}}><option value="">{componentsBusy?'불러오는 중…':'지정하지 않음'}</option>{components.map(c=><option key={c.id} value={c.id}>{c.name}</option>)}</select></label><p className="text-xs leading-5 text-slate-500">보드 필터에 component 조건이 있으면 일치하는 컴포넌트를 선택해 주세요.</p>{componentsError && <p role="alert" className="text-xs leading-5 text-amber-700">컴포넌트 조회 실패: {componentsError} 지정 없이 등록하려면 보드 표시 조건을 확인해 주세요.</p>}</div>}
          <label className="block space-y-1.5 text-sm font-medium">{kind==='confluence'?'회의 제목':'할 일 제목'}<input disabled={busy} maxLength={255} className={field} value={title} onChange={e=>{setTitle(e.target.value);setApproved(false);}} placeholder={kind==='jira'?'예: 로그인 오류 재현 조건 확인':''}/></label>
          {kind==='jira'&&<p className="text-xs leading-5 text-slate-500">요약에서 필요한 할 일과 근거를 남겨 주세요. 이 버전은 기본 필드로 이슈를 생성하며 담당자·기한·추가 필수 필드는 Jira에서 확인해 주세요.</p>}
          <label className="block space-y-1.5 text-sm font-medium">게시할 본문<textarea disabled={busy} className={`${field} min-h-[230px] leading-6`} value={body} onChange={e=>{setBody(e.target.value);setApproved(false);}}/></label>
          <label className="flex cursor-pointer items-start gap-2.5 rounded-lg border border-teal-100 bg-teal-50 p-3 text-sm leading-6 text-teal-900"><input type="checkbox" disabled={busy} checked={approved} onChange={e=>setApproved(e.target.checked)} className="mt-1.5 accent-teal-700"/>게시 대상과 내용을 확인했습니다. 이 내용을 Atlassian Cloud에 저장합니다.</label>
        </>}
        {error&&<p role="alert" className="rounded-lg bg-red-50 p-3 text-sm leading-6 text-red-700">{error}</p>}
        <div className="flex justify-end gap-2 border-t pt-4"><button disabled={busy} className={secondary} onClick={()=>setOpen(false)}>취소</button><button disabled={busy||componentsBusy||!approved||!title.trim()||!body.trim()||!connection||(kind==='jira'&&!issueType)} onClick={()=>void publish()} className={primary}>{busy?<Loader2 size={16} className="animate-spin"/>:<Send size={16}/>} {kind==='confluence'?'페이지 게시':'이슈 등록'}</button></div>
      </div>}
    </DialogContent></Dialog>
  </>;
}
