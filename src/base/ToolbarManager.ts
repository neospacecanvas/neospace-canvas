import { MarkdownNode } from "./MarkdownNode";
import { CSVNode } from "./CSVNode";

// ToolbarManager.ts
export class ToolbarManager {
    private container: HTMLElement;
    private onNodeCreate?: (type: string, data: any) => void;

    constructor(containerId: string, onNodeCreate?: (type: string, data: any) => void) {
        const container = document.getElementById(containerId);
        if (!container) throw new Error('Toolbar container not found');
        
        this.container = container;
        this.onNodeCreate = onNodeCreate;
        this.setupToolbar();
    }

    private setupToolbar(): void {
        const toolbar = document.createElement('div');
        toolbar.className = 'canvas-toolbar';
        
        const tools = [
            { id: 'hand', icon: '✋', title: 'Pan Mode' },
            { id: 'select', icon: '⬚', title: 'Select Mode' },
            { id: 'markdown', icon: '📝', title: 'Add Markdown', action: () => this.handleMarkdownCreate() },
            { id: 'upload', icon: '↑', title: 'Upload File', action: () => this.handleCSVUpload() }
        ];
    
        tools.forEach((tool, index) => {
            if (index === 2) {
                const divider = document.createElement('div');
                divider.className = 'toolbar-divider';
                toolbar.appendChild(divider);
            }
    
            const button = document.createElement('button');
            button.className = 'toolbar-button';
            button.setAttribute('title', tool.title);
            button.setAttribute('data-tool', tool.id);
            button.textContent = tool.icon;
    
            button.addEventListener('click', tool.action || (() => console.log('Clicked:', tool.id)));
            toolbar.appendChild(button);
        });
    
        this.container.appendChild(toolbar);
    }
    
    private async handleCSVUpload(): Promise<void> {
        const input = document.createElement('input');
        input.type = 'file';
        input.accept = '.csv';
        
        input.onchange = async (e: Event) => {
            const file = (e.target as HTMLInputElement).files?.[0];
            if (!file) return;
            
            try {
                // First, read the file content
                const content = await file.text();
                
                // Create the node data
                const nodeData = {
                    fileName: file.name,
                    content: content
                };

                // Notify parent through callback
                if (this.onNodeCreate) {
                    this.onNodeCreate('csv', nodeData);
                } else {
                    // Fallback to direct creation if no callback provided
                    const node = new CSVNode(file.name);
                    document.getElementById('canvas-nodes')?.appendChild(node.getElement());
                }
            } catch (error) {
                console.error('Error handling CSV upload:', error);
            }
        };
        
        input.click();
    }

    private handleMarkdownCreate(): void {
        if (this.onNodeCreate) {
            this.onNodeCreate('markdown', { content: '' });
        } else {
            // Fallback to direct creation
            const node = new MarkdownNode();
            document.getElementById('canvas-nodes')?.appendChild(node.getElement());
        }
    }
}