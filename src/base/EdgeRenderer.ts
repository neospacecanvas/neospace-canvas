import { ViewportManager } from './Viewport';
import { AnchorSide } from '../types/types';
import { EdgePoint, EdgeCurvePoints, EdgeDrawOptions, EdgePathElements } from '../types/edge-types';

export class EdgeRenderer {
    private svgContainer: SVGElement;
    private defs: SVGDefsElement;
    private viewportManager: ViewportManager;

    constructor(container: HTMLElement) {
        this.viewportManager = ViewportManager.getInstance();
        this.setupSVGContainer(container);
        this.setupMarkers();
    }

    private setupSVGContainer(container: HTMLElement) {
        this.svgContainer = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
        this.svgContainer.id = 'canvas-edges';
        this.svgContainer.style.position = 'absolute';
        this.svgContainer.style.top = '0';
        this.svgContainer.style.left = '0';
        this.svgContainer.style.width = '100%';
        this.svgContainer.style.height = '100%';
        this.svgContainer.style.pointerEvents = 'none';
        container.insertBefore(this.svgContainer, container.firstChild);

        this.defs = document.createElementNS('http://www.w3.org/2000/svg', 'defs');
        this.svgContainer.appendChild(this.defs);
    }

    private setupMarkers() {
        // Default arrow marker
        const marker = this.createArrowMarker('arrowhead', '#000');
        const highlightedMarker = this.createArrowMarker('arrowhead-highlighted', '#3b82f6');
        
        this.defs.appendChild(marker);
        this.defs.appendChild(highlightedMarker);
    }

    private createArrowMarker(id: string, color: string): SVGMarkerElement {
        const marker = document.createElementNS('http://www.w3.org/2000/svg', 'marker');
        marker.setAttribute('id', id);
        marker.setAttribute('markerWidth', '10');
        marker.setAttribute('markerHeight', '7');
        marker.setAttribute('refX', '9');
        marker.setAttribute('refY', '3.5');
        marker.setAttribute('orient', 'auto');

        const polygon = document.createElementNS('http://www.w3.org/2000/svg', 'polygon');
        polygon.setAttribute('points', '0 0, 10 3.5, 0 7');
        polygon.setAttribute('fill', color);
        marker.appendChild(polygon);

        return marker;
    }

    public createPath(points: EdgeCurvePoints, options: EdgeDrawOptions = {}): EdgePathElements {
        const group = document.createElementNS('http://www.w3.org/2000/svg', 'g');
        const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
        const hitArea = document.createElementNS('http://www.w3.org/2000/svg', 'path');

        const d = this.createPathData(points);
        
        // Set up hit area
        hitArea.setAttribute('d', d);
        hitArea.setAttribute('stroke-width', '20');
        hitArea.setAttribute('stroke', 'rgba(0,0,0,0.001)');
        hitArea.setAttribute('fill', 'none');
        hitArea.style.pointerEvents = 'all';
        hitArea.style.cursor = 'pointer';

        // Set up visible path
        path.setAttribute('d', d);
        path.setAttribute('stroke', options.stroke || '#000');
        path.setAttribute('stroke-width', options.strokeWidth || '1');
        path.setAttribute('fill', 'none');
        path.setAttribute('marker-end', options.markerEnd || 'url(#arrowhead)');
        path.style.pointerEvents = 'none';
        path.style.transition = 'all 0.2s ease';

        if (options.className) {
            path.classList.add(options.className);
        }

        group.appendChild(hitArea);
        group.appendChild(path);

        return { group, path, hitArea };
    }

    public createTemporaryPath(): SVGPathElement {
        const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
        path.setAttribute('stroke', '#000');
        path.setAttribute('stroke-width', '1');
        path.setAttribute('fill', 'none');
        path.setAttribute('marker-end', 'url(#arrowhead)');
        return path;
    }

    private createPathData(points: EdgeCurvePoints): string {
        const { start, end, control1, control2 } = points;
        return `M ${start.x} ${start.y} C ${control1.x} ${control1.y}, ${control2.x} ${control2.y}, ${end.x} ${end.y}`;
    }

    public getAnchorPoint(element: HTMLElement, side: AnchorSide): EdgePoint {
        const { scale } = this.viewportManager.getState();
        const rect = element.getBoundingClientRect();
        
        const x = parseInt(element.style.left, 10);
        const y = parseInt(element.style.top, 10);
        const width = rect.width / scale;
        const height = rect.height / scale;

        switch(side) {
            case 'top':    return { x: x + width/2, y };
            case 'right':  return { x: x + width, y: y + height/2 };
            case 'bottom': return { x: x + width/2, y: y + height };
            case 'left':   return { x, y: y + height/2 };
        }
    }

    public calculateCurvePoints(
        startElement: HTMLElement,
        endElement: HTMLElement,
        fromSide: AnchorSide,
        toSide: AnchorSide
    ): EdgeCurvePoints {
        const start = this.getAnchorPoint(startElement, fromSide);
        const end = this.getAnchorPoint(endElement, toSide);
        
        const dx = end.x - start.x;
        const dy = end.y - start.y;
        const tensionX = Math.abs(dx) * 0.5;
        const tensionY = Math.abs(dy) * 0.5;

        const control1 = { ...start };
        const control2 = { ...end };

        // Adjust control points based on sides
        switch(fromSide) {
            case 'right':  control1.x += tensionX; break;
            case 'left':   control1.x -= tensionX; break;
            case 'bottom': control1.y += tensionY; break;
            case 'top':    control1.y -= tensionY; break;
        }

        switch(toSide) {
            case 'right':  control2.x += tensionX; break;
            case 'left':   control2.x -= tensionX; break;
            case 'bottom': control2.y += tensionY; break;
            case 'top':    control2.y -= tensionY; break;
        }

        return { start, end, control1, control2 };
    }

    public clear(): void {
        while (this.svgContainer.lastChild && this.svgContainer.lastChild !== this.defs) {
            this.svgContainer.removeChild(this.svgContainer.lastChild);
        }
    }

    public destroy(): void {
        this.svgContainer.remove();
    }

    public getSVGContainer(): SVGElement {
        return this.svgContainer;
    }
}