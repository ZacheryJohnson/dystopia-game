<script setup lang="ts">

  import {onMounted, ref, type Ref} from "vue";

  // ZJ-TODO: this should be generated from protocol, not defined in both client + server
  type CombatantT = {
    combatantName: string,
    combatantTeamName: string,
  };
const combatants: Ref<CombatantT[]> = ref([]);

  onMounted(async () => {
    const combatant_results = JSON.parse((await (await fetch(`api/combatants`)).json()));

    combatants.value = [];
    for (const combatant of combatant_results) {
      console.log(combatant);
      const newCombatant: CombatantT = {
        combatantName: combatant["combatant_name"],
        combatantTeamName: combatant["team_name"],
      };

      combatants.value.push(newCombatant);
    }
  });
</script>

<template>
  <table>
    <thead>
      <tr>
        <th>Team</th>
        <th>Combatant Name</th>
      </tr>
    </thead>
    <tbody>
      <tr v-for="combatant in combatants" :key="combatant.combatantName">
        <td>{{ combatant.combatantTeamName }}</td>
        <td>{{ combatant.combatantName }}</td>
      </tr>
    </tbody>
  </table>
</template>

<style scoped>
table {
  width: 100%;
  border: 1px solid darkgray;
  border-collapse: collapse;
  table-layout: fixed;
  text-align: center;
}

th {
  font-weight: bolder;
}

th, td {
  border: 1px dotted;
}
</style>