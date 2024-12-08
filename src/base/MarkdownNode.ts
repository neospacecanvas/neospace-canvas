import { AbstractNode } from './AbstractNode';
import { NodeType } from '../types/types';

export class MarkdownNode extends AbstractNode {
    private editor: HTMLDivElement | null = null;
    private preview: HTMLDivElement | null = null;

    constructor(x?: number, y?: number) {
        super(NodeType.MARKDOWN, '', x, y, 480, 320);
        this.element.classList.add('markdown-node');
        this.setupHeader();
        this.setupContent();
    }

    private setupHeader() {
        const header = document.createElement('div');
        header.className = 'node-header';
        header.textContent = 'Markdown';
        this.element.appendChild(header);
    }

    private setupContent() {
        const content = document.createElement('div');
        content.className = 'node-content';

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
            if (!this.element.contains(e.relatedTarget as HTMLElement)) {
                this.editor!.style.display = 'none';
                this.preview!.style.display = 'block';
                if (!this.editor!.innerText.trim()) {
                    this.editor!.innerText = 'Type markdown here...';
                    this.preview!.innerHTML = 'Type markdown here...';
                    this.preview!.classList.add('placeholder');
                }
            }
        });

        content.appendChild(this.editor);
        content.appendChild(this.preview);
        this.element.appendChild(content);
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

    public setContent(content: string): void {
        if (this.editor && this.preview) {
            this.editor.innerText = content;
            this.preview.innerHTML = this.parseMarkdown(content);
            this.preview.classList.remove('placeholder');

            this.nodeStore.updateNodeContent(this.id, {
                type: NodeType.MARKDOWN,
                data: { content }
            });
        }
    }

    public getContent(): string {
        return this.editor?.innerText || '';
    }

    protected duplicate(): void {
        const newNode = new MarkdownNode(
            parseInt(this.element.style.left) + 20,
            parseInt(this.element.style.top) + 20
        );

        const content = this.getContent();
        if (content !== 'Type markdown here...') {
            newNode.setContent(content);
        }

        document.getElementById('canvas-nodes')?.appendChild(newNode.getElement());
    }
}