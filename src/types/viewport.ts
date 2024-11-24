
export interface ViewportState {
    scale: number;
    panX: number;
    panY: number;
}

export interface ViewportConstraints {
    MIN_SCALE: number;
    MAX_SCALE: number;
}

// can zoom between 10% and 400% (.1x and 4x)
export const DEFAULT_VIEWPORT_CONSTRAINTS: ViewportConstraints = {
    MIN_SCALE: 0.1,
    MAX_SCALE: 4.0
};