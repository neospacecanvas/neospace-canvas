import { Coordinate, SerializedNode } from "./types";

export interface ViewportState {
    scale: number;
    panOffset: Coordinate;
}
export interface GridConfig {
    size: number;
    color: string;
    visible: boolean;
}

export interface SearializedCanvas {
    version: string;
    timestamp: string;
    viewport: ViewportState;
    nodes: SerializedNode[];
}

export interface CanvasOptions {
    gridSize?: number;
    gridColor?: string;
    minScale?: number;
    maxScale?: number;
    initialViewport?: ViewportState;
}
