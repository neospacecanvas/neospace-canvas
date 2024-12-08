import { ViewportManager } from './Viewport';
import { Edge, AnchorSide } from '../types/types';
import { EdgeDrawingState, EdgeUIState, EdgePathElements } from '../types/edge-types';

export class EdgeInteractionManager {
    private readonly SNAP_DISTANCE = 50;
    private viewportManager: ViewportManager;
    private drawingState: EdgeDrawingState = {
        isDrawing: false,
        startNode: null,
        startSide: null,
        tempPath: null
    };
    private uiState: EdgeUIState = {
        selectedEdgeId: null,
        hoveredEdgeId: null,
        toolbarVisible: false
    };
    
    private onEdgeComplete: (fromNode: string, toNode: string, fromSide: AnchorSide, toSide: AnchorSide) => void;
    private onEdgeDelete: (edgeId: string) => void;
    private onEdgeSelect: (edgeId: string | null) => void;

    constructor(callbacks: {
        onEdgeComplete: (fromNode: string, toNode: string, fromSide: AnchorSide, toSide: AnchorSide) => void;
        onEdgeDelete: (edgeId: string) => void;
        onEdgeSelect: (edgeId: string | null) => void;
    }) {
        this.viewportManager = ViewportManager.getInstance();
        this.onEdgeComplete = callbacks.onEdgeComplete;
        this.onEdgeDelete = callbacks.onEdgeDelete;
        this.onEdgeSelect = callbacks.onEdgeSelect;
    }

    public setupPathInteraction(edge: Edge, elements: EdgePathElements): void {
        const { hitArea, path } = elements;

        hitArea.addEventListener('mouseenter', () => {
            if (this.uiState.selectedEdgeId !== edge.id) {
                path.style.stroke = 'rgba(59, 130, 246, 0.5)';
                path.style.strokeWidth = '2';
                path.style.filter = 'drop-shadow(0 0 3px rgba(59, 130, 246, 0.3))';
                path.setAttribute('marker-end', 'url(#arrowhead-highlighted)');
            }
            this.uiState.hoveredEdgeId = edge.id;
        });

        hitArea.addEventListener('mouseleave', () => {
            if (this.uiState.selectedEdgeId !== edge.id) {
                path.style.stroke = '#000';
                path.style.strokeWidth = '1';
                path.style.filter = 'none';
                path.setAttribute('marker-end', 'url(#arrowhead)');
            }
            this.uiState.hoveredEdgeId = null;
        });

        hitArea.addEventListener('click', (e) => {
            e.stopPropagation();
            this.selectEdge(edge.id, path);
        });
    }

    public startEdgeDrawing(nodeId: string, side: AnchorSide): void {
        this.drawingState = {
            isDrawing: true,
            startNode: nodeId,
            startSide: side,
            tempPath: null
        };
    }

    public handleEdgeDrawing(
        mouseX: number, 
        mouseY: number, 
        getNodeAtPoint: (x: number, y: number) => HTMLElement | null
    ): { targetNode: HTMLElement | null; targetSide: AnchorSide | null } {
        const { scale, panX, panY } = this.viewportManager.getState();
        const x = (mouseX - panX) / scale;
        const y = (mouseY - panY) / scale;

        const targetNode = getNodeAtPoint(mouseX, mouseY);
        let targetSide: AnchorSide | null = null;

        if (targetNode && targetNode.id !== this.drawingState.startNode) {
            const anchors = targetNode.querySelectorAll('.anchor-point');
            let closestAnchor: Element | null = null;
            let closestDistance = this.SNAP_DISTANCE;

            anchors.forEach(anchor => {
                const rect = anchor.getBoundingClientRect();
                const anchorX = rect.left + rect.width / 2;
                const anchorY = rect.top + rect.height / 2;
                const distance = Math.sqrt(
                    Math.pow(mouseX - anchorX, 2) + 
                    Math.pow(mouseY - anchorY, 2)
                );

                if (distance < closestDistance) {
                    closestDistance = distance;
                    closestAnchor = anchor;
                }
            });

            if (closestAnchor) {
                const side = Array.from(closestAnchor.classList)
                    .find(cls => cls.startsWith('anchor-'))
                    ?.replace('anchor-', '') as AnchorSide;
                
                if (side) {
                    targetSide = side;
                    anchors.forEach(a => a.classList.remove('highlight'));
                    closestAnchor.classList.add('highlight');
                }
            }
        }

        return { targetNode, targetSide };
    }

    public completeEdgeDrawing(targetNode: string, targetSide: AnchorSide): void {
        if (this.drawingState.startNode && this.drawingState.startSide) {
            this.onEdgeComplete(
                this.drawingState.startNode,
                targetNode,
                this.drawingState.startSide,
                targetSide
            );
        }
        this.cancelEdgeDrawing();
    }

    public cancelEdgeDrawing(): void {
        if (this.drawingState.tempPath) {
            this.drawingState.tempPath.remove();
        }
        
        document.querySelectorAll('.anchor-point.highlight').forEach(point => {
            point.classList.remove('highlight');
        });

        this.drawingState = {
            isDrawing: false,
            startNode: null,
            startSide: null,
            tempPath: null
        };
    }

    private selectEdge(edgeId: string, pathElement: SVGPathElement): void {
        if (this.uiState.selectedEdgeId) {
            // Deselect previously selected edge
            const prevSelected = document.querySelector(`path[data-edge-id="${this.uiState.selectedEdgeId}"]`) as SVGPathElement | null;
            if (prevSelected) {
                prevSelected.style.stroke = '#000';
                prevSelected.style.strokeWidth = '1';
                prevSelected.setAttribute('marker-end', 'url(#arrowhead)');
            }
        }
    
        this.uiState.selectedEdgeId = edgeId;
        pathElement.style.stroke = '#3b82f6';
        pathElement.style.strokeWidth = '2';
        pathElement.setAttribute('marker-end', 'url(#arrowhead-highlighted)');
        this.onEdgeSelect(edgeId);
    }

    public deselectCurrentEdge(): void {
        if (this.uiState.selectedEdgeId) {
            const selected = document.querySelector(`path[data-edge-id="${this.uiState.selectedEdgeId}"]`) as SVGPathElement | null;
            if (selected) {
                selected.style.stroke = '#000';
                selected.style.strokeWidth = '1';
                selected.setAttribute('marker-end', 'url(#arrowhead)');
            }
            this.uiState.selectedEdgeId = null;
            this.onEdgeSelect(null);
        }
    }

    public deleteSelectedEdge(): void {
        if (this.uiState.selectedEdgeId) {
            this.onEdgeDelete(this.uiState.selectedEdgeId);
            this.uiState.selectedEdgeId = null;
        }
    }

    public get isDrawing(): boolean {
        return this.drawingState.isDrawing;
    }

    public setTempPath(path: SVGPathElement | null): void {
        this.drawingState.tempPath = path;
    }
}