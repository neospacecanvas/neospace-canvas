import { ViewportManager } from './Viewport';
import { NodeStore } from './NodeStore';
import { NodeType } from '../types/types';

export class MarkdownNode {
    private element: HTMLElement;
    private header: HTMLElement;
    private content: HTMLElement;
    private toolbar: HTMLElement;
    private isDragging: boolean = false;
    private isSpacePressed: boolean = false;
    private readonly TOOLBAR_OFFSET = -55;
    private editor: HTMLDivElement | null = null;
    private preview: HTMLDivElement | null = null;
    private viewportManager: ViewportManager;
    private nodeStore: NodeStore;
    private id: string;
    private unsubscribeViewport: () => void;
    private unsubscribeStore: () => void;

    constructor(x: number = window.innerWidth / 2, y: number = window.innerHeight / 2) {
        this.viewportManager = ViewportManager.getInstance();
        this.nodeStore = NodeStore.getInstance();

        // Create node in store
        this.id = this.nodeStore.createNode(
            NodeType.MARKDOWN,
            { x, y },
            { width: 480, height: 320 },
            'Type markdown here...'
        );

        this.setupElement();
        this.setupHeader();
        this.setupContent();
        this.setupToolbar();
        this.setupDrag();
        this.setupResize();
        this.setupHoverEffects();

        this.unsubscribeViewport = this.viewportManager.subscribe(() => { });

        // Subscribe to store updates
        this.unsubscribeStore = this.nodeStore.subscribe(this.id, (_, node) => {
            if (node.content.type === NodeType.MARKDOWN) {
                const content = node.content.data.content;
                if (this.editor && this.editor.innerText !== content) {
                    this.editor.innerText = content;
                    this.preview!.innerHTML = this.parseMarkdown(content);
                }
            }

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
        this.element.className = 'node markdown-node';
        this.element.style.position = 'absolute';

        const node = this.nodeStore.getNode(this.id);
        if (node) {
            this.element.style.left = `${node.position.x}px`;
            this.element.style.top = `${node.position.y}px`;
            this.element.style.width = `${node.dimensions.width}px`;
            this.element.style.height = `${node.dimensions.height}px`;
        }
    }

    private setupHeader() {
        this.header = document.createElement('div');
        this.header.className = 'node-header';
        this.header.textContent = 'Markdown';
        this.element.appendChild(this.header);
    }

    private parseMarkdown(text: string): string {
        return text
            .replace(/^(#{1,6})\s+(.+)$/gm, (_, level, content) => {
                const size = 7 - level.length;
                return `<h${level.length} style="font-size: ${size * 0.25}rem; font-weight: bold; margin: 0.5em 0">${content}</h${level.length}>`;
            })
            .replace(/(?:^|\s)(#[a-zA-Z]\w*)/g,
                ' <span style="background: #e3f2fd; color: #1976d2; padding: 2px 6px; border-radius: 4px; font-size: 0.9em">$1</span>')
            .replace(/```([\s\S]*?)```/g, '<pre style="background: #f5f5f5; padding: 8px; border-radius: 4px; margin: 8px 0"><code>$1</code></pre>')
            .replace(/`([^`]+)`/g, '<code style="background: #f5f5f5; padding: 2px 4px; border-radius: 3px">$1</code>')
            .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
            .replace(/\*(.*?)\*/g, '<em>$1</em>')
            .replace(/^\s*[-*+]\s+(.+)$/gm, '<li style="margin-left: 20px">$1</li>')
            .replace(/(<li.*<\/li>)/s, '<ul style="list-style-type: disc; margin: 8px 0">$1</ul>')
            .replace(/\n/g, '<br>');
    }

    private setupContent() {
        this.content = document.createElement('div');
        this.content.className = 'node-content';

        this.editor = document.createElement('div');
        this.editor.className = 'markdown-editor';
        this.editor.contentEditable = 'true';
        this.editor.style.display = 'none';
        this.editor.style.minHeight = '100px';
        this.editor.style.padding = '12px';
        this.editor.style.outline = 'none';

        this.preview = document.createElement('div');
        this.preview.className = 'markdown-preview placeholder';
        this.preview.innerHTML = 'Type markdown here...';
        this.preview.style.minHeight = '100px';
        this.preview.style.padding = '12px';

        this.editor.addEventListener('keydown', (e) => {
            if (e.code === 'Space') e.stopPropagation();
        });

        this.preview.addEventListener('dblclick', () => {
            this.editor!.style.display = 'block';
            this.preview!.style.display = 'none';
            this.editor!.focus();
            if (this.editor!.innerText === 'Type markdown here...') {
                this.editor!.innerText = '';
            }
        });

        this.editor.addEventListener('input', () => {
            const content = this.editor!.innerText;
            this.preview!.innerHTML = this.parseMarkdown(content);

            // Update store
            this.nodeStore.updateNodeContent(this.id, {
                type: NodeType.MARKDOWN,
                data: { content }
            });

            if (content && content !== 'Type markdown here...') {
                this.preview!.classList.remove('placeholder');
            } else {
                this.preview!.classList.add('placeholder');
            }
        });

        this.editor.addEventListener('blur', (e) => {
            if (!this.element.contains(e.relatedTarget as Node)) {
                this.editor!.style.display = 'none';
                this.preview!.style.display = 'block';
                if (!this.editor!.innerText.trim()) {
                    this.editor!.innerText = 'Type markdown here...';
                    this.preview!.innerHTML = 'Type markdown here...';
                    this.preview!.classList.add('placeholder');
                }
            }
        });

        this.editor.innerText = 'Type markdown here...';

        this.content.appendChild(this.editor);
        this.content.appendChild(this.preview);
        this.element.appendChild(this.content);
    }

    private setupToolbar() {
        this.toolbar = document.createElement('div');
        this.toolbar.className = 'node-toolbar';
        this.toolbar.style.display = 'none';
        this.toolbar.style.top = `${this.TOOLBAR_OFFSET}px`;

        const tools = [
            { icon: 'H2', label: 'Header', action: () => this.insertMarkdown('## ') },
            { icon: 'B', label: 'Bold', action: () => this.insertMarkdown('**', '**') },
            { icon: 'I', label: 'Italic', action: () => this.insertMarkdown('*', '*') },
            { icon: '`', label: 'Code', action: () => this.insertMarkdown('`', '`') },
            { icon: '•', label: 'List', action: () => this.insertMarkdown('- ') },
            { icon: '#', label: 'Tag', action: () => this.insertMarkdown('#') },
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

    private insertMarkdown(prefix: string, suffix: string = '') {
        if (this.editor!.style.display === 'none') {
            this.editor!.style.display = 'block';
            this.preview!.style.display = 'none';
            this.editor!.focus();
        }

        const selection = window.getSelection();
        const range = selection?.getRangeAt(0);

        if (range) {
            const text = prefix + range.toString() + suffix;
            document.execCommand('insertText', false, text);

            // Update store with new content
            this.nodeStore.updateNodeContent(this.id, {
                type: NodeType.MARKDOWN,
                data: { content: this.editor!.innerText }
            });

            this.preview!.innerHTML = this.parseMarkdown(this.editor!.innerText);
        }
    }

    private setupDrag() {
        let startX: number;
        let startY: number;
        let currentLeft: number;
        let currentTop: number;

        this.header.addEventListener('mousedown', (e) => {
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

            // Update store
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

    private setupResize() {
        const handles = ['se', 'sw', 'ne', 'nw'];

        handles.forEach(direction => {
            const handle = document.createElement('div');
            handle.className = `resize-handle resize-${direction}`;

            let isResizing = false;
            let startWidth: number;
            let startHeight: number;
            let startX: number;
            let startY: number;
            let startLeft: number;
            let startTop: number;

            handle.addEventListener('mousedown', (e) => {
                isResizing = true;
                e.stopPropagation();
                startX = e.clientX;
                startY = e.clientY;

                startWidth = this.element.offsetWidth;
                startHeight = this.element.offsetHeight;
                startLeft = parseInt(this.element.style.left) || 0;
                startTop = parseInt(this.element.style.top) || 0;

                document.body.style.cursor = `${direction}-resize`;
            });

            window.addEventListener('mousemove', (e) => {
                if (!isResizing) return;

                const { scale } = this.viewportManager.getState();
                const dx = (e.clientX - startX) / scale;
                const dy = (e.clientY - startY) / scale;

                let newWidth = startWidth;
                let newHeight = startHeight;
                let newLeft = startLeft;
                let newTop = startTop;

                if (direction.includes('e')) {
                    newWidth = Math.min(Math.max(startWidth + dx, 200), 800);
                } else if (direction.includes('w')) {
                    const proposedWidth = startWidth - dx;
                    if (proposedWidth >= 200 && proposedWidth <= 800) {
                        newWidth = proposedWidth;
                        newLeft = startLeft + dx;
                    }
                }

                if (direction.includes('s')) {
                    newHeight = Math.min(Math.max(startHeight + dy, 150), 600);
                } else if (direction.includes('n')) {
                    const proposedHeight = startHeight - dy;
                    if (proposedHeight >= 150 && proposedHeight <= 600) {
                        newHeight = proposedHeight;
                        newTop = startTop + dy;
                    }
                }

                // Update dimensions
                this.nodeStore.updateNodeDimensions(this.id, {
                    width: newWidth,
                    height: newHeight
                });

                // Update position
                this.nodeStore.updateNodePosition(this.id, {
                    x: newLeft,
                    y: newTop
                });
            });

            window.addEventListener('mouseup', () => {
                if (isResizing) {
                    isResizing = false;
                    document.body.style.cursor = '';
                }
            });

            this.element.appendChild(handle);
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

    private duplicate() {
        const node = this.nodeStore.getNode(this.id);
        if (!node || node.content.type !== NodeType.MARKDOWN) return;

        const newNode = new MarkdownNode(
            parseInt(this.element.style.left) + 20,
            parseInt(this.element.style.top) + 20
        );

        // Copy the content from the current node
        newNode.editor!.innerText = this.editor!.innerText;
        newNode.preview!.innerHTML = this.parseMarkdown(newNode.editor!.innerText);
        if (newNode.editor!.innerText !== 'Type markdown here...') {
            newNode.preview!.classList.remove('placeholder');
        }

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