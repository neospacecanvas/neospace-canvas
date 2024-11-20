import { Coordinate, Dimensions } from "@/types/types";
import { CanvasNode } from "./CanvasNode";

export class Canvas {
    private nodes: Map<string, CanvasNode>;
    // private version: string = "1.0;"

    constructor() {
        this.nodes = new Map();
    }

    addNode(node: CanvasNode): void {
        this.nodes.set(node.getId(), node);
    }

    removeNode(nodeId: string): void {
        this.nodes.delete(nodeId);
    }

    getNode(nodeId: string): CanvasNode | undefined {
        return this.nodes.get(nodeId);
    }

    getAllNodes(): CanvasNode[] {
        return Array.from(this.nodes.values());
    }

    updateNodePosition(nodeId: string, position: Coordinate): void {
        const node = this.nodes.get(nodeId);
        if (node) {
            node.setPosition(position);
        }
    }

    updateNodeDimensions(nodeId: string, dimensions: Dimensions): void {
        const node = this.nodes.get(nodeId);
        if (node) {
            node.setDimensions(dimensions);
        }
    }

    clear(): void {
        this.nodes.clear();
    }

    //TODO: remove this if we are not going to do render culling in the demo
    getNodesInViewport(viewportBounds: { topLeft: Coordinate; bottomRight: Coordinate }): CanvasNode[] {
        return this.getAllNodes().filter(node => {
            const pos = node.getPosition();
            const dim = node.getDimensions();
            return (
                pos.x < viewportBounds.bottomRight.x &&
                pos.x + dim.width > viewportBounds.topLeft.x &&
                pos.y < viewportBounds.bottomRight.y &&
                pos.y + dim.height > viewportBounds.topLeft.y
            );
        });
    }

}
