import { DataTableColumn } from "naive-ui";

export interface RowData {
    uid: string;
    sku: string;
    metadata: any;
}

export interface RepositoryCallback {
    load: () => RowData[] | Promise<RowData[]>;
    get_by_uid: (
        uid: string
    ) => RowData | undefined | Promise<RowData | undefined>;
    get_by_sku: (
        sku: string
    ) => RowData | undefined | Promise<RowData | undefined>;
    add: (sku: string, metadata: any) => void | Promise<void>;
    rm: (uid: string) => void | Promise<void>;
    format_metadata: (metadata: any) => string;
}

export const columnHeaders: DataTableColumn<RowData>[] = [
    {
        type: "expand",
        renderExpand: (rowData) => {
            return `${rowData.metadata}`;
        },
    },
    {
        key: "uid",
        title: "UID",
        width: "20rem",
        className: "mono-column",
        filter(v: string | number, row: RowData) {
            return row.uid.includes(String(v));
        },
    },
    {
        key: "sku",
        title: "SKU",
        width: "20rem",
        className: "mono-column",
        filter(v: string | number, row: RowData) {
            return row.sku.includes(String(v));
        },
    },
];
