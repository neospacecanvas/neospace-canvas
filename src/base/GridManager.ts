import { ViewportState } from '@/types/viewport';
import { ViewportManager } from './Viewport';

export class GridManager {
    private readonly container: HTMLElement;
    private isDragging: boolean = false;
    private lastMousePos: { x: number; y: number } | null = null;
    private viewportManager: ViewportManager;

    private readonly GRID_SIZE = 20;
    private readonly GRID_COLOR = '#a5d8ff';

    constructor(containerId: string) {
        const container = document.getElementById(containerId);
        if (!container) throw new Error('Grid container not found');
        this.container = container;
        
        this.viewportManager = ViewportManager.getInstance();
        
        this.setupContainer();
        this.setupEventListeners();
    }

    private setupContainer(): void {
        // Grid setup remains the same since it uses CSS variables directly
        this.container.style.backgroundImage = `radial-gradient(${this.GRID_COLOR} calc(var(--scale)*0.5px + 0.5px), transparent 0)`;
        this.container.style.backgroundSize = `calc(var(--scale) * ${this.GRID_SIZE}px) calc(var(--scale) * ${this.GRID_SIZE}px)`;
        this.container.style.backgroundPosition = 'calc(var(--pan-x) - 19px) calc(var(--pan-y) - 19px)';
    }

    private setupEventListeners(): void {
        // Zoom handling
        this.container.addEventListener('wheel', (e: WheelEvent) => {
            if (e.ctrlKey || e.metaKey) {
                e.preventDefault();

                const rect = this.container.getBoundingClientRect();
                const x = e.clientX - rect.left;
                const y = e.clientY - rect.top;

                const currentState = this.viewportManager.getState();
                const delta = e.deltaY * -0.001;
                const newScale = currentState.scale * (1 + delta);
                const scaleFactor = newScale / currentState.scale;

                const newState: ViewportState = {
                    scale: newScale,
                    panX: currentState.panX + (x - currentState.panX) * (1 - scaleFactor),
                    panY: currentState.panY + (y - currentState.panY) * (1 - scaleFactor)
                };

                this.viewportManager.updateState(newState);
            }
        }, { passive: false });

        // Pan handling
        this.container.addEventListener('mousedown', (e: MouseEvent) => {
            if (e.target === this.container) {
                this.isDragging = true;
                this.lastMousePos = { x: e.clientX, y: e.clientY };
                this.container.style.cursor = 'grabbing';
            }
        });

        window.addEventListener('mousemove', (e: MouseEvent) => {
            if (!this.isDragging || !this.lastMousePos) return;

            const dx = e.clientX - this.lastMousePos.x;
            const dy = e.clientY - this.lastMousePos.y;
            
            const currentState = this.viewportManager.getState();
            this.viewportManager.updateState({
                ...currentState,
                panX: currentState.panX + dx,
                panY: currentState.panY + dy
            });

            this.lastMousePos = { x: e.clientX, y: e.clientY };
        });

        window.addEventListener('mouseup', () => {
            this.isDragging = false;
            this.lastMousePos = null;
            this.container.style.cursor = '';
        });
    }
}