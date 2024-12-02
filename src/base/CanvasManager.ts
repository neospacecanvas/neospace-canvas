import { NodeStore } from './NodeStore';
import { MarkdownNode } from './MarkdownNode';
import { CSVNode } from './CSVNode';
import { NodeType } from '../types/types';
import { ToolbarManager } from './ToolbarManager';

export class CanvasManager {
    private nodeStore: NodeStore;
    private toolbarManager: ToolbarManager;
    private nodesContainer: HTMLElement;

    constructor(appId: string) {
        // Find the existing canvas container
        const container = document.querySelector('.canvas-container');
        if (!container) {
            throw new Error('Canvas container not found');
        }

        // Create nodes container if it doesn't exist
        let nodesContainer = container.querySelector('#canvas-nodes');
        if (!nodesContainer) {
            nodesContainer = document.createElement('div');
            nodesContainer.id = 'canvas-nodes';
            container.appendChild(nodesContainer);
        }
        this.nodesContainer = nodesContainer as HTMLElement;

        // Initialize stores and managers
        this.nodeStore = NodeStore.getInstance();
        
        // Initialize toolbar with node creation callback
        this.toolbarManager = new ToolbarManager('toolbar-container', (type, data) => {
            this.handleNodeCreate(type, data);
        });

        this.setupEventListeners();
    }

    private handleNodeCreate(type: string, data: any) {
        let node;
        switch (type) {
            case NodeType.CSV: {
                if (!data.fileName || !data.content) {
                    console.error('Missing required CSV data');
                    return;
                }
                node = new CSVNode(data.fileName, data.content);
                break;
            }
            case NodeType.MARKDOWN: {
                node = new MarkdownNode();
                break;
            }
            default: {
                console.error('Unknown node type:', type);
                return;
            }
        }

        if (node) {
            this.nodesContainer.appendChild(node.getElement());
        }
    }

    private setupEventListeners() {
        // Handle node deletion
        document.addEventListener('keydown', (e) => {
            // Only handle if the target is not an input or contenteditable element
            const target = e.target as HTMLElement;
            if (target.isContentEditable || target.tagName === 'INPUT') {
                return;
            }

            if (e.key === 'Delete' || e.key === 'Backspace') {
                const selectedNode = document.querySelector('.node.is-selected');
                if (selectedNode) {
                    const nodeId = selectedNode.id.replace('node-', '');
                    this.nodeStore.deleteNode(nodeId);
                    selectedNode.remove();
                }
            }
        });

        // Handle space bar for panning mode
        document.addEventListener('keydown', (e) => {
            if (e.code === 'Space' && !e.repeat) {
                const container = document.querySelector('.canvas-container');
                if (container) {
                    container.classList.add('panning');
                    container.style.cursor = 'grab';
                }
            }
        });

        document.addEventListener('keyup', (e) => {
            if (e.code === 'Space') {
                const container = document.querySelector('.canvas-container');
                if (container) {
                    container.classList.remove('panning');
                    container.style.cursor = '';
                }
            }
        });

        // Deselect nodes when clicking canvas background
        this.nodesContainer.addEventListener('click', (e) => {
            if (e.target === this.nodesContainer) {
                const selectedNodes = document.querySelectorAll('.node.is-selected');
                selectedNodes.forEach(node => {
                    node.classList.remove('is-selected');
                    const toolbar = node.querySelector('.node-toolbar') as HTMLElement;
                    if (toolbar) {
                        toolbar.style.display = 'none';
                    }
                });
            }
        });
    }

    // Method to get all nodes (useful for saving state)
    public getAllNodes(): string[] {
        return Array.from(this.nodesContainer.children)
            .map(node => node.id)
            .filter(id => id.startsWith('node-'))
            .map(id => id.replace('node-', ''));
    }

    // Method to clear the canvas
    public clearCanvas() {
        while (this.nodesContainer.firstChild) {
            const nodeId = this.nodesContainer.firstChild.id.replace('node-', '');
            this.nodeStore.deleteNode(nodeId);
            this.nodesContainer.firstChild.remove();
        }
    }

    // Method to save the current state
    public saveState() {
        return this.nodeStore.serialize();
    }

    // Method to destroy the canvas manager
    public destroy() {
        this.clearCanvas();
        // Remove event listeners if needed
        // Note: The ones attached to document will persist
    }
}