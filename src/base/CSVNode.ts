import { ViewportManager } from './Viewport';
import { NodeStore } from './NodeStore';
import { NodeType } from '../types/types';

export class CSVNode {
    private element: HTMLElement;
    private toolbar: HTMLElement;
    private isDragging: boolean = false;
    private isSpacePressed: boolean = false;
    private readonly TOOLBAR_OFFSET = -45;
    private viewportManager: ViewportManager;
    private nodeStore: NodeStore;
    private id: string;
    private unsubscribeViewport: () => void;
    private unsubscribeStore: () => void;
    
    constructor(fileName: string, csvContent: string, x: number = window.innerWidth/2, y: number = window.innerHeight/2) {
        this.viewportManager = ViewportManager.getInstance();
        this.nodeStore = NodeStore.getInstance();

        // Create node in store
        this.id = this.nodeStore.createNode(
            NodeType.CSV,
            { x, y },
            { width: 110, height: 130 },
            csvContent
        );
        
        this.setupElement();
        this.setupContent(fileName);
        this.setupToolbar();
        this.setupDrag();
        this.setupHoverEffects();
        
        // Subscribe to viewport changes
        this.unsubscribeViewport = this.viewportManager.subscribe(() => {});

        // Subscribe to store updates
        this.unsubscribeStore = this.nodeStore.subscribe(this.id, (_, node) => {
            if (node.position.x !== parseInt(this.element.style.left) ||
                node.position.y !== parseInt(this.element.style.top)) {
                this.element.style.left = `${node.position.x}px`;
                this.element.style.top = `${node.position.y}px`;
            }

            if (node.dimensions.width !== parseInt(this.element.style.width) ||
                node.dimensions.height !== parseInt(this.element.style.height)) {
                this.element.style.width = `${node.dimensions.width}px`;
                this.element.style.height = `${node.dimensions.height}px`;
            }
        });
    }

    private setupElement() {
        this.element = document.createElement('div');
        this.element.id = `node-${this.id}`;
        this.element.className = 'node csv-node';
        this.element.style.position = 'absolute';
        
        const node = this.nodeStore.getNode(this.id);
        if (node) {
            this.element.style.left = `${node.position.x}px`;
            this.element.style.top = `${node.position.y}px`;
            this.element.style.width = `${node.dimensions.width}px`;
            this.element.style.height = `${node.dimensions.height}px`;
        }
    }

    private setupContent(fileName: string) {
        const content = document.createElement('div');
        content.className = 'node-content';
        
        // Icon container
        const iconContainer = document.createElement('div');
        iconContainer.className = 'node-icon';
        
        // CSV Icon
        const icon = document.createElement('img');
        icon.src = '../../public/assets/csv.png';
        icon.className = 'csv-icon';
        icon.alt = 'CSV file';
        icon.draggable = false; // Prevent image dragging
        
        // Prevent default drag behaviors
        icon.addEventListener('dragstart', (e) => {
            e.stopPropagation();
        });
        
        iconContainer.appendChild(icon);
        
        // Filename
        const fileNameElement = document.createElement('div');
        fileNameElement.className = 'node-filename';
        fileNameElement.textContent = fileName;
        
        content.appendChild(iconContainer);
        content.appendChild(fileNameElement);
        this.element.appendChild(content);
        
        // Prevent drag events on the entire content
        content.addEventListener('dragstart', (e) => {
            e.preventDefault();
        });
    }

    private setupToolbar() {
        this.toolbar = document.createElement('div');
        this.toolbar.className = 'node-toolbar';
        this.toolbar.style.display = 'none';
        this.toolbar.style.top = `${this.TOOLBAR_OFFSET}px`;
        
        const tools = [
            { icon: '📋', label: 'Duplicate', action: () => this.duplicate() },
            { icon: '🗑️', label: 'Delete', action: () => this.destroy() }
        ];

        tools.forEach(tool => {
            const button = document.createElement('button');
            button.textContent = tool.icon;
            button.title = tool.label;
            button.className = 'toolbar-button';
            button.onclick = (e) => {
                e.stopPropagation();
                tool.action();
            };
            this.toolbar.appendChild(button);
        });

        this.element.appendChild(this.toolbar);
        
        this.element.addEventListener('click', () => {
            document.querySelectorAll('.node-toolbar').forEach(toolbar => {
                if (toolbar !== this.toolbar) {
                    (toolbar as HTMLElement).style.display = 'none';
                }
            });
            this.toolbar.style.display = 'flex';
            this.element.classList.add('is-selected');
        });

        document.addEventListener('click', (e) => {
            if (!this.element.contains(e.target as Node)) {
                this.toolbar.style.display = 'none';
                this.element.classList.remove('is-selected');
            }
        });
    }

    private setupDrag() {
        let startX: number;
        let startY: number;
        let currentLeft: number;
        let currentTop: number;
        
        this.element.addEventListener('mousedown', (e) => {
            if (this.isSpacePressed) return;
            
            this.isDragging = true;
            this.element.classList.add('is-dragging');
            
            startX = e.clientX;
            startY = e.clientY;
            
            currentLeft = parseInt(this.element.style.left) || 0;
            currentTop = parseInt(this.element.style.top) || 0;
            
            e.stopPropagation();
        });
        
        window.addEventListener('mousemove', (e) => {
            if (!this.isDragging) return;
            
            const { scale } = this.viewportManager.getState();
            const dx = (e.clientX - startX) / scale;
            const dy = (e.clientY - startY) / scale;
            
            currentLeft += dx;
            currentTop += dy;
            
            // Update store with new position
            this.nodeStore.updateNodePosition(this.id, {
                x: currentLeft,
                y: currentTop
            });
            
            startX = e.clientX;
            startY = e.clientY;
        });
        
        window.addEventListener('mouseup', () => {
            if (this.isDragging) {
                this.isDragging = false;
                this.element.classList.remove('is-dragging');
            }
        });
    }

    private setupHoverEffects() {
        this.element.addEventListener('mouseenter', () => {
            document.body.style.cursor = 'grab';
        });

        this.element.addEventListener('mouseleave', () => {
            if (!this.isDragging) {
                document.body.style.cursor = '';
            }
        });
    }

    private duplicate() {
        const node = this.nodeStore.getNode(this.id);
        if (!node || node.content.type !== NodeType.CSV) return;

        // Convert CSV data back to string format for the new node
        const csvContent = [
            node.content.data.headers.join(','),
            ...node.content.data.rows.map(row => row.join(','))
        ].join('\n');

        const newNode = new CSVNode(
            this.element.querySelector('.node-filename')?.textContent || 'Untitled.csv',
            csvContent,
            parseInt(this.element.style.left) + 20,
            parseInt(this.element.style.top) + 20
        );
        document.getElementById('canvas-nodes')?.appendChild(newNode.getElement());
    }

    public destroy() {
        this.nodeStore.deleteNode(this.id);
        this.unsubscribeViewport?.();
        this.unsubscribeStore?.();
        this.element.remove();
    }

    getElement(): HTMLElement {
        return this.element;
    }
}