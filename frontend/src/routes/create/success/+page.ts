import type { PageLoad } from './$types';

export const load: PageLoad = ({ url }) => {
    const repo = url.searchParams.get('repo') ?? '';
    return { repo };
};
