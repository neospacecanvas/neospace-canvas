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
    fileName: string;
    headers: string[];
    rows: string[][];
}

export type NodeContent =
    | { type: NodeType.MARKDOWN; data: MarkdownData }
    | { type: NodeType.CSV; data: CSVData };

export type AnchorSide = 'top' | 'right' | 'bottom' | 'left';
export type ArrowEnd = 'arrow' | 'none';

export interface Edge {
    id: string;
    fromNode: string;
    toNode: string;
    fromSide: AnchorSide;
    toSide: AnchorSide;
    toEnd: ArrowEnd;
}

export interface SerializedNode {
    id: string;
    position: Coordinate;
    dimensions: Dimensions;
    content: NodeContent;
    version: number;
}

export interface ViewportState {
    scale: number;
    panX: number;
    panY: number;
}

export interface CanvasState {
    version: string;
    timestamp: string;
    viewport: ViewportState;
    nodes: SerializedNode[];
    edges: Edge[];
}

export enum NodeType {
    MARKDOWN = 'markdown',
    CSV = 'csv'
}

export interface NodeCreateOptions {
    id?: string;
    position?: Coordinate;
    dimensions?: Dimensions;
    content?: NodeContent;
}