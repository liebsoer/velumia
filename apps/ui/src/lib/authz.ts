import { api } from "./api";

type AuthzResponse = { allowed: boolean | string; reason?: string; permission?: string };

export function isAuthzAllowed(result: AuthzResponse): boolean {
  return result.allowed === true || result.allowed === "true";
}

export async function can(action: string): Promise<boolean> {
  const result = await api.checkAuthorize(action);
  return isAuthzAllowed(result as AuthzResponse);
}
