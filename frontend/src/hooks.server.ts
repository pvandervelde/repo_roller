import type { Handle } from '@sveltejs/kit';

// Auth guard stub — full implementation in task 13.6.
// In this stub all requests are passed through and session is always null.
export const handle: Handle = async ({ event, resolve }) => {
  event.locals.session = null;
  return resolve(event);
};
