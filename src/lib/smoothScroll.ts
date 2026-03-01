/**
 * Smooth scroll fix for WebKitGTK (Tauri v2 on Linux).
 *
 * High-polling-rate mice emit wheel events faster than the compositor
 * can repaint, causing visible jank. This module intercepts wheel events,
 * accumulates a "target" scroll offset, and animates toward it with
 * framerate-independent exponential decay.
 */

/** Half-life in ms — how long to cover half the remaining distance. ~19ms ≈ 0.45 factor at 60fps. */
const HALF_LIFE_MS = 19;
const DECAY = Math.LN2 / HALF_LIFE_MS;
const STOP_THRESHOLD = 0.5;

export function enableSmoothScroll(): () => void {
	if (!navigator.userAgent.includes('Linux')) {
		return () => {};
	}

	let targetX = 0;
	let targetY = 0;
	let currentX = 0;
	let currentY = 0;

	let scrollEl: HTMLElement | null = null;
	let inDialog = false;
	let rafId: number | null = null;
	let lastTime = 0;

	// Cache scroll context to avoid getComputedStyle on every wheel event
	let cachedTarget: EventTarget | null = null;
	let cachedScrollEl: HTMLElement | null = null;
	let cachedInDialog = false;

	function resolveScrollContext(eventTarget: EventTarget | null) {
		if (eventTarget === cachedTarget) {
			scrollEl = cachedScrollEl;
			inDialog = cachedInDialog;
			return;
		}
		cachedTarget = eventTarget;
		let node = eventTarget as HTMLElement | null;
		let foundEl: HTMLElement | null = null;
		let foundDialog = false;
		while (node && node !== document.documentElement) {
			if (!foundEl) {
				const oy = node.style.overflowY;
				// Fast path: check inline style first, fall back to computed only if needed
				const overflow = (oy === 'auto' || oy === 'scroll')
					? oy
					: getComputedStyle(node).overflowY;
				if ((overflow === 'auto' || overflow === 'scroll') && node.scrollHeight > node.clientHeight) {
					foundEl = node;
				}
			}
			if (node.role === 'dialog' || node.hasAttribute('data-dialog-overlay')) {
				foundDialog = true;
				break;
			}
			node = node.parentElement;
		}
		cachedScrollEl = foundEl;
		cachedInDialog = foundDialog;
		scrollEl = foundEl;
		inDialog = foundDialog;
	}

	function tick(now: number) {
		const dt = lastTime ? Math.min(now - lastTime, 100) : 16;
		lastTime = now;

		const prevX = currentX;
		const prevY = currentY;

		const factor = 1 - Math.exp(-DECAY * dt);
		currentX += (targetX - currentX) * factor;
		currentY += (targetY - currentY) * factor;

		if (Math.abs(targetX - currentX) < STOP_THRESHOLD) currentX = targetX;
		if (Math.abs(targetY - currentY) < STOP_THRESHOLD) currentY = targetY;

		const dx = currentX - prevX;
		const dy = currentY - prevY;
		if (Math.abs(dx) > 0.01 || Math.abs(dy) > 0.01) {
			if (scrollEl) {
				const topBefore = scrollEl.scrollTop;
				const leftBefore = scrollEl.scrollLeft;
				scrollEl.scrollBy(dx, dy);
				if (!inDialog) {
					const remainX = dx - (scrollEl.scrollLeft - leftBefore);
					const remainY = dy - (scrollEl.scrollTop - topBefore);
					if (Math.abs(remainX) > 0.1 || Math.abs(remainY) > 0.1) {
						window.scrollBy(remainX, remainY);
					}
				}
			} else if (!inDialog) {
				window.scrollBy(dx, dy);
			}
		}

		if (currentX !== targetX || currentY !== targetY) {
			rafId = requestAnimationFrame(tick);
		} else {
			targetX = 0;
			targetY = 0;
			currentX = 0;
			currentY = 0;
			rafId = null;
		}
	}

	function onWheel(e: WheelEvent) {
		if (e.ctrlKey) return;
		e.preventDefault();

		targetX += e.deltaX;
		targetY += e.deltaY;
		resolveScrollContext(e.target);

		if (rafId === null) {
			lastTime = 0;
			rafId = requestAnimationFrame(tick);
		}
	}

	document.addEventListener('wheel', onWheel, { capture: true, passive: false });

	return () => {
		document.removeEventListener('wheel', onWheel, { capture: true } as EventListenerOptions);
		if (rafId !== null) {
			cancelAnimationFrame(rafId);
			rafId = null;
		}
	};
}
