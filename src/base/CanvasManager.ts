import { WebGLGridManager } from '@/base/WebGLGridManager';
import { Coordinate, NodeType } from '@/types/types';
import { Node } from '@/base/Node';
import { ViewportState } from '@/types/canvas';
import { CSVNode } from './CSVNode';

export class CanvasManager {
    [x: string]: any;
    private readonly container: HTMLElement;
    private readonly nodesContainer: HTMLDivElement;
    private readonly edgesContainer: SVGSVGElement;
    private readonly gridManager: WebGLGridManager;
    private readonly nodes: Map<string, Node>;
    
    // Viewport state
    private scale: number = 1;
    private panOffset: Coordinate = { x: 0, y: 0 };
    
    // Interaction state
    private isDragging: boolean = false;
    private isSpacePressed: boolean = false;
    private lastMousePos: Coordinate | null = null;
    // private selectedNodeId: string | null = null;
    
    // Constants
    private readonly MIN_SCALE = 0.1;
    private readonly MAX_SCALE = 5.0;
    private readonly ZOOM_SPEED = 0.001;

    constructor(containerId: string) {
        // Initialize containers
        const container = document.getElementById(containerId);
        if (!container) throw new Error('Canvas container not found');
        this.container = container;
        
        // Create nodes container
        this.nodesContainer = document.createElement('div');
        this.nodesContainer.id = 'canvas-nodes';
        this.container.appendChild(this.nodesContainer);
        
        // Create edges container
        this.edgesContainer = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
        this.edgesContainer.id = 'canvas-edges';
        this.container.appendChild(this.edgesContainer);
        
        // Initialize WebGL grid
        this.gridManager = new WebGLGridManager(this.container);
        
        // Initialize node storage
        this.nodes = new Map();
        
        // Setup event listeners
        this.setupEventListeners();
        this.updateTransform();
    }

    private setupEventListeners(): void {
        // Zoom handling
        this.container.addEventListener('wheel', this.handleWheel.bind(this));
        
        // Pan handling
        this.container.addEventListener('mousedown', this.handleMouseDown.bind(this));
        window.addEventListener('mousemove', this.handleMouseMove.bind(this));
        window.addEventListener('mouseup', this.handleMouseUp.bind(this));
        
        // Keyboard handling
        window.addEventListener('keydown', this.handleKeyDown.bind(this));
        window.addEventListener('keyup', this.handleKeyUp.bind(this));
        
        // Window resize handling
        window.addEventListener('resize', this.handleResize.bind(this));
    }

    private handleWheel(e: WheelEvent): void {
        if (e.ctrlKey || e.metaKey) {
            e.preventDefault();
            
            const rect = this.container.getBoundingClientRect();
            const x = e.clientX - rect.left;
            const y = e.clientY - rect.top;
            
            const zoomDelta = e.deltaY * -this.ZOOM_SPEED;
            const newScale = Math.min(Math.max(this.scale * (1 + zoomDelta), this.MIN_SCALE), this.MAX_SCALE);
            const scaleFactor = newScale / this.scale;
            
            this.panOffset.x += (x - this.panOffset.x) * (1 - scaleFactor);
            this.panOffset.y += (y - this.panOffset.y) * (1 - scaleFactor);
            this.scale = newScale;
            
            this.updateTransform();
        }
    }

    private handleMouseDown(e: MouseEvent): void {
        if (e.button === 0 && (this.isSpacePressed || e.target === this.container)) {
            this.isDragging = true;
            this.lastMousePos = { x: e.clientX, y: e.clientY };
            this.container.style.cursor = 'grabbing';
        }
    }

    private handleMouseMove(e: MouseEvent): void {
        if (!this.isDragging || !this.lastMousePos) return;
        
        const dx = e.clientX - this.lastMousePos.x;
        const dy = e.clientY - this.lastMousePos.y;
        
        this.panOffset.x -= dx;
        this.panOffset.y += dy;
        
        this.lastMousePos = { x: e.clientX, y: e.clientY };
        this.updateTransform();
    }

    private handleMouseUp(): void {
        if (this.isDragging) {
            this.isDragging = false;
            this.lastMousePos = null;
            this.container.style.cursor = '';
        }
    }

    private handleKeyDown(e: KeyboardEvent): void {
        if (e.target instanceof HTMLElement) {
            if (e.code === 'Space' && !e.repeat && !e.target.isContentEditable) {
                e.preventDefault();
                this.isSpacePressed = true;
                this.container.style.cursor = 'grab';
            }
        }
    }

    private handleKeyUp(e: KeyboardEvent): void {
        if (e.code === 'Space') {
            this.isSpacePressed = false;
            this.container.style.cursor = '';
        }
    }

    private handleResize(): void {
        // Now calls the public method
        this.gridManager.resize();
    }

    private updateTransform(): void {
        // Update CSS variables for nodes and edges transform
        document.documentElement.style.setProperty('--scale', String(this.scale));
        document.documentElement.style.setProperty('--pan-x', `${this.panOffset.x}px`);
        document.documentElement.style.setProperty('--pan-y', `${this.panOffset.y}px`);
        
        // Update WebGL grid
        this.gridManager.updateViewport({
            scale: this.scale,
            panOffset: this.panOffset
        });
    }

    public addNode(node: Node): void {
        console.log('Adding node to canvas:', node);
        this.nodes.set(node.getId(), node);
        const element = this.createNodeElement(node);
        console.log('Created element:', element);
        this.nodesContainer.appendChild(element);
        console.log('Node container children:', this.nodesContainer.children.length);
    }
    
    private createNodeElement(node: Node): HTMLElement {
        const element = document.createElement('div');
        element.id = node.getId();
        element.className = 'node';
        
        const pos = node.getPosition();
        element.style.left = `${pos.x}px`;
        element.style.top = `${pos.y}px`;
        
        console.log('Creating element for node type:', node.getType());
        
        if (node.getType() === NodeType.CSV) {
            element.className += ' csv-node';
            const dimensions = node.getDimensions();
            element.style.width = `${dimensions.width}px`;
            element.style.height = `${dimensions.height}px`;
            
            element.innerHTML = `
                <div class="node-header">CSV File</div>
                <div class="node-content">
                    <div class="node-icon">📊</div>
                    <div class="node-filename">${(node as CSVNode).getFileName()}</div>
                </div>
            `;
            console.log('Created CSV node element:', element.outerHTML);
        }
        
        return element;
    }

    public removeNode(nodeId: string): void {
        const element = document.getElementById(nodeId);
        if (element) {
            element.remove();
        }
        this.nodes.delete(nodeId);
    }

    public getViewportState(): ViewportState {
        return {
            scale: this.scale,
            panOffset: this.panOffset
        };
    }

    public destroy(): void {
        // Remove event listeners
        window.removeEventListener('mousemove', this.handleMouseMove.bind(this));
        window.removeEventListener('mouseup', this.handleMouseUp.bind(this));
        window.removeEventListener('keydown', this.handleKeyDown.bind(this));
        window.removeEventListener('keyup', this.handleKeyUp.bind(this));
        window.removeEventListener('resize', this.handleResize.bind(this));
        
        // Clean up grid
        this.gridManager.destroy();
        
        // Clear nodes
        this.nodes.clear();
        this.nodesContainer.innerHTML = '';
    }
    
    public getViewportCenter(): Coordinate {
        return {
            x: -this.panOffset.x / this.scale + window.innerWidth / 2 / this.scale,
            y: -this.panOffset.y / this.scale + window.innerHeight / 2 / this.scale
        };
    }
}