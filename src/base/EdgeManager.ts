import { ViewportManager } from './Viewport';
import { Edge, AnchorSide } from '../types/types';
import { EdgeRenderer } from './EdgeRenderer';
import { EdgeInteractionManager } from './EdgeInteractionManager';

export class EdgeManager {
    private static instance: EdgeManager;
    private edges: Edge[] = [];
    private renderer: EdgeRenderer;
    private interactionManager: EdgeInteractionManager;
    private viewportManager: ViewportManager;
    private unsubscribeViewport: () => void;

    private constructor() {
        const container = document.querySelector('.canvas-container') as HTMLElement;
        if (!container) throw new Error('Canvas container not found');

        this.viewportManager = ViewportManager.getInstance();
        this.renderer = new EdgeRenderer(container);
        this.interactionManager = new EdgeInteractionManager({
            onEdgeComplete: this.handleEdgeComplete.bind(this),
            onEdgeDelete: this.deleteEdge.bind(this),
            onEdgeSelect: this.handleEdgeSelect.bind(this)
        });

        this.setupViewportSubscription();
        this.setupGlobalListeners();
    }

    public static getInstance(): EdgeManager {
        if (!EdgeManager.instance) {
            EdgeManager.instance = new EdgeManager();
        }
        return EdgeManager.instance;
    }

    private setupViewportSubscription(): void {
        this.unsubscribeViewport = this.viewportManager.subscribe(() => {
            this.drawEdges();
        });
    }

    private setupGlobalListeners(): void {
        document.addEventListener('click', (e) => {
            if (!(e.target as Element).closest('path')) {
                this.interactionManager.deselectCurrentEdge();
            }
        });

        document.addEventListener('keydown', (e) => {
            if (e.key === 'Delete' || e.key === 'Backspace') {
                if (!(e.target as Element).closest('input, [contenteditable="true"]')) {
                    this.interactionManager.deleteSelectedEdge();
                }
            }
            if (e.key === 'Escape') {
                this.interactionManager.cancelEdgeDrawing();
            }
        });
    }

    public createEdge(edge: Edge): void {
        this.edges.push(edge);
        this.drawEdges();
    }

    private handleEdgeComplete(
        fromNode: string,
        toNode: string,
        fromSide: AnchorSide,
        toSide: AnchorSide
    ): void {
        const edge: Edge = {
            id: crypto.randomUUID(),
            fromNode,
            toNode,
            fromSide,
            toSide,
            toEnd: 'arrow'
        };
        this.createEdge(edge);
    }

    private handleEdgeSelect(edgeId: string | null): void {
        // Future implementation for edge selection handling
    }

    public startEdge(nodeId: string, side: AnchorSide): void {
        this.interactionManager.startEdgeDrawing(nodeId, side);
        const tempPath = this.renderer.createTemporaryPath();
        this.interactionManager.setTempPath(tempPath);
        this.renderer.getSVGContainer().appendChild(tempPath);
    }

    public updateTempEdge(mouseX: number, mouseY: number): void {
        if (!this.interactionManager.isDrawing) return;

        const getNodeAtPoint = (x: number, y: number): HTMLElement | null => {
            return document.elementsFromPoint(x, y)
                .find(el => el.classList.contains('node')) as HTMLElement || null;
        };

        this.interactionManager.handleEdgeDrawing(
            mouseX,
            mouseY,
            getNodeAtPoint
        );

        // Update the temporary path if needed
        // Implementation details would go here
    }

    public completeEdge(nodeId: string, side: AnchorSide): void {
        this.interactionManager.completeEdgeDrawing(nodeId, side);
    }

    public cancelEdge(): void {
        this.interactionManager.cancelEdgeDrawing();
    }

    public drawEdges(): void {
        this.renderer.clear();
        
        this.edges.forEach(edge => {
            const startElement = document.getElementById(edge.fromNode);
            const endElement = document.getElementById(edge.toNode);
            
            if (startElement && endElement) {
                const points = this.renderer.calculateCurvePoints(
                    startElement,
                    endElement,
                    edge.fromSide,
                    edge.toSide
                );

                const elements = this.renderer.createPath(points, {
                    markerEnd: 'url(#arrowhead)'
                });

                elements.path.setAttribute('data-edge-id', edge.id);
                this.interactionManager.setupPathInteraction(edge, elements);
                this.renderer.getSVGContainer().appendChild(elements.group);
            }
        });
    }

    public removeEdgesForNode(nodeId: string): void {
        this.edges = this.edges.filter(edge => 
            edge.fromNode !== nodeId && edge.toNode !== nodeId
        );
        this.drawEdges();
    }

    public deleteEdge(edgeId: string): void {
        this.edges = this.edges.filter(edge => edge.id !== edgeId);
        this.drawEdges();
    }

    public clearEdges(): void {
        this.edges = [];
        this.drawEdges();
    }

    public serialize(): Edge[] {
        return this.edges.map(edge => ({
            id: edge.id,
            fromNode: edge.fromNode,
            toNode: edge.toNode,
            fromSide: edge.fromSide,
            toSide: edge.toSide,
            toEnd: edge.toEnd
        }));
    }

    public deserialize(edges: Edge[]): void {
        this.clearEdges();
        this.edges = edges;
        this.drawEdges();
    }

    public destroy(): void {
        this.unsubscribeViewport?.();
        this.renderer.destroy();
        this.clearEdges();
    }

    public get isDrawing(): boolean {
        return this.interactionManager.isDrawing;
    }
}