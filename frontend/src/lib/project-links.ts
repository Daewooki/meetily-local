// Configure the independently maintained distribution at build time.
export const projectRepository = (
  process.env.NEXT_PUBLIC_PROJECT_REPOSITORY || 'https://github.com/Daewooki/meetily-local'
).replace(/\/+$/, '');
export const upstreamRepository = 'https://github.com/Zackriya-Solutions/meetily';
