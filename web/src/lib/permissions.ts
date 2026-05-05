import type { Role } from './api';

/**
 * Page-level access matrix.
 *
 * Each entry maps a route href to the minimum role required. Pages absent
 * from the map are accessible to all signed-in users. The sidebar uses this
 * to hide unauthorized links; the API layer is the source of truth for
 * server-side enforcement.
 */
const PAGE_REQUIREMENTS: Record<string, Role> = {
	'/settings': 'admin'
};

const RANK: Record<Role, number> = {
	viewer: 0,
	member: 1,
	admin: 2
};

export function hasRole(actual: Role | undefined, required: Role): boolean {
	if (!actual) return false;
	return RANK[actual] >= RANK[required];
}

export function canAccessRoute(role: Role | undefined, href: string): boolean {
	const required = PAGE_REQUIREMENTS[href];
	if (!required) return true;
	return hasRole(role, required);
}

export function isAdmin(role: Role | undefined): boolean {
	return hasRole(role, 'admin');
}

export function isMemberOrAbove(role: Role | undefined): boolean {
	return hasRole(role, 'member');
}
