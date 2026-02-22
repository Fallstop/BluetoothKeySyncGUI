import type {
	BluetoothController,
	BluetoothData,
	BluetoothDevice,
	BluetoothLinkKey,
	BluetoothLowEnergyKey,
	LongTermKeyData,
	SignatureKeyData,
	HostDistributions
} from '#root/bindings';

// Mirrors Rust SyncProposal/SyncRequest/SyncResult in sync_api.rs
// These aren't in the auto-generated bindings yet
export type SyncProposal =
	| {
			action: 'CopyKeys';
			source_device: BluetoothDevice;
			target_device: BluetoothDevice;
			source_os: HostDistributions;
			target_os: HostDistributions;
	  }
	| {
			action: 'DeleteDevice';
			device: BluetoothDevice;
			os: HostDistributions;
			controller_address: string;
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
}

export interface ManualMatch {
	id: string;
	windowsDevice: BluetoothDevice;
	linuxDevice: BluetoothDevice;
	windowsControllerAddr: string;
	linuxControllerAddr: string;
	comparison: KeyComparison;
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
	direction: SyncDirection | null;
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
	return [
		compareField('key', 'Link Key', windowsKey?.key, linuxKey?.key),
		compareField('key_type', 'Key Type', windowsKey?.key_type, linuxKey?.key_type),
		compareField('pin_length', 'PIN Length', windowsKey?.pin_length, linuxKey?.pin_length)
	];
}

function compareLtkGroup(
	prefix: 'ltk' | 'peripheral',
	winKey: LongTermKeyData | null | undefined,
	linKey: LongTermKeyData | null | undefined
): KeyFieldComparison[] {
	if (prefix === 'ltk') {
		return [
			compareField('long_term_key', 'Long Term Key (LTK)', winKey?.key, linKey?.key),
			compareField('ltk_authenticated', 'LTK Authenticated', winKey?.authenticated, linKey?.authenticated),
			compareField('ediv', 'EDIV', winKey?.ediv, linKey?.ediv),
			compareField('rand', 'Rand', winKey?.rand, linKey?.rand),
			compareField('key_length', 'Key Length', winKey?.key_length, linKey?.key_length)
		];
	} else {
		return [
			compareField('peripheral_ltk', 'Peripheral LTK', winKey?.key, linKey?.key),
			compareField('peripheral_ltk_authenticated', 'Peripheral LTK Authenticated', winKey?.authenticated, linKey?.authenticated),
			compareField('peripheral_ediv', 'Peripheral EDIV', winKey?.ediv, linKey?.ediv),
			compareField('peripheral_rand', 'Peripheral Rand', winKey?.rand, linKey?.rand),
			compareField('peripheral_key_length', 'Peripheral Key Length', winKey?.key_length, linKey?.key_length)
		];
	}
}

