import { ViewportManager } from './Viewport';
import { AnchorSide, Edge } from '../types/types';

export class EdgeManager {
    private static instance: EdgeManager;
    private edges: Edge[] = [];
    private svgContainer: SVGElement;
    private currentEdge: Partial<Edge> | null = null;
    private tempLine: SVGPathElement | null = null;
    private viewportManager: ViewportManager;
    private _isDrawing: boolean = false;
    private selectedEdge: SVGPathElement | null = null;
    private unsubscribeViewport: () => void;
    private readonly SNAP_DISTANCE = 50;

    private constructor() {
        this.viewportManager = ViewportManager.getInstance();
        this.setupSVGContainer();
        this.setupArrowMarker();
        this.setupViewportSubscription();
        this.setupCanvasClickHandler();
    }

    public static getInstance(): EdgeManager {
        if (!EdgeManager.instance) {
            EdgeManager.instance = new EdgeManager();
        }
        return EdgeManager.instance;
    }

    private setupSVGContainer() {
        let svg = document.querySelector('#canvas-edges') as SVGElement;
        if (!svg) {
            svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
            svg.id = 'canvas-edges';
            const container = document.querySelector('.canvas-container');
            if (container) {
                container.insertBefore(svg, container.firstChild);
            }
        }
        this.svgContainer = svg;
        this.svgContainer.style.pointerEvents = 'none';
    }

    private setupArrowMarker() {
        const defs = document.createElementNS('http://www.w3.org/2000/svg', 'defs');
        
        // Default arrow marker
        const marker = document.createElementNS('http://www.w3.org/2000/svg', 'marker');
        marker.setAttribute('id', 'arrowhead');
        marker.setAttribute('markerWidth', '10');
        marker.setAttribute('markerHeight', '7');
        marker.setAttribute('refX', '9');
        marker.setAttribute('refY', '3.5');
        marker.setAttribute('orient', 'auto');
        const polygon = document.createElementNS('http://www.w3.org/2000/svg', 'polygon');
        polygon.setAttribute('points', '0 0, 10 3.5, 0 7');
        polygon.setAttribute('fill', '#000');
        marker.appendChild(polygon);
        
        // Highlighted arrow marker
        const highlightedMarker = document.createElementNS('http://www.w3.org/2000/svg', 'marker');
        highlightedMarker.setAttribute('id', 'arrowhead-highlighted');
        highlightedMarker.setAttribute('markerWidth', '10');
        highlightedMarker.setAttribute('markerHeight', '7');
        highlightedMarker.setAttribute('refX', '9');
        highlightedMarker.setAttribute('refY', '3.5');
        highlightedMarker.setAttribute('orient', 'auto');
        const highlightedPolygon = document.createElementNS('http://www.w3.org/2000/svg', 'polygon');
        highlightedPolygon.setAttribute('points', '0 0, 10 3.5, 0 7');
        highlightedPolygon.setAttribute('fill', '#3b82f6');
        highlightedMarker.appendChild(highlightedPolygon);

        defs.appendChild(marker);
        defs.appendChild(highlightedMarker);
        this.svgContainer.appendChild(defs);
    }

    private setupViewportSubscription() {
        this.unsubscribeViewport = this.viewportManager.subscribe(() => {
            this.drawEdges();
            this.updateToolbarPosition();
        });
    }

    private setupEdgeInteractions(path: SVGPathElement, edge: Edge) {
        // Create invisible wider path for hit detection
        const hitArea = path.cloneNode() as SVGPathElement;
        hitArea.setAttribute('stroke-width', '40'); // Even wider hit area
        hitArea.setAttribute('stroke', 'rgba(0,0,0,0.001)'); // Nearly transparent but not fully
        hitArea.setAttribute('fill', 'none');
        hitArea.style.pointerEvents = 'all'; // Changed from 'stroke' to 'all'
        hitArea.style.cursor = 'pointer';
        
        // Set up the visible path
        path.style.pointerEvents = 'none';
        path.style.transition = 'all 0.2s ease';
    
        // Add both to the container with proper ordering
        const group = document.createElementNS('http://www.w3.org/2000/svg', 'g');
        group.appendChild(hitArea); // Hit area goes first
        group.appendChild(path);   // Visible path on top
        
        // Handle interactions on the hit area
        hitArea.addEventListener('mouseenter', () => {
            if (this.selectedEdge !== path) {
                path.style.stroke = 'rgba(59, 130, 246, 0.5)';
                path.style.strokeWidth = '2';
                path.style.filter = 'drop-shadow(0 0 3px rgba(59, 130, 246, 0.3))';
                path.setAttribute('marker-end', 'url(#arrowhead-highlighted)');
            }
        });
    
        hitArea.addEventListener('mouseleave', () => {
            if (this.selectedEdge !== path) {
                path.style.stroke = '#000';
                path.style.strokeWidth = '1';
                path.style.filter = 'none';
                path.setAttribute('marker-end', 'url(#arrowhead)');
            }
        });
    
        hitArea.addEventListener('click', (e) => {
            e.stopPropagation();
            this.selectEdge(path, edge);
        });
    
        return group;
    }

