<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { getSeasonStore } from '@/stores/Season.ts'
import { DataTable } from 'datatables.net-vue3'
import { type Config as DataTableConfig } from 'datatables.net-dt'

const tableData = computed(() => {
    let tableData = [];
    const mapData = getSeasonStore().stats;
    for (const [combatantId, stats] of mapData) {
        let row: (number | null)[] = [];
        row.push(combatantId);

        // ZJ-TODO: this entire block should be removed
        //          combatants should hold references to the team they play for
        let teamId: number | null = null;
        for (const [_, team] of Object.entries(getSeasonStore().worldState.teams)) {
            if (teamId) {
                break;
            }

            // ZJ-TODO: this should be combatant, not teamCombatantId, but shit's broke yo
            for (const teamCombatantId of team?.combatants!) {
                if (teamId) {
                    break;
                }

                // @ts-ignore: above ZJ-TODO
                if (teamCombatantId == combatantId) {
                    teamId = team?.id || null;
                    break;
                }
            }
        }
        row.push(teamId);
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

    const combatantId = Number(combatant_cell.innerText);
    const combatantName = getSeasonStore().worldState.combatants[combatantId]?.name!;
    combatant_cell.innerHTML = `<a href="/combatant/${combatantId}">${combatantName}</a>`;

    const team_column_index = 1;
    const team_cell = cells[team_column_index];
    if (team_cell == null) {
        return;
    }

    const teamId = Number(team_cell.innerText);
    const teamName = getSeasonStore().worldState.teams[teamId]?.name!;
    // ZJ-TODO: once team pages are introduced, uncomment this line
    // team_cell.innerHTML = `<a href="/team/${teamId}">${teamName}</a>`;
    team_cell.innerText = teamName;
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