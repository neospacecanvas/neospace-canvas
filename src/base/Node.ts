import { ViewportManager } from './Viewport';
import { NodeStore } from './NodeStore';
import { EdgeManager } from './EdgeManager';
import { NodeType, NodeContent } from '../types/types';

export abstract class Node {
    protected element: HTMLElement;
    protected isDragging: boolean = false;
    protected isSpacePressed: boolean = false;
    protected viewportManager: ViewportManager;
    protected nodeStore: NodeStore;
    protected edgeManager: EdgeManager;
    protected id: string;
    protected unsubscribeViewport: () => void;
    protected unsubscribeStore: () => void;

    constructor(
        type: NodeType,
        initialContent: string,
        x: number = window.innerWidth/2, 
        y: number = window.innerHeight/2,
        width: number = 200,
        height: number = 150
    ) {
        this.viewportManager = ViewportManager.getInstance();
        this.nodeStore = NodeStore.getInstance();
        this.edgeManager = EdgeManager.getInstance();

        // Create node in store
        this.id = this.nodeStore.createNode(
            type,
            { x, y },
            { width, height },
            initialContent
        );

        this.setupElement();
        this.setupAnchorPoints();
        this.setupDrag();
        this.setupViewportSubscription();
    }

    private setupAnchorPoints() {
        const positions: Array<{ side: 'top' | 'right' | 'bottom' | 'left' }> = [
            { side: 'top' },
            { side: 'right' },
            { side: 'bottom' },
            { side: 'left' }
        ];

        positions.forEach(({ side }) => {
            const anchor = document.createElement('div');
            anchor.className = `anchor-point anchor-${side}`;

            anchor.addEventListener('mousedown', (e) => {
                e.stopPropagation();
                this.edgeManager.startEdge(`node-${this.id}`, side);
                anchor.classList.add('active');
            });

            anchor.addEventListener('mouseenter', () => {
                if (this.edgeManager.isDrawing) {
                    this.edgeManager.completeEdge(`node-${this.id}`, side);
                    const activeAnchors = this.element.querySelectorAll('.anchor-point.active');
                    activeAnchors.forEach(point => point.classList.remove('active'));
                }
            });

            this.element.appendChild(anchor);
        });
    }

    protected setupElement() {
        this.element = document.createElement('div');
        this.element.id = `node-${this.id}`;
        this.element.className = 'node';
        this.element.style.position = 'absolute';
        
        const node = this.nodeStore.getNode(this.id);
        if (node) {
            this.element.style.left = `${node.position.x}px`;
            this.element.style.top = `${node.position.y}px`;
            this.element.style.width = `${node.dimensions.width}px`;
            this.element.style.height = `${node.dimensions.height}px`;
        }
    }

    protected setupDrag() {
        let startX: number;
        let startY: number;
        let currentLeft: number;
        let currentTop: number;
        
        this.element.addEventListener('mousedown', (e) => {
            if (this.isSpacePressed) return;
            
            this.isDragging = true;
            this.element.classList.add('is-dragging');
            
            startX = e.clientX;
            startY = e.clientY;
            
            currentLeft = parseInt(this.element.style.left) || 0;
            currentTop = parseInt(this.element.style.top) || 0;
            
            e.stopPropagation();
        });
        
        window.addEventListener('mousemove', (e) => {
            if (!this.isDragging) return;
            
            const { scale } = this.viewportManager.getState();
            const dx = (e.clientX - startX) / scale;
            const dy = (e.clientY - startY) / scale;
            
            currentLeft += dx;
            currentTop += dy;
            
            this.nodeStore.updateNodePosition(this.id, {
                x: currentLeft,
                y: currentTop
            });
            
            startX = e.clientX;
            startY = e.clientY;
        });
        
        window.addEventListener('mouseup', () => {
            if (this.isDragging) {
                this.isDragging = false;
                this.element.classList.remove('is-dragging');
            }
        });
    }

    private setupViewportSubscription() {
        this.unsubscribeViewport = this.viewportManager.subscribe(() => {
            this.edgeManager.drawEdges();
        });

        this.unsubscribeStore = this.nodeStore.subscribe(this.id, (_, node) => {
            if (node.position.x !== parseInt(this.element.style.left) ||
                node.position.y !== parseInt(this.element.style.top)) {
                this.element.style.left = `${node.position.x}px`;
                this.element.style.top = `${node.position.y}px`;
                this.edgeManager.drawEdges();
            }

            if (node.dimensions.width !== parseInt(this.element.style.width) ||
                node.dimensions.height !== parseInt(this.element.style.height)) {
                this.element.style.width = `${node.dimensions.width}px`;
                this.element.style.height = `${node.dimensions.height}px`;
                this.edgeManager.drawEdges();
            }
        });
    }

    public destroy() {
        this.edgeManager.removeEdgesForNode(`node-${this.id}`);
        this.nodeStore.deleteNode(this.id);
        this.unsubscribeViewport?.();
        this.unsubscribeStore?.();
        this.element.remove();
    }

    public getElement(): HTMLElement {
        return this.element;
    }
}