    private selectEdge(path: SVGPathElement, edge: Edge) {
        if (this.selectedEdge) {
            this.selectedEdge.style.stroke = '#000';
            this.selectedEdge.style.strokeWidth = '1';
            this.selectedEdge.setAttribute('marker-end', 'url(#arrowhead)');
            this.removeEdgeToolbar();
        }

        this.selectedEdge = path;
        path.style.stroke = '#3b82f6';
        path.style.strokeWidth = '2';
        path.setAttribute('marker-end', 'url(#arrowhead-highlighted)');

        this.showEdgeToolbar(edge);
    }

    private showEdgeToolbar(edge: Edge) {
        const toolbar = document.createElement('div');
        toolbar.className = 'edge-toolbar';
        
        const deleteButton = document.createElement('button');
        deleteButton.className = 'toolbar-button';
        deleteButton.innerHTML = '🗑️';
        deleteButton.onclick = (e) => {
            e.stopPropagation();
            this.deleteEdge(edge.id);
        };

        toolbar.appendChild(deleteButton);
        this.positionToolbar(toolbar);
        document.body.appendChild(toolbar);
    }

    private positionToolbar(toolbar: HTMLElement) {
        if (this.selectedEdge) {
            const bbox = this.selectedEdge.getBBox();
            const { scale, panX, panY } = this.viewportManager.getState();
            toolbar.style.left = `${(bbox.x + bbox.width/2) * scale + panX - 20}px`;
            toolbar.style.top = `${(bbox.y + bbox.height/2) * scale + panY - 20}px`;
        }
    }

    private updateToolbarPosition() {
        const toolbar = document.querySelector('.edge-toolbar');
        if (toolbar) {
            this.positionToolbar(toolbar as HTMLElement);
        }
    }

    private removeEdgeToolbar() {
        const toolbar = document.querySelector('.edge-toolbar');
        if (toolbar) {
            toolbar.remove();
        }
        this.selectedEdge = null;
    }

    private deleteEdge(edgeId: string) {
        this.edges = this.edges.filter(edge => edge.id !== edgeId);
        this.removeEdgeToolbar();
        this.drawEdges();
    }

    private setupCanvasClickHandler() {
        document.addEventListener('click', (e) => {
            const target = e.target as Element;
            if (!target.closest('path') && !target.closest('.edge-toolbar')) {
                if (this.selectedEdge) {
                    this.selectedEdge.style.stroke = '#000';
                    this.selectedEdge.style.strokeWidth = '1';
                    this.selectedEdge.setAttribute('marker-end', 'url(#arrowhead)');
                    this.removeEdgeToolbar();
                }
            }
        });
    }

    public startEdge(nodeId: string, side: AnchorSide) {
        this._isDrawing = true;
        this.currentEdge = {
            fromNode: nodeId,
            fromSide: side,
            toEnd: 'arrow'
        };

        this.tempLine = document.createElementNS('http://www.w3.org/2000/svg', 'path');
        this.tempLine.setAttribute('stroke', 'black');
        this.tempLine.setAttribute('fill', 'none');
        this.tempLine.setAttribute('marker-end', 'url(#arrowhead)');
        this.svgContainer.appendChild(this.tempLine);
    }

    public updateTempEdge(mouseX: number, mouseY: number) {
        if (!this._isDrawing || !this.currentEdge || !this.tempLine) return;

        const { scale, panX, panY } = this.viewportManager.getState();
        const x = (mouseX - panX) / scale;
        const y = (mouseY - panY) / scale;

        const startPoint = this.getAnchorPoint(this.currentEdge.fromNode!, this.currentEdge.fromSide!);
        
        const target = document.elementsFromPoint(mouseX, mouseY)
            .find(el => el.classList.contains('node') && el.id !== this.currentEdge.fromNode) as HTMLElement;

        let endPoint = { x, y };
        let toSide: AnchorSide = 'right';

        if (target) {
            const targetAnchors = target.querySelectorAll('.anchor-point');
            let closestAnchor: Element | null = null;
            let closestDistance = this.SNAP_DISTANCE;

            targetAnchors.forEach(anchor => {
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
                    toSide = side;
                    endPoint = this.getAnchorPoint(target.id, side);
                    targetAnchors.forEach(a => a.classList.remove('highlight'));
                    closestAnchor.classList.add('highlight');
                }
            }
        }

        this.drawCurvedPath(this.tempLine, startPoint, endPoint, this.currentEdge.fromSide!, toSide);
    }

