import type { HostDistributions } from "#root/bindings";

export class NodeID {
	os: HostDistributions;
	controller: string;
	device: string;

	constructor (os: HostDistributions, controller: string, device: string) {
		this.os = os;
		this.controller = controller;
		this.device = device;
	}


	public toString(): string {
		return `${this.os}/${this.controller}/${this.device}`;
	}

	public static parse(id: string): NodeID | null {
		if (!id) {
			return null;
		}

		const parts = id.split("/");
		if (parts.length != 3) {
			return null;
		}

		const os = id[0] as HostDistributions;
		const controller = id[1];
		const device = id[2];

		return new NodeID(os, controller, device);
	}
}
