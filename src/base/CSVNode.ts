import { Coordinate, NodeType, CSVData } from "@/types/types";
import { Node } from './Node';

export class CSVNode extends Node {
    private fileName: string;

    constructor(position: Coordinate, fileName: string, csvData: CSVData) {
        super(
            position,
            { type: NodeType.CSV, data: csvData },
            { width: 100, height: 120 }
        );
        this.fileName = fileName;
    }

    protected getNodeClassName(): string {
        return 'csv-node';
    }

    protected renderContent(): string {
        // For debugging, let's use a simple emoji instead of an image first
        return `
            <div class="node-header">CSV File</div>
            <div class="node-content">
                <div class="node-icon">📊</div>
                <div class="node-filename">${this.fileName}</div>
            </div>
        `;
    }
}