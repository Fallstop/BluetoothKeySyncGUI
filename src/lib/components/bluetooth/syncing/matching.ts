import type {
	BluetoothController,
	BluetoothData,
	BluetoothDevice,
	BluetoothLinkKey,
	BluetoothLowEnergyKey,
	HostDistributions
} from '#root/bindings';

// Mirrors Rust SyncProposal/SyncRequest/SyncResult in sync_api.rs
// These aren't in the auto-generated bindings yet
export type SyncProposal = {
	source_device: BluetoothDevice;
	target_device: BluetoothDevice;
	source_os: HostDistributions;
	target_os: HostDistributions;
	action: string;
};

export type SyncRequest = {
	proposals: SyncProposal[];
};

export type SyncResult = {
	success: boolean;
	applied_count: number;
	failed_count: number;
	errors: string[];
};

// --- Types ---

export type SyncDirection = 'win_to_linux' | 'linux_to_win';

export type KeyFieldStatus = 'match' | 'mismatch' | 'source_only' | 'target_only' | 'both_missing';

export interface KeyFieldComparison {
	field: string;
	label: string;
	windowsValue: string | null;
	linuxValue: string | null;
	status: KeyFieldStatus;
}

export interface KeyComparison {
	linkKey: KeyFieldComparison[];
	leData: KeyFieldComparison[];
	overallStatus: 'synced' | 'needs_sync' | 'no_keys';
}

export interface MatchedDevicePair {
	windowsDevice: BluetoothDevice;
	linuxDevice: BluetoothDevice;
	controllerAddress: string;
	comparison: KeyComparison;
	recommendedDirection: SyncDirection;
}

export interface UnmatchedDevice {
	device: BluetoothDevice;
	os: HostDistributions;
	controllerAddress: string;
}

export interface ControllerMatch {
	controllerAddress: string;
	controllerName: string | null;
	windowsController: BluetoothController | null;
	linuxController: BluetoothController | null;
}

export interface MatchResult {
	needsSync: MatchedDevicePair[];
	alreadySynced: MatchedDevicePair[];
	unmatched: UnmatchedDevice[];
	controllerMatches: ControllerMatch[];
}

export interface SyncSelection {
	enabled: boolean;
	direction: SyncDirection;
}

// Key for SyncSelections map: `${controllerAddress}/${deviceAddress}`
export type SyncSelections = Map<string, SyncSelection>;

// --- Comparison Functions ---

function compareField(
	field: string,
	label: string,
	windowsValue: string | number | null | undefined,
	linuxValue: string | number | null | undefined
): KeyFieldComparison {
	const wVal = windowsValue != null ? String(windowsValue) : null;
	const lVal = linuxValue != null ? String(linuxValue) : null;

	let status: KeyFieldStatus;
	if (wVal === null && lVal === null) {
		status = 'both_missing';
	} else if (wVal !== null && lVal === null) {
		status = 'source_only';
	} else if (wVal === null && lVal !== null) {
		status = 'target_only';
	} else if (wVal === lVal) {
		status = 'match';
	} else {
		status = 'mismatch';
	}

	return { field, label, windowsValue: wVal, linuxValue: lVal, status };
}

export function compareLinkKeys(
	windowsKey: BluetoothLinkKey | null,
	linuxKey: BluetoothLinkKey | null
): KeyFieldComparison[] {
	return [compareField('key', 'Link Key', windowsKey?.key, linuxKey?.key)];
}

export function compareLeData(
	windowsLe: BluetoothLowEnergyKey | null,
	linuxLe: BluetoothLowEnergyKey | null
): KeyFieldComparison[] {
	return [
		compareField('long_term_key', 'Long Term Key (LTK)', windowsLe?.long_term_key, linuxLe?.long_term_key),
		compareField(
			'identity_resolving_key',
			'Identity Resolving Key (IRK)',
			windowsLe?.identity_resolving_key,
			linuxLe?.identity_resolving_key
		),
		compareField(
			'local_signature_key',
			'Signature Key (CSRK)',
			windowsLe?.local_signature_key,
			linuxLe?.local_signature_key
		),
		compareField('ediv', 'EDIV', windowsLe?.ediv, linuxLe?.ediv),
		compareField('rand', 'Rand', windowsLe?.rand, linuxLe?.rand),
		compareField('key_length', 'Key Length', windowsLe?.key_length, linuxLe?.key_length)
	];
}

