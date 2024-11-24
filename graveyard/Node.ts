import { Coordinate, Dimensions, NodeContent } from "@/types/types";
import { v4 as uuidv4 } from 'uuid';

export abstract class Node {
    private readonly id: string;
    private position: Coordinate;
    private dimensions: Dimensions;
    protected content: NodeContent;

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

    // Concrete methods for state management
    getId(): string { return this.id; }
    getType(): NodeContent['type'] { return this.content.type; }
    getPosition(): Coordinate { return { ...this.position }; }
    getDimensions(): Dimensions { return { ...this.dimensions }; }
    
    setPosition(position: Coordinate): void {
        this.position = { ...position };
        // Optionally trigger a DOM update if needed
        this.updateDOMPosition();
    }

    setDimensions(dimensions: Dimensions): void {
        this.dimensions = { ...dimensions };
        // Optionally trigger a DOM update if needed
        this.updateDOMDimensions();
    }

    // Optional DOM update methods that derived classes can override
    protected updateDOMPosition(): void {
        const element = document.getElementById(this.id);
        if (element) {
            element.style.left = `${this.position.x}px`;
            element.style.top = `${this.position.y}px`;
        }
    }

    protected updateDOMDimensions(): void {
        const element = document.getElementById(this.id);
        if (element) {
            element.style.width = `${this.dimensions.width}px`;
            element.style.height = `${this.dimensions.height}px`;
        }
    }

    // Abstract method for content rendering
    protected abstract renderContent(): string;

    protected getDefaultDimensions(): Dimensions {
        return { width: 200, height: 150 };
    }

    // Template method for creating the full node element
    createNodeElement(): HTMLElement {
        const element = document.createElement('div');
        element.id = this.getId();
        element.className = `node ${this.getNodeClassName()}`;
        
        element.style.left = `${this.position.x}px`;
        element.style.top = `${this.position.y}px`;
        element.style.width = `${this.dimensions.width}px`;
        element.style.height = `${this.dimensions.height}px`;
        
        element.innerHTML = this.renderContent();
        return element;
    }

    protected getNodeClassName(): string {
        return '';
    }
}