import { v4 as uuidv4 } from 'uuid';

export class MarkdownNode {
    private element: HTMLElement;
    private header: HTMLElement;
    private content: HTMLElement;
    private toolbar: HTMLElement;
    private isDragging: boolean = false;
    private isSpacePressed: boolean = false;
    private scale: number = 1;
    private readonly TOOLBAR_OFFSET = -55;
    
    constructor(x: number = window.innerWidth/2, y: number = window.innerHeight/2) {
        this.element = document.createElement('div');
        this.element.id = 'node-' + uuidv4();
        this.element.className = 'node markdown-node';
        this.element.style.left = x + 'px';
        this.element.style.top = y + 'px';
        this.element.style.width = '480px';
        this.element.style.position = 'absolute';
        
        this.setupHeader();
        this.setupContent();
        this.setupToolbar();
        this.setupDrag();
        this.setupResize();
        this.setupHoverEffects();
        this.setupFormatTracking();
    }

    private setupHeader() {
        this.header = document.createElement('div');
        this.header.className = 'node-header';
        this.header.textContent = 'Markdown';
        this.element.appendChild(this.header);
    }

    private setupContent() {
        this.content = document.createElement('div');
        this.content.className = 'node-content';
        this.content.contentEditable = 'true';
        this.content.innerHTML = '<p>Click to edit text</p>';
        
        this.content.addEventListener('keydown', (e) => {
            if (e.code === 'Space') {
                e.stopPropagation();
            }
        });
        
        this.content.addEventListener('paste', (e) => {
            e.preventDefault();
            const text = e.clipboardData?.getData('text/plain');
            if (text) {
                document.execCommand('insertText', false, text);
            }
        });

        this.element.appendChild(this.content);
    }

    private setupToolbar() {
        this.toolbar = document.createElement('div');
        this.toolbar.className = 'node-toolbar';
        this.toolbar.style.display = 'none';
        this.toolbar.style.top = `${this.TOOLBAR_OFFSET}px`;
        
        const tools = [
            { icon: '𝐁', label: 'Bold', action: () => this.toggleFormat('bold'), isFormat: true, format: 'bold' },
            { icon: '𝐼', label: 'Italic', action: () => this.toggleFormat('italic'), isFormat: true, format: 'italic' },
            { icon: '̲U̲', label: 'Underline', action: () => this.toggleFormat('underline'), isFormat: true, format: 'underline' },
            { icon: '•', label: 'Bullet List', action: () => document.execCommand('insertUnorderedList'), isFormat: false },
            { icon: '1.', label: 'Numbered List', action: () => document.execCommand('insertOrderedList'), isFormat: false },
            { icon: '📋', label: 'Duplicate', action: (e: Event) => {
                e.stopPropagation();
                this.duplicate();
            }},
            { icon: '🗑️', label: 'Delete', action: (e: Event) => {
                e.stopPropagation();
                this.element.remove();
            }}
        ];

        tools.forEach(tool => {
            const button = document.createElement('button');
            button.textContent = tool.icon;
            button.title = tool.label;
            if (tool.isFormat) {
                button.setAttribute('data-format', tool.format);
            }
            button.onclick = tool.action as any;
            button.className = `toolbar-button${tool.isFormat ? ' format-button' : ''}`;
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

    private toggleFormat(format: string) {
        document.execCommand(format);
        this.updateFormatButtons();
    }

    private setupFormatTracking() {
        document.addEventListener('selectionchange', () => {
            const selection = window.getSelection();
            if (!selection?.rangeCount) return;
            
            const range = selection.getRangeAt(0);
            if (!this.content.contains(range.commonAncestorContainer)) return;

            this.updateFormatButtons();
        });

        this.content.addEventListener('input', () => {
            this.updateFormatButtons();
        });
    }

    private updateFormatButtons() {
        const formats = {
            'bold': document.queryCommandState('bold'),
            'italic': document.queryCommandState('italic'),
            'underline': document.queryCommandState('underline')
        };

        this.toolbar.querySelectorAll('.format-button').forEach((button: Element) => {
            const format = button.getAttribute('data-format');
            if (format && format in formats) {
                if (formats[format as keyof typeof formats]) {
                    button.classList.add('active');
                } else {
                    button.classList.remove('active');
                }
            }
        });
    }

    private setupDrag() {
        let startX = 0;
        let startY = 0;
        
        this.header.addEventListener('mousedown', (e) => {
            if (this.isSpacePressed) return;
            
            this.isDragging = true;
            this.element.classList.add('is-dragging');
            
            startX = e.clientX;
            startY = e.clientY;
            
            e.stopPropagation();
        });
        
        window.addEventListener('mousemove', (e) => {
            if (!this.isDragging) return;
            
            const dx = (e.clientX - startX) / this.scale;
            const dy = (e.clientY - startY) / this.scale;
            
            const currentLeft = parseInt(this.element.style.left) || 0;
            const currentTop = parseInt(this.element.style.top) || 0;
            
            this.element.style.left = `${currentLeft + dx}px`;
            this.element.style.top = `${currentTop + dy}px`;
            
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

    private setupResize() {
        const directions = ['e', 'w'];
        const handles = directions.map(direction => {
            const handle = document.createElement('div');
            handle.className = `resize-handle resize-${direction}`;
            return handle;
        });
        
        let isResizing = false;
        let startWidth: number;
        let startX: number;
        
        handles.forEach(handle => {
            handle.addEventListener('mousedown', (e) => {
                isResizing = true;
                startWidth = this.element.offsetWidth;
                startX = e.clientX;
                e.stopPropagation();
                document.body.style.cursor = 'ew-resize';
            });
            
            this.element.appendChild(handle);
        });
        
        window.addEventListener('mousemove', (e) => {
            if (!isResizing) return;
            
            const dx = (e.clientX - startX) / this.scale;
            const side = (e.target as Element).className.includes('resize-e') ? 1 : -1;
            const newWidth = startWidth + (dx * side);
            
            if (newWidth >= 200 && newWidth <= 800) {
                this.element.style.width = `${newWidth}px`;
                if (side === -1) {
                    this.element.style.left = `${parseInt(this.element.style.left) + dx}px`;
                }
            }
        });
        
        window.addEventListener('mouseup', () => {
            if (isResizing) {
                isResizing = false;
                document.body.style.cursor = '';
            }
        });
    }

    private setupHoverEffects() {
        this.element.addEventListener('mouseenter', () => {
            document.body.style.cursor = 'default';
        });

        this.element.addEventListener('mouseleave', () => {
            if (!this.isDragging) {
                document.body.style.cursor = 'grab';
            }
        });
    }

    private toggleLock() {
        const isLocked = this.element.classList.toggle('locked');
        this.header.style.cursor = isLocked ? 'default' : 'grab';
    }

    private duplicate() {
        const newNode = new MarkdownNode(
            parseInt(this.element.style.left) + 20,
            parseInt(this.element.style.top) + 20
        );
        newNode.content.innerHTML = this.content.innerHTML;
        document.getElementById('canvas-nodes')?.appendChild(newNode.element);
    }

    getElement(): HTMLElement {
        return this.element;
    }
}