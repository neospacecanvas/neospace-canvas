
import { CanvasManager } from './CanvasManager';

export class ToolbarManager {
    private container: HTMLElement;
    private canvasManager: CanvasManager;

    constructor(containerId: string, canvasManager: CanvasManager) {
        const container = document.getElementById(containerId);
        if (!container) throw new Error('Toolbar container not found');
        
        this.container = container;
        this.canvasManager = canvasManager;
        this.setupToolbar();
    }

    private setupToolbar(): void {
        const toolbar = document.createElement('div');
        toolbar.className = 'canvas-toolbar';
        
        const tools = [
            { id: 'hand', icon: '✋', title: 'Pan Mode' },
            { id: 'select', icon: '⬚', title: 'Select Mode' },
            { id: 'markdown', icon: '📝', title: 'Add Markdown', action: () => this.handleMarkdownCreate() },
            { id: 'upload', icon: '↑', title: 'Upload CSV', action: () => this.handleCSVUpload() },
            { id: 'save', icon: '💾', title: 'Save Canvas', action: () => this.handleSave() },
            { id: 'load', icon: '📂', title: 'Load Canvas', action: () => this.handleLoad() }
        ];
    
        tools.forEach((tool, index) => {
            if (index === 2 || index === 4) {
                const divider = document.createElement('div');
                divider.className = 'toolbar-divider';
                toolbar.appendChild(divider);
            }
    
            const button = document.createElement('button');
            button.className = 'toolbar-button';
            button.setAttribute('title', tool.title);
            button.setAttribute('data-tool', tool.id);
            button.textContent = tool.icon;
    
            if (tool.action) {
                button.addEventListener('click', tool.action);
            }
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
                const content = await file.text();
                this.canvasManager.createCSVNode({
                    fileName: file.name,
                    content: content
                });
            } catch (error) {
                console.error('Error handling CSV upload:', error);
            }
        };
        
        input.click();
    }

    private handleMarkdownCreate(): void {
        this.canvasManager.createMarkdownNode();
    }

    private async handleSave(): Promise<void> {
        const state = this.canvasManager.saveState();
        const blob = new Blob([JSON.stringify(state, null, 2)], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        
        const a = document.createElement('a');
        a.href = url;
        a.download = `canvas-${new Date().toISOString().slice(0, 10)}.json`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    }

    private handleLoad(): void {
        const input = document.createElement('input');
        input.type = 'file';
        input.accept = '.json';

        input.onchange = async (e: Event) => {
            const file = (e.target as HTMLInputElement).files?.[0];
            if (!file) return;

            try {
                const content = await file.text();
                const state = JSON.parse(content);
                this.canvasManager.loadState(state);
            } catch (error) {
                console.error('Error loading canvas state:', error);
            }
        };

        input.click();
    }

    public destroy(): void {
        this.container.innerHTML = '';
    }
}