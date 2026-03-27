import type { BrandConfig } from '$lib/types/brand';
import type { Session } from '$lib/types/session';

declare global {
  namespace App {
    interface Locals {
      session: Session | null;
    }
    interface PageData {
      brandConfig: BrandConfig;
      session: Session | null;
    }
    // interface Error {}
    // interface Platform {}
  }
}

export {};