    public completeEdge(nodeId: string, side: AnchorSide) {
        if (!this.currentEdge || this.currentEdge.fromNode === nodeId) {
            this.cancelEdge();
            return;
        }

        const edge: Edge = {
            id: crypto.randomUUID(),
            fromNode: this.currentEdge.fromNode!,
            toNode: nodeId,
            fromSide: this.currentEdge.fromSide!,
            toSide: side,
            toEnd: 'arrow'
        };

        this.edges.push(edge);
        this.drawEdges();
        this.cancelEdge();
    }

    public cancelEdge() {
        if (this.tempLine) {
            this.tempLine.remove();
            this.tempLine = null;
        }
        this.currentEdge = null;
        this._isDrawing = false;

        document.querySelectorAll('.anchor-point.highlight').forEach(point => {
            point.classList.remove('highlight');
        });
    }

    private getAnchorPoint(nodeId: string, side: AnchorSide): {x: number, y: number} {
        const node = document.getElementById(nodeId);
        if (!node) return {x: 0, y: 0};

        const { scale } = this.viewportManager.getState();
        const rect = node.getBoundingClientRect();
        
        const x = parseInt(node.style.left, 10);
        const y = parseInt(node.style.top, 10);
        const width = rect.width / scale;
        const height = rect.height / scale;

        switch(side) {
            case 'top':    return {x: x + width/2, y};
            case 'right':  return {x: x + width, y: y + height/2};
            case 'bottom': return {x: x + width/2, y: y + height};
            case 'left':   return {x, y: y + height/2};
        }
    }

    private drawCurvedPath(
        path: SVGPathElement, 
        start: {x: number, y: number}, 
        end: {x: number, y: number},
        fromSide: AnchorSide,
        toSide: AnchorSide
    ) {
        const dx = end.x - start.x;
        const dy = end.y - start.y;

        let cp1x = start.x;
        let cp1y = start.y;
        let cp2x = end.x;
        let cp2y = end.y;

        const tensionX = Math.abs(dx) * 0.5;
        const tensionY = Math.abs(dy) * 0.5;

        switch(fromSide) {
            case 'right':  cp1x += tensionX; break;
            case 'left':   cp1x -= tensionX; break;
            case 'bottom': cp1y += tensionY; break;
            case 'top':    cp1y -= tensionY; break;
        }

        switch(toSide) {
            case 'right':  cp2x += tensionX; break;
            case 'left':   cp2x -= tensionX; break;
            case 'bottom': cp2y += tensionY; break;
            case 'top':    cp2y -= tensionY; break;
        }

        const d = `M ${start.x} ${start.y} C ${cp1x} ${cp1y}, ${cp2x} ${cp2y}, ${end.x} ${end.y}`;
        path.setAttribute('d', d);
    }

    public drawEdges() {
        // Clear existing paths
        const paths = this.svgContainer.querySelectorAll('g');
        paths.forEach(path => path.remove());
    
        this.edges.forEach(edge => {
            const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
            const startPoint = this.getAnchorPoint(edge.fromNode, edge.fromSide);
            const endPoint = this.getAnchorPoint(edge.toNode, edge.toSide);
    
            this.drawCurvedPath(path, startPoint, endPoint, edge.fromSide, edge.toSide);
            path.setAttribute('stroke', '#000');
            path.setAttribute('stroke-width', '1');
            path.setAttribute('fill', 'none');
            path.setAttribute('marker-end', 'url(#arrowhead)');
            
            const group = this.setupEdgeInteractions(path, edge);
            this.svgContainer.appendChild(group);
        });
    }

    public removeEdgesForNode(nodeId: string) {
        this.edges = this.edges.filter(edge => 
            edge.fromNode !== nodeId && edge.toNode !== nodeId
        );
        this.drawEdges();
    }

    public get isDrawing(): boolean {
        return this._isDrawing;
    }

    public destroy() {
        // Clean up subscriptions and listeners
        this.unsubscribeViewport?.();
        this.removeEdgeToolbar();
        
        // Clear all edges
        this.edges = [];
        this.drawEdges();
    }
}