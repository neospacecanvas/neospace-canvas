import { ViewportState } from '@/types/canvas';

export class WebGLGridManager {
    private readonly canvas: HTMLCanvasElement;
    private readonly gl: WebGLRenderingContext;
    private program: WebGLProgram;
    private scale: number = 1;
    private panOffset: { x: number; y: number } = { x: 0, y: 0 };
    private gridSize: number = 20;
    // private isDragging: boolean = false;
    // private lastMousePos: { x: number; y: number } | null = null;

    // Shader locations
    private positionLocation: number = -1;
    private resolutionLocation: WebGLUniformLocation | null = null;
    private scaleLocation: WebGLUniformLocation | null = null;
    private panOffsetLocation: WebGLUniformLocation | null = null;
    private gridSizeLocation: WebGLUniformLocation | null = null;
    private dotColorLocation: WebGLUniformLocation | null = null;

    constructor(container: HTMLElement) {
        // Create and setup canvas
        this.canvas = document.createElement('canvas');
        this.canvas.style.position = 'absolute';
        this.canvas.style.top = '0';
        this.canvas.style.left = '0';
        this.canvas.style.width = '100%';
        this.canvas.style.height = '100%';
        this.canvas.style.pointerEvents = 'none';
        container.appendChild(this.canvas);

        // Initialize WebGL
        const gl = this.canvas.getContext('webgl', {
            antialias: true,
            alpha: true
        });
        if (!gl) throw new Error('WebGL not supported');
        this.gl = gl;

        // Enable alpha blending
        this.gl.enable(this.gl.BLEND);
        this.gl.blendFunc(this.gl.SRC_ALPHA, this.gl.ONE_MINUS_SRC_ALPHA);

        // Create shaders and program
        const program = this.initShaders();
        if (!program) throw new Error('Failed to create shader program');
        this.program = program;

        // Setup program and start rendering
        this.setupProgram();
        this.resize();
        this.render();
    }

    public updateViewport(viewport: ViewportState): void {
        this.scale = viewport.scale;
        this.panOffset = { ...viewport.panOffset };
    }

    public setGridSize(size: number): void {
        this.gridSize = size;
    }

    public destroy(): void {
        this.canvas.remove();
        if (this.program) {
            this.gl.deleteProgram(this.program);
        }
    }

    private createShader(type: number, source: string): WebGLShader | null {
        const shader = this.gl.createShader(type);
        if (!shader) return null;

        this.gl.shaderSource(shader, source);
        this.gl.compileShader(shader);

        if (!this.gl.getShaderParameter(shader, this.gl.COMPILE_STATUS)) {
            console.error('Shader compile error:', this.gl.getShaderInfoLog(shader));
            this.gl.deleteShader(shader);
            return null;
        }
        return shader;
    }

    private initShaders(): WebGLProgram | null {
        const vertexShaderSource = `
            attribute vec2 position;
            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
        `;

        const fragmentShaderSource = `
            precision highp float;
            uniform vec2 uResolution;
            uniform float uScale;
            uniform vec2 uPanOffset;
            uniform float uGridSize;
            uniform vec3 uDotColor;
            
            float drawDot(vec2 point, vec2 center, float radius) {
                float dist = length(point - center);
                return smoothstep(radius, radius - 1.0, dist);
            }
            
            void main() {
                vec2 pixelCoord = gl_FragCoord.xy;
                vec2 adjustedCoord = (pixelCoord + uPanOffset) / uScale;
                float scaledGridSize = uGridSize * uScale;
                vec2 gridPos = mod(adjustedCoord, vec2(scaledGridSize));
                vec2 cellCenter = vec2(scaledGridSize * 0.5);
                float dotRadius = min(2.0, max(1.0, uScale * 0.5));
                float dot = drawDot(gridPos, cellCenter, dotRadius);
                gl_FragColor = vec4(uDotColor, dot);
            }
        `;

        const vertexShader = this.createShader(this.gl.VERTEX_SHADER, vertexShaderSource);
        const fragmentShader = this.createShader(this.gl.FRAGMENT_SHADER, fragmentShaderSource);

        if (!vertexShader || !fragmentShader) return null;

        const program = this.gl.createProgram();
        if (!program) return null;

        this.gl.attachShader(program, vertexShader);
        this.gl.attachShader(program, fragmentShader);
        this.gl.linkProgram(program);

        if (!this.gl.getProgramParameter(program, this.gl.LINK_STATUS)) {
            console.error('Program link error:', this.gl.getProgramInfoLog(program));
            return null;
        }

        return program;
    }

    private setupProgram(): void {
        const positions = new Float32Array([
            -1, -1,
             1, -1,
            -1,  1,
             1,  1
        ]);

        const buffer = this.gl.createBuffer();
        this.gl.bindBuffer(this.gl.ARRAY_BUFFER, buffer);
        this.gl.bufferData(this.gl.ARRAY_BUFFER, positions, this.gl.STATIC_DRAW);

        this.positionLocation = this.gl.getAttribLocation(this.program, 'position');
        this.resolutionLocation = this.gl.getUniformLocation(this.program, 'uResolution');
        this.scaleLocation = this.gl.getUniformLocation(this.program, 'uScale');
        this.panOffsetLocation = this.gl.getUniformLocation(this.program, 'uPanOffset');
        this.gridSizeLocation = this.gl.getUniformLocation(this.program, 'uGridSize');
        this.dotColorLocation = this.gl.getUniformLocation(this.program, 'uDotColor');

        this.gl.enableVertexAttribArray(this.positionLocation);
        this.gl.vertexAttribPointer(this.positionLocation, 2, this.gl.FLOAT, false, 0, 0);
    }

    public resize = (): void => {
        const { width, height } = this.canvas.parentElement!.getBoundingClientRect();
        const dpr = window.devicePixelRatio || 1;
        
        this.canvas.width = width * dpr;
        this.canvas.height = height * dpr;
        this.gl.viewport(0, 0, this.canvas.width, this.canvas.height);
    };

    private render = (): void => {
        this.gl.clearColor(0.118, 0.118, 0.118, 1.0);
        this.gl.clear(this.gl.COLOR_BUFFER_BIT);

        this.gl.useProgram(this.program);
        this.gl.uniform2f(this.resolutionLocation!, this.canvas.width, this.canvas.height);
        this.gl.uniform1f(this.scaleLocation!, this.scale);
        this.gl.uniform2f(this.panOffsetLocation!, this.panOffset.x, this.panOffset.y);
        this.gl.uniform1f(this.gridSizeLocation!, this.gridSize);
        this.gl.uniform3f(this.dotColorLocation!, 0.647, 0.847, 1.0); // Light blue dots

        this.gl.drawArrays(this.gl.TRIANGLE_STRIP, 0, 4);

        requestAnimationFrame(this.render);
    };
}