export function compareKeys(windowsDevice: BluetoothDevice, linuxDevice: BluetoothDevice): KeyComparison {
	const linkKey = compareLinkKeys(windowsDevice.link_key, linuxDevice.link_key);
	const leData = compareLeData(windowsDevice.le_data, linuxDevice.le_data);

	const allFields = [...linkKey, ...leData];
	const hasAnyKey = allFields.some((f) => f.status !== 'both_missing');
	const allMatch = allFields.every((f) => f.status === 'match' || f.status === 'both_missing');

	let overallStatus: KeyComparison['overallStatus'];
	if (!hasAnyKey) {
		overallStatus = 'no_keys';
	} else if (allMatch) {
		overallStatus = 'synced';
	} else {
		overallStatus = 'needs_sync';
	}

	return { linkKey, leData, overallStatus };
}

function countKeyFields(device: BluetoothDevice): number {
	let count = 0;
	if (device.link_key?.key) count++;
	if (device.le_data) {
		if (device.le_data.long_term_key) count += 4; // LTK is most important
		if (device.le_data.identity_resolving_key) count += 2;
		if (device.le_data.local_signature_key) count++;
		if (device.le_data.key_length != null) count++;
		if (device.le_data.ediv) count++;
		if (device.le_data.rand) count++;
	}
	return count;
}

export function recommendDirection(
	windowsDevice: BluetoothDevice,
	linuxDevice: BluetoothDevice
): SyncDirection {
	const winCount = countKeyFields(windowsDevice);
	const linCount = countKeyFields(linuxDevice);
	// More complete side is source; tie-break favors Windows
	return linCount > winCount ? 'linux_to_win' : 'win_to_linux';
}

// --- Matching Functions ---

export function matchDevicesForController(
	windowsController: BluetoothController | null,
	linuxController: BluetoothController | null,
	controllerAddress: string
): { paired: MatchedDevicePair[]; unmatched: UnmatchedDevice[] } {
	const paired: MatchedDevicePair[] = [];
	const unmatched: UnmatchedDevice[] = [];

	const winDevices = windowsController?.devices ?? [];
	const linDevices = linuxController?.devices ?? [];

	// Build MAC -> device map for Linux
	const linuxByMac = new Map<string, BluetoothDevice>();
	for (const dev of linDevices) {
		linuxByMac.set(dev.address.toUpperCase(), dev);
	}

	const matchedLinuxMacs = new Set<string>();

	for (const winDev of winDevices) {
		const mac = winDev.address.toUpperCase();
		const linDev = linuxByMac.get(mac);

		if (linDev) {
			matchedLinuxMacs.add(mac);
			const comparison = compareKeys(winDev, linDev);
			paired.push({
				windowsDevice: winDev,
				linuxDevice: linDev,
				controllerAddress,
				comparison,
				recommendedDirection: recommendDirection(winDev, linDev)
			});
		} else {
			unmatched.push({ device: winDev, os: 'Windows', controllerAddress });
		}
	}

	for (const linDev of linDevices) {
		if (!matchedLinuxMacs.has(linDev.address.toUpperCase())) {
			unmatched.push({ device: linDev, os: 'Linux', controllerAddress });
		}
	}

	return { paired, unmatched };
}

