import { CSVData } from "@/types/types";

export function parseCSV(csvText: string): CSVData {
    const lines = csvText.split('\n').map(line => 
        line.split(',').map(cell => cell.trim())
    );
    if (lines.length < 2) {
        throw new Error('This town ain\'t big enough for the two of us. AKA CSV was < 2 lines');
    }
    return {
        headers: lines[0],
        rows: lines.slice(1).filter(row => row.some(cell => cell.length > 0))
    };
}