// ToolbarManager.ts
export class ToolbarManager {
    private container: HTMLElement;

    constructor(containerId: string) {
        const container = document.getElementById(containerId);
        if (!container) throw new Error('Toolbar container not found');
        
        this.container = container;
        this.setupToolbar();
    }

    private setupToolbar(): void {
        const toolbar = document.createElement('div');
        toolbar.className = 'canvas-toolbar';
        
        const tools = [
            { id: 'hand', icon: '✋', title: 'Pan Mode' },
            { id: 'select', icon: '⬚', title: 'Select Mode' },
            { id: 'text', icon: 'T', title: 'Text Mode' },
            { id: 'upload', icon: '↑', title: 'Upload File' }
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
            button.textContent = tool.icon;

            button.addEventListener('click', () => console.log('Clicked:', tool.id));
            toolbar.appendChild(button);
        });

        this.container.appendChild(toolbar);
    }
}