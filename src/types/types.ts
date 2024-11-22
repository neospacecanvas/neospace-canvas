
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
    | { type: NodeType.MARKDOWN; data: MarkdownData}
    | {type: NodeType.CSV; data: CSVData};

export interface SerializedNode {
    id: string;
    position: Coordinate;
    dimensions: Dimensions;
    content: NodeContent;
    version: number;
}

export enum NodeType {
    MARKDOWN = 'markdown',
    CSV = 'csv'
}