export function compareLeData(
	windowsLe: BluetoothLowEnergyKey | null,
	linuxLe: BluetoothLowEnergyKey | null
): KeyFieldComparison[] {
	const fields: KeyFieldComparison[] = [];

	const winLtk = windowsLe?.long_term_key ?? null;
	const winPeripheralLtk = windowsLe?.peripheral_long_term_key ?? null;
	const linLtk = linuxLe?.long_term_key ?? null;
	const linPeripheralLtk = linuxLe?.peripheral_long_term_key ?? null;

	// Cross-role: one side has LTK only, other has Peripheral LTK only.
	// Windows BTHPORT doesn't distinguish roles — always stores as LTK.
	// Linux BlueZ does — peripherals use [PeripheralLongTermKey].
	// Compare them against each other as one effective "Long Term Key".
	const isCrossRole =
		(winLtk && !winPeripheralLtk && !linLtk && linPeripheralLtk) ||
		(!winLtk && winPeripheralLtk && linLtk && !linPeripheralLtk);

	if (isCrossRole) {
		const effectiveWin = winLtk ?? winPeripheralLtk;
		const effectiveLin = linLtk ?? linPeripheralLtk;
		fields.push(...compareLtkGroup('ltk', effectiveWin, effectiveLin));
	} else {
		// Same-slot or dual-role: compare each separately
		fields.push(...compareLtkGroup('ltk', winLtk, linLtk));
		fields.push(...compareLtkGroup('peripheral', winPeripheralLtk, linPeripheralLtk));
	}

	// IRK
	fields.push(compareField('identity_resolving_key', 'Identity Resolving Key (IRK)', windowsLe?.identity_resolving_key, linuxLe?.identity_resolving_key));
	// Signature keys
	fields.push(compareField('local_signature_key', 'Local CSRK', windowsLe?.local_signature_key?.key, linuxLe?.local_signature_key?.key));
	fields.push(compareField('local_csrk_counter', 'Local CSRK Counter', windowsLe?.local_signature_key?.counter, linuxLe?.local_signature_key?.counter));
	fields.push(compareField('remote_signature_key', 'Remote CSRK', windowsLe?.remote_signature_key?.key, linuxLe?.remote_signature_key?.key));
	fields.push(compareField('remote_csrk_counter', 'Remote CSRK Counter', windowsLe?.remote_signature_key?.counter, linuxLe?.remote_signature_key?.counter));
	// Address type
	fields.push(compareField('address_type', 'Address Type', windowsLe?.address_type, linuxLe?.address_type));

	return fields;
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
				comparison
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

export function initSelections(_matchResult: MatchResult): SyncSelections {
	// Selections are only created when users manually pair devices.
	// No auto-selections — users pick exactly what they want to sync.
	return new Map();
}

export function manualPairKey(id: string): string {
	return `manual/${id}`;
}

let manualMatchCounter = 0;
export function createManualMatch(
	winDevice: BluetoothDevice,
	linDevice: BluetoothDevice,
	winCtrlAddr: string,
	linCtrlAddr: string
): ManualMatch {
	return {
		id: `manual-${++manualMatchCounter}`,
		windowsDevice: winDevice,
		linuxDevice: linDevice,
		windowsControllerAddr: winCtrlAddr,
		linuxControllerAddr: linCtrlAddr,
		comparison: compareKeys(winDevice, linDevice)
	};
}

export function buildSyncProposals(
	matchResult: MatchResult,
	selections: SyncSelections,
	manualMatches: ManualMatch[] = []
): SyncProposal[] {
	const proposals: SyncProposal[] = [];

	for (const pair of matchResult.needsSync) {
		const key = pairKey(pair.controllerAddress, pair.windowsDevice.address);
		const selection = selections.get(key);
		if (!selection?.direction) continue;

		const isWinToLin = selection.direction === 'win_to_linux';
		proposals.push({
			action: 'CopyKeys',
			source_device: isWinToLin ? pair.windowsDevice : pair.linuxDevice,
			target_device: isWinToLin ? pair.linuxDevice : pair.windowsDevice,
			source_os: isWinToLin ? 'Windows' : 'Linux',
			target_os: isWinToLin ? 'Linux' : 'Windows'
		});
	}

	for (const match of manualMatches) {
		const key = manualPairKey(match.id);
		const selection = selections.get(key);
		if (!selection?.direction) continue;

		const isWinToLin = selection.direction === 'win_to_linux';
		proposals.push({
			action: 'CopyKeys',
			source_device: isWinToLin ? match.windowsDevice : match.linuxDevice,
			target_device: isWinToLin ? match.linuxDevice : match.windowsDevice,
			source_os: isWinToLin ? 'Windows' : 'Linux',
			target_os: isWinToLin ? 'Linux' : 'Windows'
		});
	}

	return proposals;
}

export function deviceKey(os: HostDistributions, controllerAddress: string, deviceAddress: string): string {
	return `${os}/${controllerAddress}/${deviceAddress}`;
}

export function buildDeleteProposals(
	deletions: Set<string>,
	allDevices: UnmatchedDevice[]
): SyncProposal[] {
	const proposals: SyncProposal[] = [];

	for (const d of allDevices) {
		const key = deviceKey(d.os, d.controllerAddress, d.device.address);
		if (deletions.has(key)) {
			proposals.push({
				action: 'DeleteDevice',
				device: d.device,
				os: d.os,
				controller_address: d.controllerAddress
			});
		}
	}

	return proposals;
}

export function describeSyncChanges(
	pair: MatchedDevicePair | ManualMatch,
	direction: SyncDirection | null
): string[] {
	if (!direction) return [];

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