export function matchAllDevices(
	windows: BluetoothData | null,
	linux: BluetoothData | null
): MatchResult {
	const result: MatchResult = {
		needsSync: [],
		alreadySynced: [],
		unmatched: [],
		controllerMatches: []
	};

	if (!windows || !linux) return result;

	const winControllers = windows.controllers ?? [];
	const linControllers = linux.controllers ?? [];

	// Match controllers by MAC address (reuses logic from old Sync.svelte)
	const unmatchedLinux = new Set(linControllers.map((c) => c.address.toUpperCase()));

	for (const winCtrl of winControllers) {
		const winMac = winCtrl.address.toUpperCase();
		const linCtrl = linControllers.find((c) => c.address.toUpperCase() === winMac);

		const ctrlMatch: ControllerMatch = {
			controllerAddress: winMac,
			controllerName: winCtrl.name ?? linCtrl?.name ?? null,
			windowsController: winCtrl,
			linuxController: linCtrl ?? null
		};
		result.controllerMatches.push(ctrlMatch);

		if (linCtrl) {
			unmatchedLinux.delete(winMac);
			const { paired, unmatched } = matchDevicesForController(winCtrl, linCtrl, winMac);
			for (const pair of paired) {
				if (pair.comparison.overallStatus === 'synced') {
					result.alreadySynced.push(pair);
				} else {
					result.needsSync.push(pair);
				}
			}
			result.unmatched.push(...unmatched);
		} else {
			// All Windows devices under unmatched controller
			for (const dev of winCtrl.devices) {
				result.unmatched.push({ device: dev, os: 'Windows', controllerAddress: winMac });
			}
		}
	}

	for (const linMac of unmatchedLinux) {
		const linCtrl = linControllers.find((c) => c.address.toUpperCase() === linMac)!;
		result.controllerMatches.push({
			controllerAddress: linMac,
			controllerName: linCtrl.name,
			windowsController: null,
			linuxController: linCtrl
		});
		for (const dev of linCtrl.devices) {
			result.unmatched.push({ device: dev, os: 'Linux', controllerAddress: linMac });
		}
	}

	// Sort: controllers by device count descending
	result.controllerMatches.sort((a, b) => {
		const aCount =
			(a.windowsController?.devices.length ?? 0) + (a.linuxController?.devices.length ?? 0);
		const bCount =
			(b.windowsController?.devices.length ?? 0) + (b.linuxController?.devices.length ?? 0);
		return bCount - aCount;
	});

	return result;
}

// --- Selection Helpers ---

export function pairKey(controllerAddress: string, deviceAddress: string): string {
	return `${controllerAddress}/${deviceAddress}`;
}

export function initSelections(matchResult: MatchResult): SyncSelections {
	const selections: SyncSelections = new Map();
	for (const pair of matchResult.needsSync) {
		selections.set(pairKey(pair.controllerAddress, pair.windowsDevice.address), {
			enabled: true,
			direction: pair.recommendedDirection
		});
	}
	return selections;
}

export function buildSyncProposals(
	matchResult: MatchResult,
	selections: SyncSelections
): SyncProposal[] {
	const proposals: SyncProposal[] = [];

	for (const pair of matchResult.needsSync) {
		const key = pairKey(pair.controllerAddress, pair.windowsDevice.address);
		const selection = selections.get(key);
		if (!selection?.enabled) continue;

		const isWinToLin = selection.direction === 'win_to_linux';
		proposals.push({
			source_device: isWinToLin ? pair.windowsDevice : pair.linuxDevice,
			target_device: isWinToLin ? pair.linuxDevice : pair.windowsDevice,
			source_os: isWinToLin ? 'Windows' : 'Linux',
			target_os: isWinToLin ? 'Linux' : 'Windows',
			action: 'copy_keys'
		});
	}

	return proposals;
}

export function describeSyncChanges(pair: MatchedDevicePair, direction: SyncDirection): string[] {
	const changes: string[] = [];
	const allFields = [...pair.comparison.linkKey, ...pair.comparison.leData];
	const isWinToLin = direction === 'win_to_linux';

	for (const field of allFields) {
		if (field.status === 'match' || field.status === 'both_missing') continue;

		const sourceVal = isWinToLin ? field.windowsValue : field.linuxValue;
		if (sourceVal != null) {
			changes.push(field.label);
		}
	}

	return changes;
}
