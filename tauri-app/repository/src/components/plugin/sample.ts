import { reactive } from "vue";
import { RepositoryCallback, RowData } from "./interface";

const rows = reactive(create_test_data(100));

const sampleRepoCallback: RepositoryCallback = {
    load: () => {
        return rows;
    },
    get_by_uid: (uid: string) => {
        return rows.find((row) => row.uid === uid);
    },
    get_by_sku: (sku: string) => {
        return rows.find((row) => row.sku === sku);
    },
    add: (sku: string, metadata: string) => {
        const row = {
            uid: crypto.randomUUID().toString(),
            sku,
            metadata,
        };
        rows.push(row);
    },
    rm: (uid: string) => {
        const index = rows.findIndex((row) => row.uid === uid);
        if (index !== -1) {
            rows.splice(index, 1);
        }
    },
    format_metadata: (metadata: string) => {
        return metadata;
    },
};

function create_test_data(num: number) {
    const data: RowData[] = [];
    for (let i = 0; i < num; i++) {
        const uid = crypto.randomUUID().toString();
        data.push({
            uid,
            sku: `SKU${i + 1}`,
            metadata: `Metadata ${i + 1}`,
        });
    }
    return data;
}

export const sampleRepo = {
    repo_name: "Sample Repository",
    callback: sampleRepoCallback,
};
