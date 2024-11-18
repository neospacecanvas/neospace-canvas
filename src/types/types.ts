
export interface Coordinate {
    x: number;
    y: number;
}

export interface Dimensions {
    width: number;
    height: number;
}

export interface MarkdownData {
    content: string;
}

export interface CSVData {
    headers: string[];
    rows: string[][];
}

// Union type for all node types
export type NodeContent =
    | { type: CanvasNodeType.MARKDOWN; data: MarkdownData}
    | {type: CanvasNodeType.CSV; data: CSVData};

export interface SerializedNode {
    id: string;
    position: Coordinate;
    dimensions: Dimensions;
    content: NodeContent;
    version: number;
}

export enum CanvasNodeType {
    MARKDOWN = 'markdown',
    CSV = 'csv'
}