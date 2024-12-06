import { AbstractNode } from './AbstractNode';
import { NodeType } from '../types/types';

export class CSVNode extends AbstractNode {
    constructor(fileName: string, csvContent: string, x?: number, y?: number) {
        super(NodeType.CSV, csvContent, x, y, 110, 130);
        this.element.classList.add('csv-node');
        this.setupContent(fileName);
    }

    private setupContent(fileName: string) {
        const content = document.createElement('div');
        content.className = 'node-content';
        
        const iconContainer = document.createElement('div');
        iconContainer.className = 'node-icon';
        
        const icon = document.createElement('img');
        icon.src = '../../public/assets/csv.png';
        icon.className = 'csv-icon';
        icon.alt = 'CSV file';
        icon.draggable = false;
        
        icon.addEventListener('dragstart', (e) => {
            e.stopPropagation();
        });
        
        iconContainer.appendChild(icon);
        
        const fileNameElement = document.createElement('div');
        fileNameElement.className = 'node-filename';
        fileNameElement.textContent = fileName;
        
        content.appendChild(iconContainer);
        content.appendChild(fileNameElement);
        this.element.appendChild(content);
        
        content.addEventListener('dragstart', (e) => {
            e.preventDefault();
        });
    }

    protected duplicate(): void {
        const node = this.nodeStore.getNode(this.id);
        if (!node || node.content.type !== NodeType.CSV) return;

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
}