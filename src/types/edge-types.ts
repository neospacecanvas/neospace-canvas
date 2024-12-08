import { AnchorSide } from '../types/types';

export interface EdgeDrawOptions {
    stroke?: string;
    strokeWidth?: string;
    markerEnd?: string;
    className?: string;
}

export interface EdgePoint {
    x: number;
    y: number;
}

export interface EdgeCurvePoints {
    start: EdgePoint;
    end: EdgePoint;
    control1: EdgePoint;
    control2: EdgePoint;
}

export interface EdgeToolbarPosition {
    x: number;
    y: number;
}

export interface EdgeRenderState {
    isSelected: boolean;
    isHovered: boolean;
}

// Used internally by EdgeManager
export interface EdgeDrawingState {
    isDrawing: boolean;
    startNode: string | null;
    startSide: AnchorSide | null;
    tempPath: SVGPathElement | null;
}

export interface EdgeUIState {
    selectedEdgeId: string | null;
    hoveredEdgeId: string | null;
    toolbarVisible: boolean;
}

export interface EdgePathElements {
    group: SVGGElement;
    path: SVGPathElement;
    hitArea: SVGPathElement;
}