import { Coordinate, Dimensions, NodeContent } from "@/types/types";
import { v4 as uuidv4 } from 'uuid';

export abstract class Node {
    private readonly id: string;
    private position: Coordinate;
    private dimensions: Dimensions;
    protected content: NodeContent;  // Made protected so derived classes can access

    constructor(
        position: Coordinate,
        content: NodeContent,
        dimensions?: Dimensions,
        id?: string
    ) {
        this.id = id || uuidv4();
        this.position = position;
        this.content = content;
        this.dimensions = dimensions || this.getDefaultDimensions();
    }

    // Common methods remain the samet
    getId(): string { return this.id; }
    getType(): NodeContent['type'] { return this.content.type; }
    getPosition(): Coordinate { return { ...this.position }; }
    getDimensions(): Dimensions { return { ...this.dimensions }; }

    // Make this protected abstract to force derived classes to implement their own rendering
    protected abstract renderContent(): string;

    // Template method that handles common node structure
    createNodeElement(): HTMLElement {
        const element = document.createElement('div');
        element.id = this.getId();
        element.className = `node ${this.getNodeClassName()}`;
        
        const pos = this.getPosition();
        const dim = this.getDimensions();
        
        element.style.left = `${pos.x}px`;
        element.style.top = `${pos.y}px`;
        element.style.width = `${dim.width}px`;
        element.style.height = `${dim.height}px`;
        
        element.innerHTML = this.renderContent();
        return element;
    }

    // Allow derived classes to add their own classes
    protected getNodeClassName(): string {
        return '';
    }

    protected getDefaultDimensions(): Dimensions {
        return { width: 200, height: 150 };  // Sensible default size
    }
}
