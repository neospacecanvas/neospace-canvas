import { Coordinate, Dimensions, NodeContent, SerializedNode } from "@/types/types";
import { v4 as uuidv4 } from 'uuid';

export class CanvasNode {
    private readonly id: string;
    private position: Coordinate;
    private dimensions: Dimensions;
    private content: NodeContent;
    private isSelected: boolean = false;

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

    getId(): string {
        return this.id;
    }

    getType(): NodeContent['type'] {
        return this.content.type;
    }

    getPosition(): Coordinate {
        return { ...this.position };
    }

    getDimensions(): Dimensions {
        return { ...this.dimensions };
    }

    getData(): NodeContent['data'] {
        return this.content.data;
    }

    isNodeSelected(): boolean {
        return this.isSelected;
    }

    // Basic setters
    setPosition(position: Coordinate): void {
        this.position = { ...position };
    }

    setDimensions(dimensions: Dimensions): void {
        this.dimensions = { ...dimensions };
    }

    select(): void {
        this.isSelected = true;
    }

    deselect(): void {
        this.isSelected = false;
    }

        // Serialization
        toJSON(): SerializedNode {
            return {
                id: this.id,
                position: this.position,
                dimensions: this.dimensions,
                content: this.content,
                version: 1  // Current version
            };
        }
    
        static fromJSON(data: SerializedNode): CanvasNode {
            return new CanvasNode(
                data.position,
                data.content,
                data.dimensions,
                data.id
            );
        }
    
        // Private helpers
        private getDefaultDimensions(): Dimensions {
            switch (this.content.type) {
                case 'markdown':
                    return { width: 200, height: 150 };
                case 'csv':
                    return { width: 400, height: 300 };
            }
            return{width: 100, height: 100};
        }
}

