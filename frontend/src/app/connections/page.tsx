'use client';

import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { BookOpen, Check, ChevronRight, ExternalLink, KeyRound, Link2, ListTodo, Loader2, LockKeyhole, Plus, ShieldCheck, Trash2 } from 'lucide-react';
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/components/ui/dialog';
import { AtlassianConnection, confluenceParentLabel, integrationField as field, integrationPrimary as primary, integrationSecondary as secondary } from '@/lib/atlassian';

const blank = (kind: 'confluence' | 'jira') => ({ kind, name: '', siteUrl: '', email: '', token: '', tokenType: 'classic', spaceKey: '', parentPageId: '', projectKey: '', boardId: '' });

export default function ConnectionsPage() {
  const [connections, setConnections] = useState<AtlassianConnection[]>([]);
  const [loading, setLoading] = useState(true);
  const [open, setOpen] = useState(false);
  const [form, setForm] = useState(blank('confluence'));
  const [busy, setBusy] = useState<'test' | 'save' | null>(null);
  const [error, setError] = useState('');
  const [pageError, setPageError] = useState('');
  const [tested, setTested] = useState(false);
  const [verified, setVerified] = useState<AtlassianConnection | null>(null);
  const [removeId, setRemoveId] = useState<string | null>(null);
  const [deleting, setDeleting] = useState(false);

  const refresh = async () => {
    try { setConnections(await invoke<AtlassianConnection[]>('atlassian_connections')); setPageError(''); }
    catch (e) { setPageError(String(e)); }
    finally { setLoading(false); }
  };
  useEffect(() => { void refresh(); }, []);

  const create = (kind: 'confluence' | 'jira') => { setForm(blank(kind)); setError(''); setTested(false); setVerified(null); setOpen(true); };
  const close = () => { if (!busy) { setOpen(false); setForm(blank('confluence')); setError(''); setTested(false); } };
  const change = (key: keyof ReturnType<typeof blank>, value: string) => { setForm(p => ({ ...p, [key]: value })); setTested(false); setVerified(null); setError(''); };
  const valid = !!(form.name.trim() && form.siteUrl.trim() && form.email.trim() && form.token.trim() && (form.kind === 'confluence' ? form.spaceKey.trim() : form.projectKey.trim()));

  const run = async (mode: 'test' | 'save') => {
    if (busy) return;
    setBusy(mode); setError('');
    try {
      const result = await invoke<AtlassianConnection>(mode === 'test' ? 'atlassian_test_connection' : 'atlassian_save_connection', { input: form });
      if (mode === 'test') { setTested(true); setVerified(result); }
      else { setForm(blank('confluence')); setOpen(false); setTested(false); await refresh(); }
    } catch (e) { setError(String(e)); setTested(false); }
    finally { setBusy(null); }
  };

  return <main className="h-screen overflow-y-auto bg-[#f7f9fa] text-slate-900">
    <div className="mx-auto max-w-5xl px-10 py-10">
      <div className="mb-8 flex items-center gap-2 text-xs font-semibold tracking-widest text-teal-700"><LockKeyhole size={15} /> MEETILY LOCAL <ChevronRight size={13} /> 연결 관리</div>
      <div className="flex items-start justify-between gap-6">
        <div><h1 className="text-3xl font-semibold tracking-tight">회의의 다음 단계를 연결하세요</h1><p className="mt-3 text-sm leading-7 text-slate-500">회의는 이 Mac에서 정리하고, 확인한 결과만 팀에 공유합니다.</p></div>
        <span className="flex shrink-0 items-center gap-2 rounded-full border border-teal-200 bg-teal-50 px-3 py-2 text-xs font-medium text-teal-800"><ShieldCheck size={15} /> 로컬 처리</span>
      </div>

      <div className="my-8 grid grid-cols-3 gap-4 rounded-2xl border border-slate-200 bg-white p-6 text-sm">
        {[['01', '녹음·전사·요약', '이 Mac 안에서 처리'], ['02', '결과 검토', '공유할 내용 직접 확인'], ['03', '선택한 내용 게시', 'Atlassian Cloud에 저장']].map(([n, title, desc]) => <div key={n} className="flex gap-3"><span className="text-xs font-semibold text-teal-600">{n}</span><div><p className="font-semibold">{title}</p><p className="mt-1.5 text-xs text-slate-500">{desc}</p></div></div>)}
      </div>

      <div className="flex items-center justify-between"><h2 className="text-lg font-semibold">내 연결 <span className="ml-2 text-sm font-normal text-slate-400">{connections.length}</span></h2><div className="flex gap-2"><button onClick={() => create('confluence')} className={primary}><Plus size={16} /> Wiki 연결</button><button onClick={() => create('jira')} className={secondary}><Plus size={16} /> Jira 연결</button></div></div>
      {pageError && <div role="alert" className="mt-5 rounded-lg bg-red-50 p-4 text-sm text-red-700">{pageError}<button onClick={() => void refresh()} className="ml-3 underline">다시 불러오기</button></div>}
      {loading ? <div className="flex justify-center py-20"><Loader2 className="animate-spin text-teal-700" /></div> : connections.length === 0 ? <div className="mt-5 rounded-2xl border border-dashed border-slate-300 bg-white px-8 py-14 text-center"><div className="mx-auto mb-4 flex h-14 w-14 items-center justify-center rounded-2xl bg-slate-100"><Link2 className="text-slate-500" /></div><h3 className="font-semibold">아직 등록된 연결이 없습니다</h3><p className="mx-auto mt-2 max-w-sm text-sm leading-6 text-slate-500">Atlassian 계정의 API 토큰을 등록해 주세요.<br />회의 요약 화면에서 게시할 연결을 선택할 수 있습니다.</p><button onClick={() => create('confluence')} className={`${primary} mt-6`}><Plus size={16} /> 첫 Wiki 연결 추가</button></div> : <div className="mt-5 grid gap-4">{connections.map(c => <article key={c.id} className="flex items-center gap-5 rounded-2xl border border-slate-200 bg-white p-6"><div className={`rounded-xl p-3 ${c.kind === 'confluence' ? 'bg-blue-50 text-blue-600' : 'bg-violet-50 text-violet-600'}`}>{c.kind === 'confluence' ? <BookOpen size={24} /> : <ListTodo size={24} />}</div><div className="min-w-0 flex-1"><div className="flex items-center gap-3"><h3 className="font-semibold">{c.name}</h3><span className="rounded bg-slate-100 px-2 py-1 text-[11px] text-slate-600">{c.kind === 'confluence' ? 'Confluence' : 'Jira'}</span></div><p className="mt-1 truncate text-sm text-slate-500">{c.siteUrl}</p><p className="mt-2 text-xs text-slate-500">{c.kind === 'confluence' ? `Space ${c.spaceKey} · ${confluenceParentLabel(c)}` : `Project ${c.projectKey}${c.boardId ? ` · Board ${c.boardId} (${c.boardName || ""})` : ""}`} · {c.email}</p></div><span className="flex items-center gap-1.5 text-xs text-teal-700"><LockKeyhole size={14} /> 토큰 암호화됨</span><button aria-label={`${c.name} 연결 삭제`} onClick={() => setRemoveId(c.id)} className="rounded-lg p-2 text-slate-400 hover:bg-red-50 hover:text-red-600"><Trash2 size={17} /></button></article>)}</div>}
      <div className="mt-8 flex items-start gap-3 rounded-xl bg-slate-100 p-5 text-xs leading-6 text-slate-500"><KeyRound size={18} className="mt-0.5 shrink-0" /><p>API 토큰은 AES-256-GCM으로 암호화해 이 Mac에 저장하고, 암호화 키는 macOS Keychain에 보관합니다. 원본 녹음과 전체 전사문은 게시 요청에 포함하지 않습니다. 연결 확인은 조회만 수행하며, 작성 권한은 실제 게시 시 확인됩니다.</p></div>
    </div>

    <Dialog open={open} onOpenChange={v => { if (!v) close(); }}>
      <DialogContent className="max-h-[90vh] max-w-xl overflow-y-auto rounded-2xl p-7">
        <div className="mb-1 flex items-center gap-2 text-[11px] font-semibold tracking-[0.2em] text-teal-700"><ShieldCheck size={14} /> SECURE CONNECTION</div>
        <DialogTitle className="text-2xl">새 {form.kind === 'confluence' ? 'Wiki' : 'Jira'} 연결</DialogTitle>
        <DialogDescription>Atlassian Cloud 계정을 연결합니다.</DialogDescription>
        <form onSubmit={e => { e.preventDefault(); if (tested) void run('save'); }} className="mt-3 space-y-4">
          <label className="block space-y-1.5 text-sm font-medium">연결 이름 <span className="text-teal-700">*</span><input autoFocus disabled={!!busy} required maxLength={100} className={field} value={form.name} onChange={e => change('name',e.target.value)} placeholder="예: 개발팀 회의록" /></label>
          <label className="block space-y-1.5 text-sm font-medium">사이트 URL <span className="text-teal-700">*</span><input disabled={!!busy} type="url" required className={field} value={form.siteUrl} onChange={e => change('siteUrl',e.target.value)} placeholder="https://company.atlassian.net" /></label>
          {form.kind === 'confluence' ? <div className="grid grid-cols-2 gap-4"><label className="space-y-1.5 text-sm font-medium">Space Key <span className="text-teal-700">*</span><input disabled={!!busy} required className={field} value={form.spaceKey} onChange={e => change('spaceKey',e.target.value)} placeholder="TEAM" /></label><label className="space-y-1.5 text-sm font-medium">상위 페이지·폴더 ID <span className="font-normal text-slate-400">선택</span><input disabled={!!busy} inputMode="numeric" className={field} value={form.parentPageId} onChange={e => change('parentPageId',e.target.value)} placeholder="123456789" /></label></div> : <label className="block space-y-1.5 text-sm font-medium">Project Key <span className="text-teal-700">*</span><input disabled={!!busy} required className={field} value={form.projectKey} onChange={e => change('projectKey',e.target.value)} placeholder="PROJ" /></label>}
          {form.kind === 'confluence' && <p className="text-xs leading-5 text-slate-500">상위 페이지 또는 폴더 주소의 /pages/ 또는 /folder/ 뒤 숫자를 입력하세요. 종류는 자동 확인합니다. 비워 두면 공간 최상위에 새 페이지를 만듭니다.</p>}
          {form.kind === 'jira' && <label className="block space-y-1.5 text-sm font-medium">Board ID <span className="font-normal text-slate-400">선택</span><input disabled={!!busy} inputMode="numeric" pattern="[0-9]*" className={field} value={form.boardId} onChange={e => change('boardId',e.target.value)} placeholder="예: 1234" /><span className="block text-xs font-normal leading-5 text-slate-500">보드 주소에서 /boards/ 뒤 숫자입니다. Project Key와 별개이며, 보드의 표시 조건을 함께 확인합니다.</span></label>}
          <p className="rounded-lg bg-blue-50 px-3 py-2.5 text-xs leading-5 text-blue-800">{form.kind === 'confluence' ? '회의 제목은 Wiki 페이지 제목으로, 검토한 AI 요약·핵심 내용·결정·할 일은 페이지 본문으로 게시됩니다.' : '검토한 할 일을 이슈로 등록합니다. 등록할 때 이슈 유형과 제목·설명을 직접 확인합니다.'}</p>
          <label className="block space-y-1.5 text-sm font-medium">사용자 이메일 <span className="text-teal-700">*</span><input disabled={!!busy} type="email" autoComplete="username" required className={field} value={form.email} onChange={e => change('email',e.target.value)} placeholder="name@company.com" /></label>
          <label className="block space-y-1.5 text-sm font-medium">토큰 종류<select disabled={!!busy} className={field} value={form.tokenType} onChange={e => change('tokenType',e.target.value)}><option value="classic">일반 API 토큰 · scope 없음</option><option value="scoped">범위 지정 API 토큰 · scope 있음</option></select></label>
          <div><div className="mb-1.5 flex items-center justify-between"><label htmlFor="atlassian-token" className="text-sm font-medium">API 토큰 <span className="text-teal-700">*</span></label><button type="button" className="flex items-center gap-1 text-xs text-teal-700" onClick={() => void invoke('open_external_url',{url:'https://id.atlassian.com/manage-profile/security/api-tokens'})}>API Token 발급 <ExternalLink size={12} /></button></div><input id="atlassian-token" disabled={!!busy} type="password" required autoComplete="new-password" spellCheck={false} className={field} value={form.token} onChange={e => change('token',e.target.value)} placeholder="발급받은 API 토큰 붙여넣기" /></div>
          <p className="flex items-start gap-2 text-xs leading-5 text-slate-500"><LockKeyhole size={13} className="mt-0.5 shrink-0" />토큰은 AES-256-GCM으로 암호화되어 저장되며 저장 후 다시 표시되지 않습니다.</p>
          {tested && <p role="status" className="flex items-center gap-2 rounded-lg bg-teal-50 p-3 text-sm text-teal-800"><Check size={16} /> 연결 확인 완료. 이제 이 Mac에 저장할 수 있습니다.</p>}
          {tested && verified?.kind === 'confluence' && <p className="rounded-lg bg-slate-50 p-3 text-xs leading-6 text-slate-600">게시 위치: {confluenceParentLabel(verified)}</p>}
          {tested && verified?.boardId && <div className="rounded-lg bg-slate-50 p-3 text-xs leading-6 text-slate-600"><p className="font-semibold">보드: {verified.boardName} · {verified.boardId}</p>{verified.boardFilter && <p className="break-words">표시 조건: {verified.boardFilter}</p>}{verified.boardNote && <p>{verified.boardNote}</p>}<p>이슈는 프로젝트에 생성되며, 보드 필터·스프린트·상태 조건에 맞아야 보드에 표시됩니다.</p></div>}
          {error && <p role="alert" className="rounded-lg bg-red-50 p-3 text-sm leading-6 text-red-700">{error}</p>}
          <div className="flex justify-end gap-2 border-t border-slate-100 pt-4"><button type="button" disabled={!valid || !!busy} className={secondary} onClick={() => void run('test')}>{busy === 'test' ? <Loader2 size={16} className="animate-spin" /> : <Link2 size={16} />} 연결 확인</button><button type="submit" disabled={!tested || !!busy} className={primary}>{busy === 'save' ? <Loader2 size={16} className="animate-spin" /> : <Plus size={16} />} 연결 추가</button></div>
        </form>
      </DialogContent>
    </Dialog>
    <Dialog open={!!removeId} onOpenChange={v => { if (!v && !deleting) setRemoveId(null); }}><DialogContent><DialogTitle>이 Mac에서 연결을 삭제할까요?</DialogTitle><DialogDescription>저장된 API 토큰을 제거합니다. 이미 게시한 페이지와 이슈는 유지됩니다. 토큰 자체 폐기는 Atlassian 계정에서 할 수 있습니다.</DialogDescription><div className="flex justify-end gap-2"><button disabled={deleting} className={secondary} onClick={() => setRemoveId(null)}>취소</button><button disabled={deleting} className={primary} onClick={async () => { setDeleting(true); try { await invoke('atlassian_delete_connection',{id:removeId}); setRemoveId(null); await refresh(); } catch(e) { setPageError(String(e)); } finally { setDeleting(false); } }}>{deleting ? '삭제 중…' : '연결 삭제'}</button></div></DialogContent></Dialog>
  </main>;
}
