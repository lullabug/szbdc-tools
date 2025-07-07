import { invoke } from "@tauri-apps/api/core";
import { RepositoryCallback, RowData } from "./interface";

async function load() {
    try {
        const rs: RowData[] = await invoke("fumo_load");
        return rs;
    } catch (e) {
        console.error("Error loading database data:", e);
        return [];
    }
}

async function get_by_uid(uid: string) {
    const rs: RowData | undefined =
        (await invoke("fumo_get_by_uid", { uid })) ?? undefined;
    return rs;
}

async function get_by_sku(sku: string) {
    const rs: RowData | undefined =
        (await invoke("fumo_get_by_sku", { sku })) ?? undefined;
    return rs;
}

async function add(sku: string, metadata: string) {
    try {
        await invoke("fumo_add", { sku, metadata });
    } catch (e) {
        console.error("Error adding data:", e);
    }
}

async function remove(uid: string) {
    try {
        await invoke("fumo_remove", { uid });
    } catch (e) {
        console.error("Error removing data:", e);
    }
}

const fumoRepoCallback: RepositoryCallback = {
    load,
    get_by_uid,
    get_by_sku,
    add,
    rm: remove,
    format_metadata: (metadata: string) => {
        return metadata;
    },
};

export const fumoRepo = {
    repo_name: "Fumo Repository",
    callback: fumoRepoCallback,
};
