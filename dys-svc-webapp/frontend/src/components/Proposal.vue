<script setup lang="ts">
import { computed, onMounted, type Ref, ref } from 'vue';
import { Proposal, VoteOnProposalRequest } from '%/services/vote/vote.ts';
import { fetchApi } from '@/utils.ts';

const props = defineProps(['proposal']);

const proposal: Ref<Proposal | null> = ref(null);
const votedOnOption: Ref<number | null> = ref(null);
const hasVoted = computed(() => votedOnOption.value !== null);

const sendVote = async (proposalId: number, optionId: number) => {
    const request = VoteOnProposalRequest.create();
    request.proposalId = proposalId;
    request.optionId = optionId;

    const response = await fetchApi('vote', {
        method: 'POST',
        body: JSON.stringify(VoteOnProposalRequest.toJSON(request)),
    });

    if (response.status === 200) {
        votedOnOption.value = optionId;
    }
};

onMounted(async () => {
    proposal.value = props.proposal;
});
</script>

<template>
    <div>
        <p>
            <strong>{{ proposal?.proposalName }}</strong>
        </p>
        <p>{{ proposal?.proposalDesc }}</p>
        <button
            v-for="option in proposal?.proposalOptions"
            @click="sendVote(proposal!.proposalId!, option!.optionId!)"
            :key="option.optionId"
            :disabled="hasVoted"
            :class="[votedOnOption === option.optionId ? 'chosen' : '']"
        >
            {{ option.optionName }}
        </button>
    </div>
</template>

<style scoped>
div {
    border-style: dashed;
    border-width: 1px;
    padding-left: 5px;
}

div button:disabled.chosen {
    background-color: lightgreen;
}
</style>
