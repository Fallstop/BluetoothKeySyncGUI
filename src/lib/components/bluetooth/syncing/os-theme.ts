import type { HostDistributions } from '#root/bindings';

export const OS_COLORS = {
	Windows: {
		text: 'text-blue-600 dark:text-blue-400',
		borderL: 'border-l-blue-500',
		borderR: 'border-r-blue-500',
		badge: 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300',
		hex: '#3b82f6',
		ringActive: 'ring-blue-400/40',
		ringHover: 'ring-blue-400/60'
	},
	Linux: {
		text: 'text-orange-600 dark:text-orange-400',
		borderL: 'border-l-orange-500',
		borderR: 'border-r-orange-500',
		badge: 'bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-300',
		hex: '#f97316',
		ringActive: 'ring-orange-400/40',
		ringHover: 'ring-orange-400/60'
	}
} as const;

export function osColor(os: HostDistributions) {
	return OS_COLORS[os];
}
