import type { HostDistributions } from '#root/bindings';

export const OS_COLORS = {
	Windows: {
		hex: '#3b82f6',
		textColor: '#60a5fa',
		borderColor: '#3b82f6',
		ringColor: 'rgba(59, 130, 246, 0.4)',
		ringHoverColor: 'rgba(59, 130, 246, 0.6)',
		badgeBg: 'rgba(59, 130, 246, 0.1)',
		badgeColor: '#93c5fd',
		pillBg: 'rgba(59, 130, 246, 0.12)',
		pillBorder: 'rgba(59, 130, 246, 0.25)',
		pillText: '#93c5fd',
		accentGradient: 'linear-gradient(135deg, rgba(59, 130, 246, 0.15), rgba(59, 130, 246, 0.05))'
	},
	Linux: {
		hex: '#f97316',
		textColor: '#fb923c',
		borderColor: '#f97316',
		ringColor: 'rgba(249, 115, 22, 0.4)',
		ringHoverColor: 'rgba(249, 115, 22, 0.6)',
		badgeBg: 'rgba(249, 115, 22, 0.1)',
		badgeColor: '#fdba74',
		pillBg: 'rgba(249, 115, 22, 0.12)',
		pillBorder: 'rgba(249, 115, 22, 0.25)',
		pillText: '#fdba74',
		accentGradient: 'linear-gradient(135deg, rgba(249, 115, 22, 0.15), rgba(249, 115, 22, 0.05))'
	}
} as const;

export function osColor(os: HostDistributions) {
	return OS_COLORS[os];
}
