import { ViewportManager } from './Viewport';
import { AnchorSide } from '../types/types';
import { EdgeRenderer } from './EdgeRenderer';

interface EdgeConnection {
    id: string;
    fromNodeId: string;
    toNodeId: string;
    fromSide: AnchorSide;
    toSide: AnchorSide;
}

interface EdgePoint {
    x: number;
    y: number;
    nodeId: string;
    side: AnchorSide;
}

export class EdgeConnectionManager {
    private static instance: EdgeConnectionManager | null = null;
    private connections: EdgeConnection[] = [];
    private nodeObserver: MutationObserver;
    private resizeObserver: ResizeObserver;
    private viewportManager: ViewportManager;
    private renderer: EdgeRenderer;
    private updateScheduled: boolean = false;

    private constructor(container: HTMLElement) {
        this.viewportManager = ViewportManager.getInstance();
        this.renderer = new EdgeRenderer(container);
        this.setupObservers();
        this.setupViewportSubscription();
    }

    public static getInstance(container?: HTMLElement): EdgeConnectionManager {
        if (!EdgeConnectionManager.instance && container) {
            EdgeConnectionManager.instance = new EdgeConnectionManager(container);
        }
        return EdgeConnectionManager.instance!;
    }

    private setupObservers(): void {
        // Watch for node position changes
        this.nodeObserver = new MutationObserver((mutations) => {
            const needsUpdate = mutations.some(mutation => 
                mutation.target instanceof HTMLElement &&
                mutation.target.classList.contains('node') &&
                mutation.attributeName === 'style'
            );
            if (needsUpdate) {
                this.scheduleUpdate();
            }
        });

        // Watch for node size changes
        this.resizeObserver = new ResizeObserver(() => {
            this.scheduleUpdate();
        });

        // Observe the canvas container
        const container = document.getElementById('canvas-nodes');
        if (container) {
            this.nodeObserver.observe(container, {
                attributes: true,
                attributeFilter: ['style'],
                subtree: true
            });
        }
    }

    private setupViewportSubscription(): void {
        this.viewportManager.subscribe(() => {
            this.scheduleUpdate();
        });
    }

    private scheduleUpdate(): void {
        if (!this.updateScheduled) {
            this.updateScheduled = true;
            requestAnimationFrame(() => {
                this.updateEdges();
                this.updateScheduled = false;
            });
        }
    }

    public addConnection(fromNodeId: string, toNodeId: string, fromSide: AnchorSide, toSide: AnchorSide): string {
        const id = crypto.randomUUID();
        const connection: EdgeConnection = {
            id,
            fromNodeId,
            toNodeId,
            fromSide,
            toSide
        };
        
        this.connections.push(connection);
        
        // Start observing the nodes
        const fromNode = document.getElementById(fromNodeId);
        const toNode = document.getElementById(toNodeId);
        
        if (fromNode) this.resizeObserver.observe(fromNode);
        if (toNode) this.resizeObserver.observe(toNode);
        
        this.scheduleUpdate();
        return id;
    }

    public removeConnection(id: string): void {
        const connection = this.connections.find(conn => conn.id === id);
        if (connection) {
            // Stop observing the nodes if they're not used by other connections
            if (!this.isNodeUsedInOtherConnections(connection.fromNodeId, id)) {
                const fromNode = document.getElementById(connection.fromNodeId);
                if (fromNode) this.resizeObserver.unobserve(fromNode);
            }
            if (!this.isNodeUsedInOtherConnections(connection.toNodeId, id)) {
                const toNode = document.getElementById(connection.toNodeId);
                if (toNode) this.resizeObserver.unobserve(toNode);
            }
        }
        
        this.connections = this.connections.filter(conn => conn.id !== id);
        this.scheduleUpdate();
    }

    private isNodeUsedInOtherConnections(nodeId: string, excludeConnectionId: string): boolean {
        return this.connections.some(conn => 
            conn.id !== excludeConnectionId && 
            (conn.fromNodeId === nodeId || conn.toNodeId === nodeId)
        );
    }

    private getNodeAnchorPoint(nodeId: string, side: AnchorSide): EdgePoint | null {
        const node = document.getElementById(nodeId);
        if (!node) return null;

        const rect = node.getBoundingClientRect();
        const { scale, panX, panY } = this.viewportManager.getState();

        // Convert screen coordinates to SVG coordinate space
        const x = (rect.left - panX) / scale;
        const y = (rect.top - panY) / scale;
        const width = rect.width / scale;
        const height = rect.height / scale;

        let pointX = x;
        let pointY = y;

        switch (side) {
            case 'top':
                pointX += width / 2;
                break;
            case 'right':
                pointX += width;
                pointY += height / 2;
                break;
            case 'bottom':
                pointX += width / 2;
                pointY += height;
                break;
            case 'left':
                pointY += height / 2;
                break;
        }

        return {
            x: pointX,
            y: pointY,
            nodeId,
            side
        };
    }

    private updateEdges(): void {
        this.renderer.clear();
        
        this.connections.forEach(conn => {
            const startPoint = this.getNodeAnchorPoint(conn.fromNodeId, conn.fromSide);
            const endPoint = this.getNodeAnchorPoint(conn.toNodeId, conn.toSide);
            
            if (startPoint && endPoint) {
                this.renderer.drawEdge(conn.id, startPoint, endPoint, conn.fromSide, conn.toSide);
            }
        });
    }

    public getConnections(): EdgeConnection[] {
        return [...this.connections];
    }

    public destroy(): void {
        this.nodeObserver.disconnect();
        this.resizeObserver.disconnect();
        this.renderer.destroy();
    }
}