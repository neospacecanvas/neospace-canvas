
interface EdgeToolbarOptions {
    position: { x: number; y: number };
    onDelete: () => void;
}

export class EdgeToolbar {
    private element: HTMLElement;
    private position: { x: number; y: number };
    private readonly TOOLBAR_OFFSET = 20;

    constructor(options: EdgeToolbarOptions) {
        this.position = options.position;
        this.element = this.createToolbarElement();
        this.setupDeleteButton(options.onDelete);
        this.positionToolbar();
        document.body.appendChild(this.element);
    }

    private createToolbarElement(): HTMLElement {
        const toolbar = document.createElement('div');
        toolbar.className = 'edge-toolbar';
        toolbar.style.position = 'absolute';
        toolbar.style.zIndex = '1000';
        toolbar.style.padding = '4px';
        toolbar.style.background = 'white';
        toolbar.style.borderRadius = '4px';
        toolbar.style.boxShadow = '0 2px 4px rgba(0,0,0,0.1)';
        toolbar.style.display = 'flex';
        toolbar.style.gap = '4px';
        toolbar.style.userSelect = 'none';
        return toolbar;
    }

    private setupDeleteButton(onDelete: () => void): void {
        const deleteBtn = document.createElement('button');
        deleteBtn.className = 'toolbar-button';
        deleteBtn.innerHTML = '🗑️';
        deleteBtn.title = 'Delete Edge';
        deleteBtn.style.border = 'none';
        deleteBtn.style.background = 'transparent';
        deleteBtn.style.cursor = 'pointer';
        deleteBtn.style.padding = '4px';
        deleteBtn.style.display = 'flex';
        deleteBtn.style.alignItems = 'center';
        deleteBtn.style.justifyContent = 'center';
        deleteBtn.style.width = '28px';
        deleteBtn.style.height = '28px';
        deleteBtn.style.borderRadius = '4px';

        deleteBtn.addEventListener('mouseover', () => {
            deleteBtn.style.background = '#f3f4f6';
        });

        deleteBtn.addEventListener('mouseout', () => {
            deleteBtn.style.background = 'transparent';
        });

        deleteBtn.addEventListener('click', (e) => {
            e.stopPropagation();
            onDelete();
        });

        this.element.appendChild(deleteBtn);
    }

    private positionToolbar(): void {
        const computedStyle = getComputedStyle(document.documentElement);
        const scale = computedStyle.getPropertyValue('--scale');
        const panX = computedStyle.getPropertyValue('--pan-x');
        const panY = computedStyle.getPropertyValue('--pan-y');
        const scaleValue = parseFloat(scale) || 1;
        const panXValue = parseInt(panX) || 0;
        const panYValue = parseInt(panY) || 0;
    
        const x = this.position.x * scaleValue + panXValue;
        const y = this.position.y * scaleValue + panYValue - this.TOOLBAR_OFFSET;
    
        this.element.style.left = `${x}px`;
        this.element.style.top = `${y}px`;
        this.element.style.transform = 'translate(-50%, -100%)';
    }

    public updatePosition(position: { x: number; y: number }): void {
        this.position = position;
        this.positionToolbar();
    }

    public destroy(): void {
        this.element.remove();
    }
}