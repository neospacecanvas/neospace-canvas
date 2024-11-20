import { ViewportState } from '@/types/canvas';

export class GridManager {
    private readonly container: HTMLElement;
    private scale: number = 1;
    private panOffset: { x: number; y: number } = { x: 0, y: 0 };
    private isDragging: boolean = false;
    private lastMousePos: { x: number; y: number } | null = null;

    private readonly MIN_SCALE = 0.1;
    private readonly MAX_SCALE = 4.0;
    private readonly GRID_SIZE = 20;
    private readonly GRID_COLOR = '#a5d8ff';

    constructor(containerId: string) {
        const container = document.getElementById(containerId);
        if (!container) throw new Error('Grid container not found');
        this.container = container;
        
        this.setupContainer();
        this.setupEventListeners();
        this.updateTransform();
    }

    private setupContainer(): void {
        this.container.style.backgroundImage = `radial-gradient(${this.GRID_COLOR} calc(var(--scale)*0.5px + 0.5px), transparent 0)`;
        this.container.style.backgroundSize = `calc(var(--scale) * ${this.GRID_SIZE}px) calc(var(--scale) * ${this.GRID_SIZE}px)`;
        this.container.style.backgroundPosition = 'calc(var(--pan-x) - 19px) calc(var(--pan-y) - 19px)';
    }

    private updateTransform(): void {
        document.body.style.setProperty('--scale', this.scale.toString());
        document.body.style.setProperty('--pan-x', `${this.panOffset.x}px`);
        document.body.style.setProperty('--pan-y', `${this.panOffset.y}px`);
    }

    private setupEventListeners(): void {
        // Zoom handling
        this.container.addEventListener('wheel', (e: WheelEvent) => {
            if (e.ctrlKey || e.metaKey) {
                e.preventDefault();
                
                const rect = this.container.getBoundingClientRect();
                const x = e.clientX - rect.left;
                const y = e.clientY - rect.top;
                
                const delta = e.deltaY * -0.001;
                const newScale = Math.min(Math.max(this.MIN_SCALE, 
                    this.scale * (1 + delta)), this.MAX_SCALE);
                const scaleFactor = newScale / this.scale;
                
                this.panOffset = {
                    x: this.panOffset.x + (x - this.panOffset.x) * (1 - scaleFactor),
                    y: this.panOffset.y + (y - this.panOffset.y) * (1 - scaleFactor)
                };
                
                this.scale = newScale;
                this.updateTransform();
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

            this.panOffset = {
                x: this.panOffset.x + dx,
                y: this.panOffset.y + dy
            };

            this.lastMousePos = { x: e.clientX, y: e.clientY };
            this.updateTransform();
        });

        window.addEventListener('mouseup', () => {
            this.isDragging = false;
            this.lastMousePos = null;
            this.container.style.cursor = '';
        });
    }

    public getViewportState(): ViewportState {
        return {
            scale: this.scale,
            panOffset: { x: this.panOffset.x, y: this.panOffset.y }
        };
    }

    public setViewportState(state: ViewportState): void {
        this.scale = state.scale;
        this.panOffset = { ...state.panOffset };
        this.updateTransform();
    }
}