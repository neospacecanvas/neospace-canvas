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

    private constructor() {
        this.viewportManager = ViewportManager.getInstance();
        this.setupSVGContainer();
        this.setupArrowMarker();
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
    }

    private setupArrowMarker() {
        const defs = document.createElementNS('http://www.w3.org/2000/svg', 'defs');
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
        defs.appendChild(marker);
        this.svgContainer.appendChild(defs);
    }

    private getAnchorPoint(nodeId: string, side: AnchorSide): {x: number, y: number} {
        const node = document.getElementById(nodeId);
        if (!node) return {x: 0, y: 0};

        const { scale } = this.viewportManager.getState();
        const rect = node.getBoundingClientRect();
        const width = rect.width / scale;
        const height = rect.height / scale;
        const x = parseInt(node.style.left);
        const y = parseInt(node.style.top);

        switch(side) {
            case 'top':    return {x: x + width/2, y: y};
            case 'right':  return {x: x + width, y: y + height/2};
            case 'bottom': return {x: x + width/2, y: y + height};
            case 'left':   return {x, y: y + height/2};
        }
    }

    private findNearestSide(target: HTMLElement, x: number, y: number): AnchorSide {
        const rect = target.getBoundingClientRect();
        const { scale, panX, panY } = this.viewportManager.getState();
        
        // Convert screen coordinates to canvas coordinates
        const canvasX = (x - panX) / scale;
        const canvasY = (y - panY) / scale;

        // Get distances to each side
        const distToTop = Math.abs(canvasY - rect.top/scale);
        const distToBottom = Math.abs(canvasY - rect.bottom/scale);
        const distToLeft = Math.abs(canvasX - rect.left/scale);
        const distToRight = Math.abs(canvasX - rect.right/scale);

        const min = Math.min(distToTop, distToBottom, distToLeft, distToRight);

        if (min === distToTop) return 'top';
        if (min === distToBottom) return 'bottom';
        if (min === distToLeft) return 'left';
        return 'right';
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

        // Try to find target node
        const target = document.elementsFromPoint(mouseX, mouseY)
            .find(el => el.classList.contains('node')) as HTMLElement;

        let endX = x;
        let endY = y;
        let toSide: AnchorSide = 'right';

        if (target && target.id !== this.currentEdge.fromNode) {
            toSide = this.findNearestSide(target, mouseX, mouseY);
            const endPoint = this.getAnchorPoint(target.id, toSide);
            endX = endPoint.x;
            endY = endPoint.y;
        }

        const startPoint = this.getAnchorPoint(this.currentEdge.fromNode!, this.currentEdge.fromSide!);
        
        // Calculate control points for smooth curve
        const dx = endX - startPoint.x;
        const dy = endY - startPoint.y;
        const midX = startPoint.x + dx * 0.5;
        const midY = startPoint.y + dy * 0.5;

        let cp1x = startPoint.x;
        let cp1y = startPoint.y;
        let cp2x = endX;
        let cp2y = endY;

        // Adjust control points based on connection sides
        const tensionX = Math.abs(dx) * 0.5;
        const tensionY = Math.abs(dy) * 0.5;

        switch(this.currentEdge.fromSide) {
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

        const path = `M ${startPoint.x} ${startPoint.y} C ${cp1x} ${cp1y}, ${cp2x} ${cp2y}, ${endX} ${endY}`;
        this.tempLine.setAttribute('d', path);
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
    }

    public drawEdges() {
        // Clear existing edges
        const paths = this.svgContainer.querySelectorAll('path:not([id])');
        paths.forEach(path => path.remove());

        // Redraw all edges
        this.edges.forEach(edge => {
            const startPoint = this.getAnchorPoint(edge.fromNode, edge.fromSide);
            const endPoint = this.getAnchorPoint(edge.toNode, edge.toSide);

            const dx = endPoint.x - startPoint.x;
            const dy = endPoint.y - startPoint.y;
            const tensionX = Math.abs(dx) * 0.5;
            const tensionY = Math.abs(dy) * 0.5;

            let cp1x = startPoint.x;
            let cp1y = startPoint.y;
            let cp2x = endPoint.x;
            let cp2y = endPoint.y;

            switch(edge.fromSide) {
                case 'right':  cp1x += tensionX; break;
                case 'left':   cp1x -= tensionX; break;
                case 'bottom': cp1y += tensionY; break;
                case 'top':    cp1y -= tensionY; break;
            }

            switch(edge.toSide) {
                case 'right':  cp2x += tensionX; break;
                case 'left':   cp2x -= tensionX; break;
                case 'bottom': cp2y += tensionY; break;
                case 'top':    cp2y -= tensionY; break;
            }

            const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
            path.setAttribute('d', `M ${startPoint.x} ${startPoint.y} C ${cp1x} ${cp1y}, ${cp2x} ${cp2y}, ${endPoint.x} ${endPoint.y}`);
            path.setAttribute('stroke', 'black');
            path.setAttribute('fill', 'none');
            path.setAttribute('marker-end', 'url(#arrowhead)');
            
            this.svgContainer.appendChild(path);
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
}