<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { getSeasonStore } from '@/stores/Season.ts'
import { DataTable } from 'datatables.net-vue3'
import { type Config as DataTableConfig } from 'datatables.net-dt'

const tableData = computed(() => {
    let tableData = [];
    const mapData = getSeasonStore().stats;
    for (const [combatantId, stats] of mapData) {
        let row = [];

        const combatantInstance = getSeasonStore().worldState.combatants.find(com => com.id == combatantId);
        row.push(combatantInstance?.name || "<fixme>");
        row.push('zj-todo');
        for (const field in stats) {
            row.push((stats as any)[field]);
        }

        tableData.push(row);
    }

    return tableData;
});

const createdRowCallback = (
    row: HTMLTableRowElement,
    data: any[] | object,
    dataIndex: number,
    cells: HTMLTableCellElement[]
) => {
    const combatant_column_index = 0;
    const combatant_cell = cells[combatant_column_index];
    if (combatant_cell == null) {
        return;
    }

    // ZJ-TODO: link to combatant's page
    // combatant_cell.innerHTML = `<a href="">${combatant_cell.innerHTML}</a>`;
};

const tableOptions: DataTableConfig = {
    autoWidth: true,
    createdRow: createdRowCallback,
    orderMulti: true,
    order: {
        idx: 2,
        dir: "desc",
    },
    columnDefs: [
        {
            targets: "_all",
            orderSequence: ["desc", "asc", ""]
        }
    ]
};

onMounted(async () => {
    await getSeasonStore().fetchLatestWorldState()
    await getSeasonStore().fetchSeason()
    await getSeasonStore().fetchSeasonStats()
});
</script>

<template>
    <main>
        <DataTable
            id="stats"
            class="display"
            :data="tableData"
            :options="tableOptions"
        >
            <thead>
                <tr>
                    <th>Combatant</th>
                    <th>Team</th>
                    <th>Points</th>
                    <th>Throws</th>
                    <th>Hits</th>
                    <th>Shoves</th>
                </tr>
            </thead>
        </DataTable>
    </main>
</template>

<style scoped>
@import 'https://cdn.datatables.net/2.3.2/css/dataTables.dataTables.css';

main {
    overflow: scroll;
}
</style>