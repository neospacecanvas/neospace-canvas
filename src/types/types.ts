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

export type AnchorSide = 'top' | 'right' | 'bottom' | 'left';

export interface Edge {
    id: string;
    fromNode: string;
    toNode: string;
    fromSide: AnchorSide;
    toSide: AnchorSide;
    toEnd: 'arrow' | 'none';
}

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