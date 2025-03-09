<script setup lang="ts">
import {computed, inject, onMounted, type Ref, ref} from "vue";
import {Proposal as ProposalT, ProposalOption} from "%/services/vote/vote.ts";
import Proposal from "@/components/Proposal.vue";
import {getAuthStore} from "@/stores/Auth.ts";

const proposals: Ref<ProposalT[] | null> = ref(null);
const hasProposals = computed(() => proposals.value && proposals.value.length > 0);
const authStore = getAuthStore();
const isAuthed = computed(() => authStore.cookie.length > 0);

onMounted(async () => {
  const proposalObjects = JSON.parse(await (await fetch("/api/get_voting_proposals")).json());
  let proposalArray = [];
  for (let proposalObj of proposalObjects["proposals"]) {
    proposalObj["proposalId"] = proposalObj["proposal_id"];
    proposalObj["proposalName"] = proposalObj["proposal_name"];
    proposalObj["proposalDesc"] = proposalObj["proposal_desc"];

    let newOptionsArray = [];
    for (let option of proposalObj["proposal_options"]) {
      option["optionId"] = option["option_id"];
      option["optionName"] = option["option_name"];
      option["optionDesc"] = option["option_desc"];
      delete option["option_id"];
      delete option["option_name"];
      delete option["option_desc"];
      newOptionsArray.push(ProposalOption.fromJSON(option));
    }
    proposalObj["proposalOptions"] = newOptionsArray;
    proposalArray.push(ProposalT.fromJSON(proposalObj));
  }

  proposals.value = proposalArray;
});
</script>

<template>
  <main>
    <div v-if="hasProposals && isAuthed">
      <h1>Proposals</h1>
      <div id="proposals" v-for="proposal in proposals">
        <Proposal
          :key="proposal.proposalId"
          :proposal="proposal"
        />
      </div>
    </div>
    <div v-else>
      <article style="text-align: center; width: 50%; margin-left: auto; margin-right: auto">
        <p>DAX is a work in progress project, and many features are not implemented.</p>
        <p>Click a match result at the top to see a visualization of how the match played out.</p>
      </article>
    </div>
  </main>
</template>

<style scoped>
#proposals {
  padding-left: 5px;
}
</style>