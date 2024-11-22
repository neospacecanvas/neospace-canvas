// ToolbarManager.ts
export class ToolbarManager {
    private container: HTMLElement;
    private onNodeCreate?: (type: string , date: any) => void;


    constructor(containerId: string, onNodeCreate?: (type: string , date: any) => void) {
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
            { id: 'text', icon: 'T', title: 'Text Mode' },
            { id: 'upload', icon: '↑', title: 'Upload File', action: this.handleCSVUpload.bind(this) }
        ];
    
        tools.forEach((tool, index) => {
            if (index === 3) {
                const divider = document.createElement('div');
                divider.className = 'toolbar-divider';
                toolbar.appendChild(divider);
            }
    
            const button = document.createElement('button');
            button.className = 'toolbar-button';
            button.setAttribute('title', tool.title);
            button.setAttribute('data-tool', tool.id);
            button.textContent = tool.icon;
    
            // Add specific action if exists, otherwise use default log
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
            if (!file || !this.onNodeCreate) return;
            
            try {
                console.log('File selected:', file.name);
                const text = await file.text();
                console.log('File content:', text.substring(0, 100) + '...');
                this.onNodeCreate('csv', { fileName: file.name, content: text });
            } catch (error) {
                console.error('Error reading CSV file:', error);
            }
        };
        
        input.click();
    }
}