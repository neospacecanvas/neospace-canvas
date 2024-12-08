import { AbstractNode } from './AbstractNode';
import { NodeType } from '../types/types';

export class CSVNode extends AbstractNode {
    private fileName: string;
    private iconContainer: HTMLDivElement;
    private fileNameElement: HTMLDivElement;

    constructor(fileName: string, csvContent: string, x?: number, y?: number) {
        // Create the CSV content object with initial data
        const lines = csvContent.trim().split('\n');
        const headers = lines[0].split(',').map(header => header.trim());
        const rows = lines.slice(1).map(line => 
            line.split(',').map(cell => cell.trim())
        );

        // Initialize with CSV content
        super(NodeType.CSV, '', x, y, 110, 130);
        
        // Store file name
        this.fileName = fileName;

        // Update node store with complete CSV data
        this.nodeStore.updateNodeContent(this.id, {
            type: NodeType.CSV,
            data: {
                fileName: this.fileName,
                headers,
                rows
            }
        });

        this.element.classList.add('csv-node');
        this.setupContent();
    }

    private setupContent() {
        const content = document.createElement('div');
        content.className = 'node-content';
        
        // Create icon container
        this.iconContainer = document.createElement('div');
        this.iconContainer.className = 'node-icon';
        
        const icon = document.createElement('img');
        icon.src = '/assets/csv.png';  // Make sure this path is correct
        icon.className = 'csv-icon';
        icon.alt = 'CSV file';
        icon.draggable = false;
        
        icon.addEventListener('dragstart', (e) => {
            e.stopPropagation();
        });
        
        this.iconContainer.appendChild(icon);
        
        // Create filename element
        this.fileNameElement = document.createElement('div');
        this.fileNameElement.className = 'node-filename';
        this.fileNameElement.textContent = this.fileName;
        
        content.appendChild(this.iconContainer);
        content.appendChild(this.fileNameElement);
        this.element.appendChild(content);
        
        content.addEventListener('dragstart', (e) => {
            e.preventDefault();
        });
    }

    protected duplicate(): void {
        const node = this.nodeStore.getNode(this.id);
        if (!node || node.content.type !== NodeType.CSV) return;

        const csvData = node.content.data;
        const csvContent = [
            csvData.headers.join(','),
            ...csvData.rows.map(row => row.join(','))
        ].join('\n');

        const newNode = new CSVNode(
            this.fileName,
            csvContent,
            parseInt(this.element.style.left) + 20,
            parseInt(this.element.style.top) + 20
        );
        document.getElementById('canvas-nodes')?.appendChild(newNode.getElement());
    }

    public getFileName(): string {
        return this.fileName;
    }

    public setFileName(fileName: string): void {
        this.fileName = fileName;
        if (this.fileNameElement) {
            this.fileNameElement.textContent = fileName;
        }
        
        // Update the node store
        const node = this.nodeStore.getNode(this.id);
        if (node && node.content.type === NodeType.CSV) {
            this.nodeStore.updateNodeContent(this.id, {
                type: NodeType.CSV,
                data: {
                    ...node.content.data,
                    fileName: fileName
                }
            });
        }
    }
}