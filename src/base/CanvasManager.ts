// src/base/CanvasManager.ts
import { CanvasNode } from './CanvasNode';
import { CanvasNodeType, Coordinate, NodeContent } from '@/types/types';

export class CanvasManager {
    private readonly container: HTMLElement;
    private readonly nodesContainer: HTMLDivElement;
    private readonly edgesContainer: SVGSVGElement;
    private readonly nodes: Map<string, CanvasNode>;
    
    // Viewport state
    private scale: number = 1;
    private panOffset: Coordinate = { x: 0, y: 0 };
    
    // Interaction state
    private isDragging = false;
    private lastMousePos: Coordinate | null = null;
    private selectedNodeId: string | null = null;
    
    // Constants
    private readonly MIN_SCALE = 0.1;
    private readonly MAX_SCALE = 2;
    private readonly GRID_SIZE = 20;

    constructor(containerId: string) {
        const container = document.getElementById(containerId);
        if (!container) throw new Error('Canvas container not found');
        this.container = container;
        
        // Initialize containers
        this.nodesContainer = this.createNodesContainer();
        this.edgesContainer = this.createEdgesContainer();
        this.nodes = new Map();
        
        this.setupEventListeners();
        this.updateCSSVariables();
    }

    private createNodesContainer(): HTMLDivElement {
        const existing = document.getElementById('canvas-nodes');
        if (existing instanceof HTMLDivElement) return existing;
        
        const container = document.createElement('div');
        container.id = 'canvas-nodes';
        this.container.appendChild(container);
        return container;
    }

    private createEdgesContainer(): SVGSVGElement {
        const existing = document.getElementById('canvas-edges');
        if (existing instanceof SVGSVGElement) return existing;
        
        const container = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
        container.id = 'canvas-edges';
        this.container.appendChild(container);
        return container;
    }

    private updateCSSVariables(): void {
        document.documentElement.style.setProperty('--scale', String(this.scale));
        document.documentElement.style.setProperty('--pan-x', `${this.panOffset.x}px`);
        document.documentElement.style.setProperty('--pan-y', `${this.panOffset.y}px`);
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
                this.updateCSSVariables();
            }
        }, { passive: false });

        // Pan and drag handling
        this.container.addEventListener('mousedown', this.handleMouseDown.bind(this));
        window.addEventListener('mousemove', this.handleMouseMove.bind(this));
        window.addEventListener('mouseup', this.handleMouseUp.bind(this));

        // Double-click to add node
        this.container.addEventListener('dblclick', (e: MouseEvent) => {
            if (e.target === this.container) {
                const canvasPos = this.screenToCanvasPosition(e.clientX, e.clientY);
                this.addNode(canvasPos, {
                    type: CanvasNodeType.MARKDOWN,
                    data: { content: '# New Note\nDouble click to edit' }
                });
            }
        });
    }

    private screenToCanvasPosition(screenX: number, screenY: number): Coordinate {
        const rect = this.container.getBoundingClientRect();
        return {
            x: (screenX - rect.left - this.panOffset.x) / this.scale,
            y: (screenY - rect.top - this.panOffset.y) / this.scale
        };
    }

    private handleMouseDown(e: MouseEvent): void {
        this.isDragging = true;
        this.lastMousePos = { x: e.clientX, y: e.clientY };
        
        const target = e.target as HTMLElement;
        const nodeEl = target.closest('.node');
        if (nodeEl) {
            this.selectedNodeId = nodeEl.id;
            nodeEl.classList.add('is-dragging');
        }
    }

    private handleMouseMove(e: MouseEvent): void {
        if (!this.isDragging || !this.lastMousePos) return;

        const dx = e.clientX - this.lastMousePos.x;
        const dy = e.clientY - this.lastMousePos.y;

        if (this.selectedNodeId) {
            const node = this.nodes.get(this.selectedNodeId);
            if (node) {
                const pos = node.getPosition();
                node.setPosition({
                    x: pos.x + dx / this.scale,
                    y: pos.y + dy / this.scale
                });
                this.updateNodeElement(this.selectedNodeId);
            }
        } else {
            this.panOffset = {
                x: this.panOffset.x + dx,
                y: this.panOffset.y + dy
            };
            this.updateCSSVariables();
        }

        this.lastMousePos = { x: e.clientX, y: e.clientY };
    }

    private handleMouseUp(): void {
        if (this.selectedNodeId) {
            const nodeEl = document.getElementById(this.selectedNodeId);
            nodeEl?.classList.remove('is-dragging');
        }
        
        this.isDragging = false;
        this.selectedNodeId = null;
        this.lastMousePos = null;
    }

    private updateNodeElement(nodeId: string): void {
        const node = this.nodes.get(nodeId);
        const element = document.getElementById(nodeId);
        if (node && element) {
            const pos = node.getPosition();
            element.style.left = `${pos.x}px`;
            element.style.top = `${pos.y}px`;
        }
    }

    public addNode(position: Coordinate, content: NodeContent): void {
        const node = new CanvasNode(position, content);
        this.nodes.set(node.getId(), node);
        
        const element = document.createElement('div');
        element.id = node.getId();
        element.className = `node node-${content.type}`;
        element.style.left = `${position.x}px`;
        element.style.top = `${position.y}px`;
        
        if (content.type === 'markdown') {
            element.innerHTML = `
                <div class="node-header">
                    <div class="node-title">Markdown Note</div>
                </div>
                <div class="node-content" contenteditable="true">
                    ${content.data.content}
                </div>
            `;
        }
        
        this.nodesContainer.appendChild(element);
    }

    public removeNode(nodeId: string): void {
        const element = document.getElementById(nodeId);
        if (element) {
            element.remove();
        }
        this.nodes.delete(nodeId);
    }

    public clear(): void {
        this.nodesContainer.innerHTML = '';
        this.nodes.clear();
    }
}