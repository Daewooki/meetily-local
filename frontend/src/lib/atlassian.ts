export interface AtlassianConnection {
  id: string;
  kind: 'confluence' | 'jira';
  name: string;
  siteUrl: string;
  email: string;
  tokenType: 'classic' | 'scoped';
  cloudId: string | null;
  spaceKey: string;
  spaceId: string | null;
  parentPageId: string;
  parentType?: 'page' | 'folder' | null;
  parentTitle?: string | null;
  projectKey: string;
  boardId: string;
  boardName: string | null;
  boardFilter: string | null;
  boardNote: string | null;
  issueTypes: { id: string; name: string }[];
}

export interface PublicationReceipt {
  key: string;
  meetingId: string;
  connectionId: string;
  kind: 'confluence' | 'jira';
  title: string;
  url: string | null;
  status: 'published' | 'needs_review';
  createdAt: string;
}

export const integrationField = 'w-full rounded-lg border border-slate-200 bg-white px-3 py-2.5 text-sm text-slate-900 outline-none focus:border-teal-600 focus:ring-2 focus:ring-teal-100 disabled:bg-slate-50';
export const integrationPrimary = 'inline-flex items-center justify-center gap-2 rounded-lg bg-teal-700 px-4 py-2.5 text-sm font-semibold text-white hover:bg-teal-800 disabled:cursor-not-allowed disabled:opacity-40';
export const integrationSecondary = 'inline-flex items-center justify-center gap-2 rounded-lg border border-slate-200 bg-white px-4 py-2.5 text-sm font-medium text-slate-700 hover:bg-slate-50 disabled:opacity-40';

export function confluenceParentLabel(connection: AtlassianConnection): string {
  if (!connection.parentPageId) return '공간 최상위';
  const kind = connection.parentType === 'folder' ? '폴더' : '페이지';
  return `${kind} ${connection.parentTitle ? `${connection.parentTitle} · ` : ''}${connection.parentPageId}`;
}
